use sharedptr::Rc::SharedPtr;
use lazy_static::lazy_static;
use crate::types::{TypeClass, ISerde};
use std::rc::{Rc, Weak};
use std::cell::{RefCell, Cell, UnsafeCell};
use impl_trait_for_tuples::*;
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet};
use crate::data::Data;

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

/// 剔除 Option Weak<T>
pub auto trait SiftOption{}
impl<T:ISerde> !SiftOption for Weak<T>{}

pub auto trait NotU8{}
impl !NotU8 for u8{}

/// PKG 序列化 反序列化 的实现
#[derive(Default)]
pub struct ObjectManager{
    write_ptr_vec:UnsafeCell<Vec<*mut u32>>
}

impl ObjectManager{
    #[inline]
    pub fn new()->Self{
        ObjectManager{
            write_ptr_vec:UnsafeCell::new(Vec::new())
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
            let _hot = ClearWriteGuard {
                ptr_vec: &mut *self.write_ptr_vec.get()
            };
            if value.is_null() {
                panic!("write_to shared ptr not null")
            } else {
                self.write_sharedptr_entry(data, value);
            }
        }
    }

    /// 生成物结构内部写入
    #[inline]
    pub fn write_<T:IWriteInner>(&self,data:&mut Data,value:&T){
        value.write_(self,data);
    }

    /// 生成物结构内部写入 数组
    #[inline]
    pub fn write_array<T:IWriteInner>(&self,data:&mut Data,value:T){
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

    #[inline]
    fn write_ptr<T:ISerde>(&self,data:&mut Data,value:&SharedPtr<T>){
        value.write_to(self,data)
    }
}



impl<T:ISerde> IWriteInner for SharedPtr<T>{
    #[inline]
    fn write_(&self, om: &ObjectManager, data: &mut Data) {
         om.write_sharedptr(data,self);
    }
}

impl<T:ISerde> IWriteInner for Option<Weak<T>>{
    #[inline]
    fn write_(&self, om: &ObjectManager, data: &mut Data) {
        if let Some(weak) = self {
            weak.write_(om,data);
        }else {
            data.write_fixed(&0u8);
        }
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

impl <T:IWriteInner+SiftOption> IWriteInner for Option<T>{
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

