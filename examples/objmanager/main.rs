use xxlib::manager::ObjectManager;
use anyhow::*;
use xxlib::types::{ISerde, ISerdeTypeId, ISerdeCaseToType};
use xxlib::data::Data;
use xxlib::data_read::DataReader;
use sharedptr::Rc::SharedPtr;
use std::rc::Weak;
use sharedptr::ISetNullWeak;
use std::time::Instant;


#[derive(Default)]
struct Foo{
    __offset:u32,
    id:i32,
    name:String,
    // p:Weak<Foo2>,
    // x:SharedPtr<Foo2>
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
        // om.write_(data,&self.p);
        // om.write_(data,&self.x);
    }
    #[inline]
    fn read_from(&mut self, om: &ObjectManager, data:&mut DataReader)->Result<()> {
        om.read_(data, &mut self.id)?;
        om.read_(data, &mut self.name)?;
        // om.read_(data, &mut self.p)?;
        // om.read_(data, &mut self.x)?;
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
    __offset:u32,
    id:u64
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
    fn write_to(&self, om: &ObjectManager, data: &mut Data) {
        om.write_(data,&self.id);
    }
    #[inline]
    fn read_from(&mut self, om: &ObjectManager, data: &mut DataReader)->Result<()> {
        om.read_(data, &mut self.id)?;
        Ok(())
    }
}


fn main()->Result<()> {
    ObjectManager::register::<Foo>(16);
    ObjectManager::register::<Foo2>(32);

    let mut data = Data::with_capacity(100000000);

    let p = ObjectManager::new();

    let mut foo = Foo::default();
    foo.id = 100;
    foo.name = "111111".to_string();
    // foo.p.set_null();
    // foo.x.set_null();
    // let mut foo2 = Foo2::default();
    // foo2.id = 1000;

    let foo_ptr = SharedPtr::new(foo);

    let start=Instant::now();
    for _ in 0..10000000 {
        data.clear();
        p.write_to(&mut data, &foo_ptr);
    }

    println!("{}",start.elapsed().as_secs_f32());
    // let x = p.read_from(DataReader::from(&data[..]))?;
    // let f = x.cast::<Foo>()?;
    //
    // let x = f.p.upgrade().ok_or(anyhow!("11"))?;



    Ok(())
}