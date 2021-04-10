use xxlib::manager::ObjectManager;
use anyhow::*;
use xxlib::types::{ISerde, ISerdeTypeId, ISerdeCaseToType};
use xxlib::data::Data;
use xxlib::data_read::DataReader;
use sharedptr::Rc::SharedPtr;
use std::rc::Weak;
use sharedptr::ISetNullWeak;
use std::time::Instant;
use std::fmt::Write;
use xxlib::StringAssign;


#[derive(Default)]
struct Foo{
    __offset:u32,
    id:i32,
    name:String,
    // p:Weak<Foo2>,
    // x:SharedPtr<Foo2>,
    // m:Vec<SharedPtr<Foo2>>
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

    #[inline(always)]
    fn write_to(&self, om: &ObjectManager, data: &mut Data) {
        om.write_(data,&self.id);
        om.write_(data,&self.name);
        // om.write_(data,&self.p);
        // om.write_(data,&self.x);
        // om.write_(data,&self.m);
    }
    #[inline(always)]
    fn read_from(&mut self, om: &ObjectManager, data:&mut DataReader)->Result<()> {
        om.read_(data, &mut self.id)?;
        om.read_(data, &mut self.name)?;
        // om.read_(data, &mut self.p)?;
        // om.read_(data, &mut self.x)?;
        // om.read_(data, &mut self.m)?;
        Ok(())
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

    let mut data = Data::with_capacity(1000);

    let p = ObjectManager::new();

    let mut foo = Foo::default();
    foo.id = 100;
    foo.name = "111111".to_string();
    // foo.p.set_null();
    // foo.x.set_null();
    // let mut foo2 = Foo2::default();
    // foo2.id = 1000;
    // foo.x=SharedPtr::new(foo2);
    // foo.p=foo.x.weak().ok_or_else(||anyhow!("is none"))?;
    // foo.m.push(foo.x.clone());

    let mut foo_ptr = SharedPtr::new(foo);

    for _ in 0..10 {
        data.clear();
        let start = Instant::now();
        for _ in 0..100i32 {
          // data.clear();
           // p.write_to(&mut data, &foo_ptr);
            //  data.write_var_integer(&foo.get_type_id());
             // data.write_var_integer(&foo.id);
             // data.write_var_integer(&foo.name);
            //data.write_var_integer(&i);
            p.write_(&mut data,&(1,"123123"));
        }

        println!("W {}", start.elapsed().as_secs_f32());

        let start = Instant::now();

        let mut dr = DataReader::from(&data[..]);
        let mut t:(i32,String)=Default::default();
        for _ in 0..100 {
            //x.read_var_integer::<i32>()?;
            //dr.read_var_integer::<i32>()?;
            //str.assign(dr.read_str()?);

            //foo_ptr=  p.read_ptr(&mut dr)?.cast()?;
           // p.read_from(&mut dr,&foo_ptr)?;
            p.read_(&mut dr,&mut t);
        }

        println!("R {} {}", start.elapsed().as_secs_f32(),t.1);
    }

    Ok(())
}