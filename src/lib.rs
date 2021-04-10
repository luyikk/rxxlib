#![feature(auto_traits)]
#![feature(negative_impls)]

pub mod data;
pub mod data_read;
pub mod manager;
pub mod types;

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
