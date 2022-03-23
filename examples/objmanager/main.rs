use xxlib::manager::ObjectManager;
use anyhow::*;
use xxlib::*;
use std::time::Instant;
use data_rw::Data;
use data_rw::DataReader;
use sharedptr::unsafe_def::IGetMutUnchecked;


#[derive(Default,Debug)]
struct Foo{
    id:i32,
    name:String,
    p:Weak<Foo2>,
    x:SharedPtr<Foo2>
}


impl ISerdeTypeId for Foo{
    #[inline(always)]
    fn type_id() -> u16 where Self: Sized {
       16
    }

    #[inline(always)]
    fn get_type_id(&self) -> u16 {
        Self::type_id()
    }
}

impl ISerde for Foo{

    #[inline(always)]
    fn write_to(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
        om.write_(data,&self.id)?;
        om.write_(data,&self.name)?;
        om.write_(data,&self.p)?;
        om.write_(data,&self.x)?;
        // om.write_(data,&self.m);
        Ok(())
    }
    #[inline(always)]
    fn read_from(&mut self, om: &ObjectManager, data:&mut DataReader)->Result<()> {
        om.read_(data, &mut self.id)?;
        om.read_(data, &mut self.name)?;
        om.read_(data, &mut self.p)?;
        om.read_(data, &mut self.x)?;
        // om.read_(data, &mut self.m)?;
        Ok(())
    }
}

#[derive(Default,Debug)]
struct Foo2{
    id:u64
}
impl ISerdeTypeId for Foo2{
    #[inline(always)]
    fn type_id() -> u16 where Self: Sized {
        32
    }
    #[inline(always)]
    fn get_type_id(&self) -> u16 {
        Foo2::type_id()
    }
}
impl ISerde for Foo2{

    #[inline(always)]
    fn write_to(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
        om.write_(data,&self.id)?;
        Ok(())
    }
    #[inline(always)]
    fn read_from(&mut self, om: &ObjectManager, data: &mut DataReader)->Result<()> {
        om.read_(data, &mut self.id)?;
        Ok(())
    }
}


 fn main() ->Result<()> {

     ObjectManager::register::<Foo>(stringify!(Foo));
     ObjectManager::register::<Foo2>(stringify!(Foo2));


     let mut data = Data::with_capacity(100000000);
     let p = ObjectManager::new();

    // let foo_ptr =ObjectManager::create(Foo::type_id()).ok_or_else(||anyhow!("none"))?.cast::<Foo>()?;


     let mut foo=Foo::default();
     foo.id=100;
     foo.name = "111111".to_string();

     let  foo_ptr =SharedPtr::new(foo);
     unsafe {
         let x=SharedPtr::new(Foo2{ id: 1 });
         foo_ptr.get_mut_unchecked().p =x.weak().unwrap();
         foo_ptr.get_mut_unchecked().x=x

     }

     println!("{:?}",foo_ptr);

     for _ in 0..10 {
         data.clear();
         let start = Instant::now();
         for _ in 0..10000000i32 {
             //data.clear();
             p.write_to(&mut data, &foo_ptr)?;
             //  data.write_var_integer(&foo.get_type_id());
              // data.write_var_integer(&foo.id);
              // data.write_var_integer(&foo.name);
             //data.write_var_integer(&i);
            // p.write_(&mut data,&(1,"123123"));
            // p.write_(&mut data, &foo);
         }

         println!("W {}", start.elapsed().as_secs_f32());


         let start = Instant::now();

         let mut dr = DataReader::from(&data[..]);
         //let mut t:(i32,String)=Default::default();
         for _ in 0..10000000 {
             //x.read_var_integer::<i32>()?;
             //dr.read_var_integer::<i32>()?;
             //str.assign(dr.read_str()?);

            // let foo_ptr=  p.read_ptr(&mut dr)?;
            // println!("{}",foo_ptr.debug());
             p.read_from(&mut dr,&foo_ptr)?;
           // p.read_(&mut dr,&mut t)?;
            // p.read_(&mut dr,&mut foo)?;
         }

         println!("R {}", start.elapsed().as_secs_f32());
     }

     Ok(())
 }
