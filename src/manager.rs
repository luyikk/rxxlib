use crate::types::{TypeClass, ISerde, ISerdeCaseToType, ITypeCaseToISerde};
use std::cell::{RefCell, Cell, UnsafeCell};
use impl_trait_for_tuples::*;
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet};
use data_rw::Data;
use data_rw::DataReader;
use anyhow::*;
use crate::StringAssign;
use std::hash::Hash;


#[cfg(not(feature ="Arc"))]
pub use sharedptr::Rc::SharedPtr;
#[cfg(not(feature ="Arc"))]
use std::rc::{Rc, Weak};
#[cfg(feature ="Arc")]
pub use sharedptr::Arc::SharedPtr;
#[cfg(feature ="Arc")]
use std::sync::{Arc, Weak};



static TYPES:TypeClass<65535>=TypeClass::<65535>::new();

pub fn filter_ids(like:&str)->Vec<u16>{
    let mut table=Vec::new();
    unsafe {
        for (typeid,name) in (*TYPES.register_name.get()).iter().enumerate() {
            if let Some(name)=*name{
                if let Some(0)= name.find(like){
                    table.push(typeid as u16)
                }
            }
        }
    }
    table
}

/// 用于筛选 struct 内部写入 的类型判断
pub trait IWriteInner{
    fn write_(&self,om:&ObjectManager,data:&mut Data)->Result<()>;
}

#[impl_for_tuples(0, 50)]
impl IWriteInner for WriteTupleIdntifier{
    fn write_(&self, om: &ObjectManager, data: &mut Data) -> Result<()> {
        for_tuples!( #( WriteTupleIdntifier.write_(om,data)?; )* );
        Ok(())
    }
}


/// 用于筛选写入struct 字段
pub trait IReadInner{
    fn read_(&mut self,om:&ObjectManager,data:&mut DataReader)->Result<()>;
}

///实现 最大50个类型元素的元组 读取
#[impl_for_tuples(0,50)]
impl IReadInner for TupleIdentifier{
    #[inline]
    fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        for_tuples!( #( TupleIdentifier.read_(om,data)?; )* );
        Ok(())
    }
}

pub auto trait NotU8{}
impl !NotU8 for u8{}
impl NotU8 for String{}

/// PKG 序列化 反序列化 的实现
pub struct ObjectManager{
    write_ptr_vec:UnsafeCell<ordnung::Map<u64,u32>>,
    read_ptr_vec:UnsafeCell<Vec<SharedPtr<dyn ISerde>>>
}

impl Default for ObjectManager{
    #[inline]
    fn default() -> Self {
        ObjectManager::new()
    }
}

impl ObjectManager{
    #[inline]
    pub fn new()->Self{
        ObjectManager{
            write_ptr_vec:UnsafeCell::new(ordnung::Map::with_capacity(10)),
            read_ptr_vec:UnsafeCell::new(Vec::with_capacity(10))
        }
    }

    /// 注册 struct 和 Typeid 映射
    #[cfg(not(feature ="Arc"))]
    #[inline]
    pub fn register<T:Default+ ISerde+'static>(name:&'static str){
        TYPES.register(T::type_id(),name,||{
            SharedPtr::from(Rc::new(T::default()) as Rc<dyn ISerde>)
        });
    }

