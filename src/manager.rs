use sharedptr::Rc::SharedPtr;
use lazy_static::lazy_static;
use crate::types::{TypeClass, ISerde};
use std::rc::Rc;



lazy_static!{
    static ref TYPES:TypeClass={
        let types=TypeClass::new();
        types
    };
}

/// PKG 序列化 反序列化 的实现
pub struct ObjectManager;

impl ObjectManager{
    #[inline]
    pub fn new()->Self{
        ObjectManager
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

    
}