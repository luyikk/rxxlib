use xxlib::*;
use anyhow::*;
use sharedptr::Rc::SharedPtr;
use std::rc::Weak;



#[derive(Default,Debug,Clone)]
struct Foo{
    id:i32,
    name:String,
    child:Weak<Foo2>
}


impl ISerdeTypeId for Foo{
    #[inline(always)]
    fn type_id() -> u16 where Self: Sized {
        16
    }
}

impl ISerde for Foo{

    #[inline(always)]
    fn get_type_id(&self) -> u16 {
       Self::type_id()
    }

    #[inline(always)]
    fn write_to(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
        let bak=data.len();
        data.write_fixed(&0u32);
        om.write_(data,&self.id)?;
        om.write_(data,&self.name)?;
        om.write_(data,&self.child)?;
        data.write_fixed_at(bak,(data.len()-bak) as u32)?;
        Ok(())
    }
    #[inline(always)]
    fn read_from(&mut self, om: &ObjectManager, data:&mut DataReader)->Result<()> {
        let end_offset = data.read_fixed::<u32>()? as usize - 4usize;
        ensure!(end_offset<=data.len(),"struct:'{}' read_from offset error end_offset:{} > have len:{}", core::any::type_name::<Self>(),end_offset,data.len());
        let mut read = DataReader::from(&data[..end_offset]);
        if read.len()>0 {
            om.read_(&mut read, &mut self.id)?;
        }else{
            self.id=Default::default();
        }
        if read.len()>0 {
            om.read_(&mut read, &mut self.name)?;
        }else{
            self.name=Default::default();
        }
        if read.len()>0 {
            om.read_(&mut read, &mut self.child)?;
        }else{
            self.child=Default::default();
        }
        data.advance(end_offset)?;
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
}
impl ISerde for Foo2{

    #[inline(always)]
    fn get_type_id(&self) -> u16 {
        Foo2::type_id()
    }
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
    ObjectManager::register::<Foo>();
    ObjectManager::register::<Foo2>();
    let mut foo=Foo::default();
    foo.name="123123".to_string();
    foo.id=100;

    let mut foo2=Foo2::default();
    foo2.id=1000;
    foo2.base=SharedPtr::new(foo);

    let foo2_ptr=SharedPtr::new(foo2);
    unsafe {
        foo2_ptr.base.get_mut_ref().child = foo2_ptr.weak().ok_or_else(|| anyhow!("is none"))?;
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