use xxlib::manager::ObjectManager;
use anyhow::*;
use xxlib::types::{ISerde, ISerdeTypeId};
use bytes::{Bytes, BytesMut};
use std::cell::{Cell, RefCell};
use sharedptr::Rc::SharedPtr;
use std::time::Instant;


#[derive(Default)]
struct Foo{
    __offset:u32,
    id:Cell<i32>,
    name:RefCell<String>
}
impl ISerdeTypeId for Foo{
    fn type_id() -> u16 where Self: Sized {
       16
    }
}

impl ISerde for Foo{

    fn get_offset_addr(&self) -> *mut u32 {
        &self.__offset as * const u32 as *mut u32
    }

    fn get_type_id(&self) -> u16 {
        Foo::type_id()
    }

    fn write_to(&self, om: &ObjectManager, data: &mut BytesMut) {
        om.write_(data,&self.id);
        om.write_(data,&self.name);
    }

    fn read_from(&self, _om: &ObjectManager, _data: &mut Bytes)->Result<()> {
        Ok(())
    }
}
impl Drop for Foo{
    fn drop(&mut self) {
        println!("foo is drop");
    }
}
#[derive(Default)]
struct Foo2{
    __offset:u32
}
impl ISerdeTypeId for Foo2{
    fn type_id() -> u16 where Self: Sized {
        32
    }
}
impl ISerde for Foo2{

    fn get_offset_addr(&self) -> *mut u32 {
        &self.__offset as * const u32 as *mut u32
    }

    fn get_type_id(&self) -> u16 {
       Foo2::type_id()
    }

    fn write_to(&self, _om: &ObjectManager, _data: &mut BytesMut) {

    }

    fn read_from(&self, _om: &ObjectManager, _data: &mut Bytes)->Result<()> {
        Ok(())
    }
}

fn main()->Result<()> {
    ObjectManager::register::<Foo>(16);
    ObjectManager::register::<Foo2>(32);

    let mut data=BytesMut::with_capacity(1024*1024*2);
    let p=ObjectManager::new();


    let foo=Foo::default();
    foo.id.set(100);
    *foo.name.borrow_mut()="111111".to_string();

    let foo_ptr=SharedPtr::new(foo);
    let start=Instant::now();
    for _ in 0..1000000 {
        p.write_to(&mut data, &foo_ptr);
    }

    println!("{}",start.elapsed().as_millis());
    Ok(())
}