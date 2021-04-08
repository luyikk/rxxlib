use xxlib::manager::ObjectManager;
use anyhow::*;
use xxlib::types::{ISerde, ISerdeTypeId};
use std::time::Instant;
use xxlib::data::Data;
use xxlib::data_read::DataReader;
use sharedptr::Rc::SharedPtr;


#[derive(Default)]
struct Foo{
    __offset:u32,
    id:i32,
    name:Vec<u8>
}
impl ISerdeTypeId for Foo{
    #[inline]
    fn type_id() -> u16 where Self: Sized {
       16
    }
}

impl ISerde for Foo{
    #[inline]
    fn get_offset_addr(&self) -> *mut u32 {
        &self.__offset as * const u32 as *mut u32
    }
    #[inline]
    fn get_type_id(&self) -> u16 {
        Foo::type_id()
    }

    #[inline]
    fn write_to(&self, om: &ObjectManager, data: &mut Data) {
        om.write_(data,&self.id);
        om.write_(data,&self.name);
    }
    #[inline]
    fn read_from(&self, _om: &ObjectManager, _data: &mut DataReader)->Result<()> {
        Ok(())
    }
}
impl Drop for Foo{
    #[inline]
    fn drop(&mut self) {
        println!("foo is drop");
    }
}
#[derive(Default)]
struct Foo2{
    __offset:u32
}
impl ISerdeTypeId for Foo2{
    #[inline]
    fn type_id() -> u16 where Self: Sized {
        32
    }
}
impl ISerde for Foo2{
    #[inline]
    fn get_offset_addr(&self) -> *mut u32 {
        &self.__offset as * const u32 as *mut u32
    }
    #[inline]
    fn get_type_id(&self) -> u16 {
       Foo2::type_id()
    }
    #[inline]
    fn write_to(&self, _om: &ObjectManager, _data: &mut Data) {

    }
    #[inline]
    fn read_from(&self, _om: &ObjectManager, _data: &mut DataReader)->Result<()> {
        Ok(())
    }
}

fn main()->Result<()> {
    ObjectManager::register::<Foo>(16);
    ObjectManager::register::<Foo2>(32);

    let mut data=Data::with_capacity(1024);
    let p=ObjectManager::new();
    let mut foo=Foo::default();
    foo.id=100;
    foo.name=b"111111".to_vec();

    let foo_ptr=SharedPtr::new(foo);

    let start=Instant::now();
    for _ in 0..10000000u32 {
        data.clear();
        p.write_to(&mut data, &foo_ptr);
       // data.clear();

        //data.write_var_integer(i);
    }

    println!("{}",start.elapsed().as_secs_f32());
    println!("{}",data.len());
    Ok(())
}