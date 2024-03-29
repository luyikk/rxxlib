use anyhow::{Result};
use std::cell::UnsafeCell;
use crate::manager::ObjectManager;
use data_rw::Data;
use data_rw::DataReader;

#[cfg(not(feature ="Arc"))]
pub use sharedptr::Rc::SharedPtr;
#[cfg(not(feature ="Arc"))]
pub use std::rc::Rc;

#[cfg(feature ="Arc")]
pub use sharedptr::Arc::SharedPtr;
#[cfg(feature ="Arc")]
pub use std::sync::Arc;
use std::fmt::Debug;

/// 用于给类型返回TypeId
pub trait ISerdeTypeId{
    /// 返回当前TypeId
    fn type_id()->u16 where Self: Sized;
    /// 获取type id
    fn get_type_id(&self)->u16;
}


/// 序列化基本trait
#[cfg(not(feature ="Arc"))]
pub trait ISerde:ISerdeTypeId+Debug{
    /// 写入当前对象 到 BytesMut
    fn write_to(&self,om:&ObjectManager,data:&mut Data)->Result<()>;
    /// 从Bytes 装载当前对象
    fn read_from(&mut self,om:&ObjectManager,data:&mut DataReader)->Result<()>;
    fn debug(&self)->String{
        format!("{:?}",self)
    }
}

/// 序列化基本trait
#[cfg(feature ="Arc")]
pub trait ISerde:ISerdeTypeId+Debug+Send+Sync{
    /// 写入当前对象 到 BytesMut
    fn write_to(&self,om:&ObjectManager,data:&mut Data)->Result<()>;
    /// 从Bytes 装载当前对象
    fn read_from(&mut self,om:&ObjectManager,data:&mut DataReader)->Result<()>;

    fn debug(&self)->String{
        format!("{:?}",self)
    }
}

/// 常规结构类型序列化反序列化接口
pub trait IStruct{
    /// 写入当前对象 到 BytesMut
    fn write_to(&self,om:&ObjectManager,data:&mut Data)->Result<()>;
    /// 从Bytes 装载当前对象
    fn read_from(&mut self,om:&ObjectManager,data:&mut DataReader)->Result<()>;
}


/// 用于创建 共享指针
pub type CreateFn=fn() -> SharedPtr<dyn ISerde>;

/// 用于存储类型和TypeId的映射 实现根据Typeid 创建对象
pub struct TypeClass<const LEN:usize>{
   pub(crate) register_table:UnsafeCell<[Option<CreateFn>;LEN]>,
   pub(crate) register_name:UnsafeCell<Vec<(u16,&'static str)>>
}

unsafe impl<const LEN:usize> Send for TypeClass<LEN>{}
unsafe impl<const LEN:usize> Sync for TypeClass<LEN>{}

impl<const LEN:usize> TypeClass<LEN>{
    pub const fn new()->Self {
        TypeClass {
            register_table:UnsafeCell::new([None;LEN]),
            register_name:UnsafeCell::new(Vec::new())
        }
    }

    /// 注册TypeId
    pub fn register(&self,typeid:u16,name:&'static str,cfn:CreateFn){
        unsafe {
            (*self.register_table.get())[typeid as usize] = Some(cfn);
            (*self.register_name.get()).push((typeid,name));
        }
    }

    /// 根据typeid 创建对象
    #[inline]
    #[allow(clippy::manual_map)]
    pub fn create(&self,typeid:u16)->Option<SharedPtr<dyn ISerde>>{
        unsafe {
            if let Some(ref f)=(*self.register_table.get())[typeid as usize]{
                Some(f())
            }else{
                None
            }
            //(*self.register_table.get())[typeid as usize].as_ref().map(|f| f())
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
    /// 注意,如果不是SharePtr<dyn ISerde> cast Shardptr<T>
    /// 那么如果 un_cast 将会出现不可预料的错误
    fn un_cast(self)-> SharedPtr<dyn ISerde>;
}

impl<T:ISerde+'static> ITypeCaseToISerde for SharedPtr<T>{
     #[cfg(not(feature ="Arc"))]
     #[inline]
     fn un_cast(self) -> SharedPtr<dyn ISerde> {
         let ptr = &self as *const SharedPtr<T> as *const Rc<T>;
         std::mem::forget(self);
         unsafe {
             let rc = ptr.read() as Rc<dyn ISerde>;
             SharedPtr::from(rc)
         }
     }

    #[cfg(feature ="Arc")]
    #[inline]
    fn un_cast(self) -> SharedPtr<dyn ISerde> {
        let ptr = &self as *const SharedPtr<T> as *const Arc<T>;
        std::mem::forget(self);
        unsafe {
            let rc = ptr.read() as Arc<dyn ISerde>;
            SharedPtr::from(rc)
        }
    }
}