#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_fn)]

pub mod data;
pub mod data_read;
pub mod manager;
pub mod types;

pub use manager::{ObjectManager,IReadInner,IWriteInner};
pub use data::Data;
pub use data_read::DataReader;
pub use types::{ISerdeTypeId, ISerde, IStruct,ISerdeCaseToType};

#[cfg(not(feature ="Arc"))]
pub use sharedptr::Rc::SharedPtr;
#[cfg(not(feature ="Arc"))]
pub use std::rc::Weak;
#[cfg(feature ="Arc")]
pub use sharedptr::Arc::SharedPtr;
#[cfg(feature ="Arc")]
pub use std::sync::Weak;

pub trait StringAssign{
    fn assign(&mut self,str:&str);
}

impl StringAssign for String{
    #[inline(always)]
    fn assign(&mut self, str: &str) {
        self.clear();
        self.push_str(str);
    }
}