    /// 注册 struct 和 Typeid 映射
    #[cfg(feature ="Arc")]
    #[inline]
    pub fn register<T:Default+ ISerde+'static>(name:&'static str){
        TYPES.register(T::type_id(),name,||{
            SharedPtr::from(Arc::new(T::default()) as Arc<dyn ISerde>)
        });
    }

    /// 根据 Typeid 返回 SharedPtr<dyn ISerde>
    /// 如果没有注册 返回 None
    #[inline]
    pub fn create(typeid:u16)->Option<SharedPtr<dyn ISerde>>{
        TYPES.create(typeid)
    }

    /// 写入入口函数
    #[inline]
    pub fn write_to<T:ISerde>(&self,data:&mut Data,value:&SharedPtr<T>)->Result<()> {
        if value.is_null() {
            panic!("write_to shared ptr not null")
        } else {
            self.write_sharedptr_entry(data, value)?;
        }
        unsafe {
            (*self.write_ptr_vec.get()).clear();
        }
        Ok(())
    }

    /// 生成物结构内部写入
    #[inline]
    pub fn write_<T:IWriteInner>(&self,data:&mut Data,value:&T)->Result<()>{
        value.write_(self,data)
    }

    /// 写入共享指针入口
    #[inline]
    pub(crate) fn write_sharedptr_entry<T:ISerde>(&self,data:&mut Data,value:&SharedPtr<T>)->Result<()>{
        let offset_addr =value.as_ref() as *const T as u64;
        unsafe {
            (*self.write_ptr_vec.get()).insert(offset_addr,1);
        }
        data.write_var_integer(&value.get_type_id());
        self.write_ptr(data, value)
    }

    /// 写入共享指针
    #[inline]
    pub(crate) fn write_sharedptr<T:ISerde>(&self,data:&mut Data,value:&SharedPtr<T>)->Result<()>{
        if value.is_null(){
            data.write_fixed(&0u8);
        }else{

            unsafe {
                let value_addr=value.as_ref() as *const T as u64;
                let offset = (*self.write_ptr_vec.get()).get(&value_addr).unwrap_or(&0);
                if *offset == 0 {
                    let offset = (*self.write_ptr_vec.get()).len() as u32 + 1;
                    (*self.write_ptr_vec.get()).insert(value_addr,offset);
                    data.write_var_integer(&offset);
                    data.write_var_integer(&value.get_type_id());
                    self.write_ptr(data, value)?;
                } else {
                    data.write_var_integer(offset);
                }
            }
        }
        Ok(())
    }

    /// 写指针
    #[inline(always)]
    fn write_ptr<T:ISerde>(&self,data:&mut Data,value:&SharedPtr<T>)->Result<()>{
        value.write_to(self,data)
    }


    /// 读SharedPtr<ISerde>
    #[inline]
    pub fn read_ptr(&self, dr:&mut DataReader) ->Result<SharedPtr<dyn ISerde>>{
        unsafe {
            let r = self.read_ptr_first(dr);
            (*self.read_ptr_vec.get()).clear();
            r
        }
    }

    #[inline]
    fn read_ptr_first(&self,data:&mut DataReader)->Result<SharedPtr<dyn ISerde>> {
        let typeid: u16 = data.read_var_integer()?;
        let ptr = ObjectManager::create(typeid)
            .ok_or_else(move ||anyhow!("typeid not found:{}",typeid))?;
        unsafe {
            (*self.read_ptr_vec.get()).push(ptr.clone());
            ptr.get_mut_ref().read_from(self, data)?;
        }
        Ok(ptr)
    }

    #[inline]
    pub fn read_from<T:ISerde+'static>(&self,dr:&mut DataReader,ptr:&SharedPtr<T>)->Result<()>{
        unsafe {
            let r = self.read_from_first(dr,ptr);
            (*self.read_ptr_vec.get()).clear();
            r
        }
    }

    #[inline]
    fn read_from_first<T:ISerde+'static>(&self,data:&mut DataReader,ptr:&SharedPtr<T>)->Result<()> {
        let typeid: u16 = data.read_var_integer()?;
        ensure!(typeid==ptr.get_type_id(),"typeid error,{}!={}",typeid,ptr.get_type_id());
        unsafe {
            (*self.read_ptr_vec.get()).push(ptr.clone().un_cast());
            ptr.get_mut_ref().read_from(self, data)?;
        }
        Ok(())
    }

    #[inline(always)]
    pub fn read_<T:IReadInner>(&self, data:&mut DataReader, v: &mut T) ->Result<()>{
        v.read_(self,data)
    }
}


///
///                             write
///
impl<T:ISerde> IWriteInner for SharedPtr<T>{
    #[inline]
    fn write_(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
        om.write_sharedptr(data, self)
    }
}

impl<T:ISerde> IWriteInner for Weak<T>{
    #[inline]
    fn write_(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
        if let Some(ptr) = self.upgrade() {
            let ptr = SharedPtr::from(ptr);
            om.write_sharedptr(data, &ptr)?;
        }else{
            data.write_fixed(&0u8);
        }
        Ok(())
    }
}
impl <T:IWriteInner> IWriteInner for Option<T>{
    #[inline]
    fn write_(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
        if let Some(v)=self{
            data.write_fixed(&1u8);
            v.write_(om,data)?;
        }else{
            data.write_fixed(&0u8);
        }
        Ok(())
    }
}
impl<T:ISerde+Default> IWriteInner for T{
    #[inline]
    fn write_(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
        self.write_to(om,data)
    }
}
macro_rules! impl_iwrite_inner_number_var {
    ($type:ty) => (
    impl IWriteInner for $type{
        #[inline]
        fn write_(&self, _: &ObjectManager, data: &mut Data)->Result<()> {
            data.write_var_integer(self);
            Ok(())
        }
    });
}
impl_iwrite_inner_number_var!(u16);
impl_iwrite_inner_number_var!(i16);
impl_iwrite_inner_number_var!(u32);
impl_iwrite_inner_number_var!(i32);
impl_iwrite_inner_number_var!(u64);
impl_iwrite_inner_number_var!(i64);
impl_iwrite_inner_number_var!(String);
impl IWriteInner for &str{
    #[inline]
    fn write_(&self, _: &ObjectManager, data: &mut Data)->Result<()> {
        data.write_var_integer(self);
        Ok(())
    }
}
macro_rules! impl_iwrite_inner_number_fixed {
    ($type:ty) => (
        impl IWriteInner for $type{
            #[inline]
            fn write_(&self, _: &ObjectManager, data: &mut Data)->Result<()> {
                data.write_fixed(self);
                Ok(())
            }
        }
    );
}
impl_iwrite_inner_number_fixed!(i8);
impl_iwrite_inner_number_fixed!(u8);
impl_iwrite_inner_number_fixed!(bool);
impl_iwrite_inner_number_fixed!(f32);
impl_iwrite_inner_number_fixed!(f64);

