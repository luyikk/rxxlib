use xxlib::manager::ObjectManager;
use anyhow::*;
use xxlib::types::{ISerde, ISerdeCaseToType, ISerdeTypeId};
use std::sync::atomic::{AtomicUsize, Ordering};
use bytes::{Bytes, BytesMut};


#[derive(Default)]
struct Foo{
    __offset:AtomicUsize
}
impl ISerdeTypeId for Foo{
    fn type_id() -> u16 where Self: Sized {
       16
    }
}

impl ISerde for Foo{
    fn set_offset(&self, value: usize) {
       self.__offset.store(value, Ordering::Release)
    }

    fn get_offset(&self) -> usize {
       self.__offset.load(Ordering::Acquire)
    }

    fn get_type_id(&self) -> u16 {
        Foo::type_id()
    }

    fn write(&self, _om: ObjectManager, _data: BytesMut) {
        unimplemented!()
    }

    fn read(&self, _om: ObjectManager, _data: Bytes)->Result<()> {
        unimplemented!()
    }
}
impl Drop for Foo{
    fn drop(&mut self) {
        println!("foo is drop");
    }
}
#[derive(Default)]
struct Foo2{
    __offset:AtomicUsize
}
impl ISerdeTypeId for Foo2{
    fn type_id() -> u16 where Self: Sized {
        32
    }
}
impl ISerde for Foo2{
    fn set_offset(&self, value: usize) {
        self.__offset.store(value, Ordering::Release)
    }

    fn get_offset(&self) -> usize {
        self.__offset.load(Ordering::Acquire)
    }

    fn get_type_id(&self) -> u16 {
       Foo2::type_id()
    }

    fn write(&self, _om: ObjectManager, _data: BytesMut) {
        todo!()
    }

    fn read(&self, _om: ObjectManager, _data: Bytes)->Result<()> {
        todo!()
    }
}

fn main()->Result<()> {
    ObjectManager::register::<Foo>(16);
    ObjectManager::register::<Foo2>(32);

    let obj=ObjectManager::create(16)
        .ok_or(anyhow!("not create 16"))?;

    let obj=obj.cast::<Foo>()?;
    obj.set_offset(1);
    println!("{}",obj.get_offset());


    Ok(())
}