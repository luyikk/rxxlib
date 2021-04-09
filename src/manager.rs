use sharedptr::Rc::SharedPtr;
use lazy_static::lazy_static;
use crate::types::{TypeClass, ISerde, ISerdeCaseToType};
use std::rc::{Rc, Weak};
use std::cell::{RefCell, Cell, UnsafeCell};
use impl_trait_for_tuples::*;
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet};
use crate::data::Data;
use crate::data_read::DataReader;
use anyhow::*;

lazy_static!{
    static ref TYPES:TypeClass={
        let types=TypeClass::new();
        types
    };
}

/// 用于清理写入 数据的时候产生的零时数据
struct ClearWriteGuard<'a>{
    ptr_vec:&'a mut Vec<*mut u32>
}

impl<'a> Drop for ClearWriteGuard<'a>{
    #[inline]
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.ptr_vec.len() {
                self.ptr_vec.as_mut_ptr().add(i).read().write(0);
            }
            self.ptr_vec.clear();
        }
    }
}

/// 用于筛选 struct 内部写入 的类型判断
#[impl_for_tuples(1, 50)]
pub trait IWriteInner{
    fn write_(&self,om:&ObjectManager,data:&mut Data);
}


/// 用于筛选写入struct 字段
pub trait IReadInner{
    fn read_(&mut self,om:&ObjectManager,data:&mut DataReader)->Result<()>;
}


pub auto trait NotU8{}
impl !NotU8 for u8{}

/// PKG 序列化 反序列化 的实现
#[derive(Default)]
pub struct ObjectManager{
    write_ptr_vec:UnsafeCell<Vec<*mut u32>>,
    read_ptr_vec:UnsafeCell<Vec<SharedPtr<dyn ISerde>>>
}

impl ObjectManager{
    #[inline]
    pub fn new()->Self{
        ObjectManager{
            write_ptr_vec:UnsafeCell::new(Vec::new()),
            read_ptr_vec:UnsafeCell::new(Vec::new())
        }
    }