impl <T:IWriteInner+NotU8> IWriteInner for Vec<T>{
    #[inline]
    fn write_(&self, om: &ObjectManager, data: &mut Data) ->Result<()>{
        data.write_var_integer(&(self.len() as u64));
        for x in self.iter() {
            x.write_(om,data)?;
        }
        Ok(())
    }
}
impl<T:IWriteInner+NotU8> IWriteInner for &[T]{
    #[inline]
    fn write_(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
        data.write_var_integer(&(self.len() as u64));
        for x in self.iter() {
            x.write_(om,data)?;
        }
        Ok(())
    }
}
impl IWriteInner for Vec<u8>{
    #[inline]
    fn write_(&self,_om: &ObjectManager, data: &mut Data)->Result<()> {
        data.write_var_integer(&(self.len() as u64));
        data.write_buf(self);
        Ok(())
    }
}
impl IWriteInner for &[u8]{
    #[inline]
    fn write_(&self, _om: &ObjectManager, data: &mut Data)->Result<()> {
        data.write_var_integer(&(self.len() as u64));
        data.write_buf(self);
        Ok(())
    }
}
macro_rules! impl_iwrite_inner_for_mapset {
    ($type:tt) =>(
    impl <K:IWriteInner> IWriteInner for $type::<K>{
        #[inline]
        fn write_(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
            data.write_var_integer(&(self.len() as u64));
            for k in self.iter() {
                k.write_(om, data)?;
            }
            Ok(())
        }
    });
}
impl_iwrite_inner_for_mapset!(HashSet);
impl_iwrite_inner_for_mapset!(BTreeSet);
macro_rules! impl_iwrite_inner_for_map {
    ($type:tt) => (
    impl <K:IWriteInner,V:IWriteInner> IWriteInner for $type<K,V>{
        #[inline]
        fn write_(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
            data.write_var_integer(&(self.len() as u64));
            for (k,v) in self.iter() {
                k.write_(om, data)?;
                v.write_(om, data)?;
            }
            Ok(())
        }
    }
    );
}
impl_iwrite_inner_for_map!(HashMap);
impl_iwrite_inner_for_map!(BTreeMap);
impl <T:IWriteInner> IWriteInner for RefCell<T>{
    fn write_(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
        self.borrow().write_(om,data)?;
        Ok(())
    }
}
impl <T:IWriteInner+Copy> IWriteInner for Cell<T>{
    fn write_(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
        self.get().write_(om,data)?;
        Ok(())
    }
}

///
///                         reader
///
macro_rules! impl_iread_object_for_var {
    ($type:ty) => (
    impl IReadInner for $type{
        #[inline]
        fn read_(&mut self, _om: &ObjectManager, data: &mut DataReader) -> Result<()> {
            *self = data.read_var_integer()?;
            Ok(())
        }
    });
}
impl_iread_object_for_var!(i16);
impl_iread_object_for_var!(u16);
impl_iread_object_for_var!(i32);
impl_iread_object_for_var!(u32);
impl_iread_object_for_var!(i64);
impl_iread_object_for_var!(u64);

macro_rules! impl_iread_inner_for_fixed {
    ($type:ty) => (
    impl IReadInner for $type{
        #[inline]
        fn read_(&mut self, _om: &ObjectManager, data: &mut DataReader) -> Result<()> {
            *self=data.read_fixed()?;
            Ok(())
        }
    });
}
impl_iread_inner_for_fixed!(i8);
impl_iread_inner_for_fixed!(u8);

impl IReadInner for f32{
    #[inline]
    fn read_(&mut self, _om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        *self=data.read_fixed()?;

        if self.is_nan(){
            bail!("NaN");
        }
        if self.is_infinite(){
            bail!("INFINITY");
        }
        Ok(())
    }
}

impl IReadInner for f64{
    #[inline]
    fn read_(&mut self, _om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        *self=data.read_fixed()?;
        if self.is_nan(){
            bail!("NaN");
        }
        if self.is_infinite(){
            bail!("INFINITY");
        }
        Ok(())
    }
}


