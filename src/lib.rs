pub mod manager;
pub mod types;

pub use data_rw::Data;
pub use data_rw::DataReader;

pub use manager::{ObjectManager,IReadInner,IWriteInner,filter_ids};
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
