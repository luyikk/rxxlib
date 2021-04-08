use sharedptr::Rc::SharedPtr;
use anyhow::{Result};
use std::cell::UnsafeCell;
use crate::manager::ObjectManager;
use crate::data::Data;
use crate::data_read::DataReader;

/// 用于给类型返回TypeId
pub trait ISerdeTypeId{
    /// 返回当前TypeId
    fn type_id()->u16 where Self: Sized;
}

/// 序列化基本trait
pub trait ISerde:ISerdeTypeId{
    /// 获取指针偏移量 用于收发包
    fn get_offset_addr(&self)->*mut u32;
    /// 获取type id
    fn get_type_id(&self)->u16;
    /// 写入当前对象 到 BytesMut
    fn write_to(&self,om:&ObjectManager,data:&mut Data);
    /// 从Bytes 装载当前对象
    fn read_from(&self,om:&ObjectManager,data:&mut DataReader)->Result<()>;
}

/// 用于创建 共享指针
pub type CreateFn=fn() -> SharedPtr<dyn ISerde>;

/// 用于存储类型和TypeId的映射 实现根据Typeid 创建对象
pub struct TypeClass{
    register_table:UnsafeCell<Vec<Option<CreateFn>>>
}

unsafe impl Send for TypeClass{}
unsafe impl Sync for TypeClass{}

impl TypeClass{
    pub fn new()->Self {
        let mut table:Vec<Option<CreateFn>>=Vec::with_capacity(65535);
        for _ in 0..65535{
            table.push(None);
        }
        let tmp = TypeClass {
            register_table:UnsafeCell::new(table)
        };
        tmp
    }

    /// 注册TypeId
    pub fn register(&self,typeid:u16,cfn:CreateFn){
        unsafe {
            (*self.register_table.get())[typeid as usize] =Some(cfn)
        }
    }

    /// 根据typeid 创建对象
    #[inline]
    pub fn create(&self,typeid:u16)->Option<SharedPtr<dyn ISerde>>{
        unsafe {
            if let Some(ref f)=(*self.register_table.get())[typeid as usize]{
                Some(f())
            }else{
                None
            }
        }
    }
}

///用于 实现 SharedPtr<dyn ISerde> 到 SharePtr<T>的转换
pub trait ISerdeCaseToType {
    /// 实现 SharedPtr<dyn ISerde> 到 SharePtr<T>的转换
    fn cast<T: ISerde+'static>(self) -> Result<SharedPtr<T>>
        where
            Self: Sized;
}

impl ISerdeCaseToType for SharedPtr<dyn ISerde> {
    #[inline]
    fn cast<T: ISerde+'static>(self) -> Result<SharedPtr<T>> {
        anyhow::ensure!(self.get_type_id() == T::type_id(),"case type error {}->{}",self.get_type_id(),T::type_id());

        let ptr = &self as *const SharedPtr<dyn ISerde> as *const SharedPtr<T>;
        std::mem::forget(self);
        unsafe { Ok(ptr.read()) }
    }
}

pub trait ITypeCaseToISerde{
    /// 实现 SharedPtr<T> 到 SharePtr<dyn ISerde>的转换
    fn un_cast(self)-> SharedPtr<dyn ISerde>;
}

impl<T:ISerde> ITypeCaseToISerde for SharedPtr<T>{
    fn un_cast(self) -> SharedPtr<dyn ISerde> {
        let ptr = &self as *const SharedPtr<T> as *const SharedPtr<dyn ISerde>;
        std::mem::forget(self);
        unsafe { ptr.read() }
    }
}