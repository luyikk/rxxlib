use xxlib::*;
use anyhow::*;
use sharedptr::unsafe_def::IGetMutUnchecked;

#[derive(Default,Debug,Clone)]
struct Foo{
    __offset:u32,
    id:i32,
    name:String,
    child:Weak<Foo2>
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
        om.write_(data,&self.child)?;
        Ok(())
    }
    #[inline(always)]
    fn read_from(&mut self, om: &ObjectManager, data:&mut DataReader)->Result<()> {
        om.read_(data, &mut self.id)?;
        om.read_(data, &mut self.name)?;
        om.read_(data, &mut self.child)?;
        Ok(())
    }
}

#[derive(Default,Debug,Clone)]
struct Foo2{
    __offset:u32,
    base:SharedPtr<Foo>,
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
        om.write_(data,&self.base)?;
        om.write_(data,&self.id)?;
        Ok(())
    }
    #[inline(always)]
    fn read_from(&mut self, om: &ObjectManager, data: &mut DataReader)->Result<()> {
        om.read_(data,&mut self.base)?;
        om.read_(data, &mut self.id)?;
        Ok(())
    }
}



#[test]
pub fn test()->Result<()>{
    ObjectManager::register::<Foo>(stringify!(Foo));
    ObjectManager::register::<Foo2>(stringify!(Foo2));
    let mut foo=Foo::default();
    foo.name="123123".to_string();
    foo.id=100;

    let mut foo2=Foo2::default();
    foo2.id=1000;
    foo2.base=SharedPtr::new(foo);

    let foo2_ptr=SharedPtr::new(foo2);
    unsafe {
        foo2_ptr.base.get_mut_unchecked().child = foo2_ptr.weak().ok_or_else(|| anyhow!("is none"))?;
    }

    let mut data=Data::new();
    let om=ObjectManager::new();

    om.write_to(&mut data,&foo2_ptr)?;
    let mut dr=DataReader::from( &data[..]);
    let ptr= om.read_ptr(&mut dr)?.cast::<Foo2>()?;

    assert_eq!(foo2_ptr.id,ptr.id);
    assert_eq!(foo2_ptr.base.id,ptr.base.id);
    assert_eq!(foo2_ptr.base.name,ptr.base.name);
    assert_eq!(foo2_ptr.id,ptr.base.child.upgrade().ok_or_else(||anyhow!("none"))?.id);



    Ok(())
}