    /// 注册 struct 和 Typeid 映射
    #[inline]
    pub fn register<T:Default+ ISerde+'static>(typeid:u16){
        TYPES.register(typeid,||{
            SharedPtr::from(Rc::new(T::default()) as Rc<dyn ISerde>)
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
    pub fn write_to<T:ISerde>(&self,data:&mut Data,value:&SharedPtr<T>){
        unsafe {
            // let _hot = ClearWriteGuard {
            //     ptr_vec: &mut *self.write_ptr_vec.get()
            // };
            if value.is_null() {
                panic!("write_to shared ptr not null")
            } else {
                self.write_sharedptr_entry(data, value);
            }

            let ptr = self.write_ptr_vec.get();
            let len = (*ptr).len();
            for i in 0..len {
                (*ptr).as_mut_ptr().add(i).read().write(0);
            }
            (*ptr).clear();
        }
    }

    /// 生成物结构内部写入
    #[inline]
    pub fn write_<T:IWriteInner>(&self,data:&mut Data,value:&T){
        value.write_(self,data);
    }

    /// 写入共享指针入口
    #[inline]
    pub(crate) fn write_sharedptr_entry<T:ISerde>(&self,data:&mut Data,value:&SharedPtr<T>){
        unsafe {
            let offst_addr=value.get_offset_addr();
            (*self.write_ptr_vec.get()).push(offst_addr);
            offst_addr.write(1);
            data.write_var_integer(&value.get_type_id());
            self.write_ptr(data, value);
        }
    }

    /// 写入共享指针
    #[inline]
    pub(crate) fn write_sharedptr<T:ISerde>(&self,data:&mut Data,value:&SharedPtr<T>){
        if value.is_null(){
            data.write_fixed(&0u8);
        }else{
            let value_addr=value.get_offset_addr();
            unsafe {
                let offset = value_addr.read();
                if offset == 0 {
                    (*self.write_ptr_vec.get()).push(value.get_offset_addr());
                    let offset = (*self.write_ptr_vec.get()).len() as u32;
                    value_addr.write(offset);
                    data.write_var_integer(&offset);
                    data.write_var_integer(&value.get_type_id());
                    self.write_ptr(data, value);
                } else {
                    data.write_var_integer(&offset);
                }
            }
        }
    }

    /// 写指针
    #[inline]
    fn write_ptr<T:ISerde>(&self,data:&mut Data,value:&SharedPtr<T>){
        value.write_to(self,data)
    }

    /// 读SharedPtr<ISerde>
    #[inline]
    pub fn read_from(&self,dr:DataReader)->Result<SharedPtr<dyn ISerde>>{
        unsafe {
            let r = self.read_ptr_first(dr);
            (*self.read_ptr_vec.get()).clear();
            r
        }
    }

    #[inline(always)]
    fn read_ptr_first(&self,mut data:DataReader)->Result<SharedPtr<dyn ISerde>> {
        let typeid: u16 = data.read_var_integer()?;
        let ptr = ObjectManager::create(typeid).ok_or(anyhow!("not found typeid:{}",typeid))?;
        unsafe {
            (*self.read_ptr_vec.get()).push(ptr.clone());
            ptr.get_mut_ref().read_from(self,&mut data)?;
        }
        Ok(ptr)
    }

    #[inline]
    pub fn read_<T:IReadInner>(&self, data:&mut DataReader, v: &mut T) ->Result<()>{
        v.read_(self,data)
    }
}


///
///                             write
///
impl<T:ISerde> IWriteInner for SharedPtr<T>{
    #[inline]
    fn write_(&self, om: &ObjectManager, data: &mut Data) {
         om.write_sharedptr(data,self);
    }
}

impl<T:ISerde> IWriteInner for Weak<T>{
    #[inline]
    fn write_(&self, om: &ObjectManager, data: &mut Data) {
        if let Some(ptr) = self.upgrade() {
            let ptr = SharedPtr::from(ptr);
            om.write_sharedptr(data, &ptr);
        }else{
            data.write_fixed(&0u8);
        }
    }
}
impl <T:IWriteInner> IWriteInner for Option<T>{
    #[inline]
    fn write_(&self, om: &ObjectManager, data: &mut Data) {
        if let Some(v)=self{
            data.write_fixed(&1u8);
            v.write_(om,data);
        }else{
            data.write_fixed(&0u8);
        }
    }
}
impl<T:ISerde+Default> IWriteInner for T{
    #[inline]
    fn write_(&self, om: &ObjectManager, data: &mut Data) {
        self.write_to(om,data);
    }
}
macro_rules! impl_iwrite_inner_number_var {
    ($type:tt) => (
    impl IWriteInner for $type{
        #[inline]
        fn write_(&self, _: &ObjectManager, data: &mut Data) {
            data.write_var_integer(self)
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
    fn write_(&self, _: &ObjectManager, data: &mut Data) {
        data.write_var_integer(self);
    }
}
macro_rules! impl_iwrite_inner_number_fixed {
    ($type:tt) => (
        impl IWriteInner for $type{
            #[inline]
            fn write_(&self, _: &ObjectManager, data: &mut Data) {
                data.write_fixed(self)
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
    fn write_(&self, om: &ObjectManager, data: &mut Data) {
        data.write_var_integer(&(self.len() as u64));
        for x in self.iter() {
            x.write_(om,data);
        }
    }
}
impl<T:IWriteInner+NotU8> IWriteInner for &[T]{
    #[inline]
    fn write_(&self, om: &ObjectManager, data: &mut Data) {
        data.write_var_integer(&(self.len() as u64));
        for x in self.iter() {
            x.write_(om,data);
        }
    }
}
impl IWriteInner for Vec<u8>{
    #[inline]
    fn write_(&self,_om: &ObjectManager, data: &mut Data) {
        data.write_var_integer(&(self.len() as u64));
        data.write_buf(self);
    }
}
impl IWriteInner for &[u8]{
    #[inline]
    fn write_(&self, _om: &ObjectManager, data: &mut Data) {
        data.write_var_integer(&(self.len() as u64));
        data.write_buf(self);
    }
}
macro_rules! impl_iwrite_inner_for_mapset {
    ($type:tt) =>(
    impl <K:IWriteInner> IWriteInner for $type::<K>{
        #[inline]
        fn write_(&self, om: &ObjectManager, data: &mut Data) {
            data.write_var_integer(&(self.len() as u64));
            for k in self.iter() {
                k.write_(om, data);
            }
        }
    });
}
impl_iwrite_inner_for_mapset!(HashSet);
impl_iwrite_inner_for_mapset!(BTreeSet);
macro_rules! impl_iwrite_inner_for_map {
    ($type:tt) => (
    impl <K:IWriteInner,V:IWriteInner> IWriteInner for $type<K,V>{
        #[inline]
        fn write_(&self, om: &ObjectManager, data: &mut Data) {
            data.write_var_integer(&(self.len() as u64));
            for (k,v) in self.iter() {
                k.write_(om, data);
                v.write_(om, data);
            }
        }
    }
    );
}
impl_iwrite_inner_for_map!(HashMap);
impl_iwrite_inner_for_map!(BTreeMap);
impl <T:IWriteInner> IWriteInner for RefCell<T>{
    fn write_(&self, om: &ObjectManager, data: &mut Data) {
        self.borrow().write_(om,data);
    }
}
impl <T:IWriteInner+Copy> IWriteInner for Cell<T>{
    fn write_(&self, om: &ObjectManager, data: &mut Data) {
        self.get().write_(om,data);
    }
}

///
///                         reader
///
macro_rules! impl_iread_object_for_var {
    ($type:tt) => (
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
impl_iread_object_for_var!(String);
macro_rules! impl_iread_inner_for_fixed {
    ($type:tt) => (
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
impl_iread_inner_for_fixed!(f32);
impl_iread_inner_for_fixed!(f64);
impl IReadInner for bool{
    #[inline]
    fn read_(&mut self, _om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        *self=if data.read_fixed::<u8>()?==1{true}else{false};
        Ok(())
    }
}
impl<T:ISerde> IReadInner for T{
    #[inline]
    fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> Result<()> {
        self.read_from(om,data)
    }
}

#[inline(always)]
fn read_shared_ptr<T:ISerde+'static>(om: &ObjectManager, data: &mut DataReader, offset: usize)->Result<SharedPtr<T>> {
    unsafe {
        let len = (*om.read_ptr_vec.get()).len();
        if offset == len + 1 {
            let typeid = data.read_var_integer::<u16>()?;
            ensure!(typeid==T::type_id(),"read typeid:{} error,not type:{}",typeid,T::type_id());
            let ptr = ObjectManager::create(typeid)
                .ok_or(anyhow!("not found typeid:{}",typeid))?;
            (*om.read_ptr_vec.get()).push(ptr.clone());
            ptr.get_mut_ref().read_from(om, data)?;
            Ok(ptr.cast::<T>()?)
        } else {
            ensure!(offset<= len,"read type:{} offset error,offset:{} > vec len:{}",T::type_id(),offset,len);
            let ptr = (*om.read_ptr_vec.get()).get(offset - 1)
                .ok_or(anyhow!("read type:{} offset error,not found offset:{}",T::type_id(),offset))?
                .clone();
            ensure!(T::type_id()==ptr.get_type_id(),"read type:{} error offset type:{}",T::type_id(),ptr.get_type_id());
            Ok(ptr.cast::<T>()?)
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
        let offset=data.read_var_integer::<u32>()? as usize;
        if offset==0{
            *self=Default::default();
            return  Ok(())
        }
        *self= read_shared_ptr::<T>(om, data, offset)?
            .weak()
            .ok_or(anyhow!("shared ptr is null,type:{}",T::type_id()))?;
        Ok(())
    }
}