impl IReadInner for bool{
    #[inline]
    fn read_(&mut self, _om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        *self=data.read_fixed::<u8>()?==1;
        Ok(())
    }
}
impl<T:ISerde> IReadInner for T{
    #[inline]
    fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        self.read_from(om,data)
    }
}

#[inline]
fn read_shared_ptr<T:ISerde+'static>(om: &ObjectManager, data: &mut DataReader, offset: usize)->Result<SharedPtr<T>> {
    unsafe {
        let len = (*om.read_ptr_vec.get()).len();
        if offset == len + 1 {
            let typeid = data.read_var_integer::<u16>()?;
            ensure!(typeid==T::type_id(),"read typeid:{} error,not type:{}",typeid,T::type_id());
            let ptr = ObjectManager::create(typeid)
                .ok_or_else(move || anyhow!("not found typeid:{}",typeid))?;
            (*om.read_ptr_vec.get()).push(ptr.clone());
            ptr.get_mut_ref().read_from(om, data)?;
            Ok(ptr.cast::<T>()?)
        } else {
            ensure!(offset<= len,"read type:{} offset error,offset:{} > vec len:{}",T::type_id(),offset,len);
            let ptr = (*om.read_ptr_vec.get()).get(offset - 1)
                .ok_or_else(move || anyhow!("read type:{} offset error,not found offset:{}",T::type_id(),offset))?.clone();
            ensure!(T::type_id()==ptr.get_type_id(),"read type:{} error offset type:{}",T::type_id(),ptr.get_type_id());
            Ok(ptr.clone().cast::<T>()?)
        }
    }
}
impl <T:ISerde+'static> IReadInner for SharedPtr<T>{
    #[inline]
    fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        let offset=data.read_var_integer::<u32>()? as usize;
        if offset==0{
            self.set_null();
            return  Ok(())
        }
        *self= read_shared_ptr::<T>(om, data, offset)?;
        Ok(())
    }
}


impl <T:ISerde+'static> IReadInner for Weak<T>{
    #[inline]
    fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        let offset = data.read_var_integer::<u32>()? as usize;
        if offset == 0 {
            *self = Default::default();
            return Ok(())
        }
        if let Some(ptr) = read_shared_ptr::<T>(om, data, offset)?.weak() {
            *self = ptr;
            Ok(())
        } else {
            bail!("shared ptr is null,type:{}",T::type_id())
        }
    }
}
impl IReadInner for String{
    #[inline(always)]
    fn read_(&mut self, _om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        self.assign(data.read_var_str()?);
        Ok(())
    }
}

impl IReadInner for Vec<u8>{
    #[inline(always)]
    fn read_(&mut self, _om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        self.clear();
        self.extend_from_slice(data.read_var_buf()?);
        Ok(())
    }
}

impl<T:IReadInner+ Default> IReadInner for Option<T>{
    #[inline]
    fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        if data.read_fixed::<u8>()? == 0 {
            *self = None
        } else {
            let mut x = T::default();
            x.read_(om, data)?;
            *self = Some(x)
        }
        Ok(())
    }
}

impl <T:IReadInner+Default+NotU8> IReadInner for Vec<T>{
    #[inline]
    fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        let len=data.read_var_integer::<u32>()?;
        self.clear();
        for _ in 0..len {
            let mut v=T::default();
            v.read_(om,data)?;
            self.push(v);
        }
        Ok(())
    }
}

impl <K:IReadInner+Default+Eq+Hash,V:IReadInner+Default> IReadInner for HashMap<K,V>{
    #[inline]
    fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        let len=data.read_var_integer::<u32>()?;
        self.clear();
        for _ in 0..len {
            let mut k=K::default();
            k.read_(om,data)?;
            let mut v=V::default();
            v.read_(om,data)?;
            self.insert(k,v);
        }
        Ok(())
    }
}
impl <K:IReadInner+Default+Ord,V:IReadInner+Default> IReadInner for BTreeMap<K,V>{
    #[inline]
    fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        let len=data.read_var_integer::<u32>()?;
        self.clear();
        for _ in 0..len {
            let mut k=K::default();
            k.read_(om,data)?;
            let mut v=V::default();
            v.read_(om,data)?;
            self.insert(k,v);
        }
        Ok(())
    }
}
impl <K:IReadInner+Default+Eq+Hash> IReadInner for HashSet<K>{
    #[inline]
    fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        let len=data.read_var_integer::<u32>()?;
        self.clear();
        for _ in 0..len {
            let mut k=K::default();
            k.read_(om,data)?;
            self.insert(k);
        }
        Ok(())
    }
}
impl <K:IReadInner+Default+Ord> IReadInner for BTreeSet<K>{
    #[inline]
    fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        let len=data.read_var_integer::<u32>()?;
        self.clear();
        for _ in 0..len {
            let mut k=K::default();
            k.read_(om,data)?;
            self.insert(k);
        }
        Ok(())
    }
}

