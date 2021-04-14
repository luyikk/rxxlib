#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_fn)]

pub mod data;
pub mod data_read;
pub mod manager;
pub mod types;

pub use manager::ObjectManager;
pub use data::Data;
pub use data_read::DataReader;
pub use types::{ISerdeTypeId, ISerde, ISerdeCaseToType};
pub use sharedptr::Rc::SharedPtr;

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


#[macro_export]
macro_rules! impl_irw_inner_for_enum {
    ($name:tt;$type:tt) => (
        impl xxlib::manager::IWriteInner for $name{
            #[inline]
            fn write_(&self, om: &ObjectManager, data: &mut Data) -> Result<()> {
                let v:$type=unsafe{
                     std::mem::transmute(*self)
                };
                om.write_(data,&v)?;
                Ok(())
            }
        }

        impl xxlib::manager::IReadInner for $name{
            #[inline]
            fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> Result<()> {
                let mut v:$type = 0;
                om.read_(data,&mut v)?;
                unsafe{
                    *self=std::mem::transmute(v)
                }
                Ok(())
            }
        }

        impl Default for $name{
            #[inline]
            fn default() -> Self {
                unsafe{
                    std::mem::transmute::<$type,$name>(0)
                }
            }
        }

    );
}
