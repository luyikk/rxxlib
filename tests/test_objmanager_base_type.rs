use anyhow::*;
use xxlib::Data;
use xxlib::DataReader;
use xxlib::manager::ObjectManager;
use xxlib::types::{ISerdeTypeId, ISerde};
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet};

#[test]
pub fn test_number()->Result<()>{
    let mut data=Data::new();
    let om=ObjectManager::new();

    om.write_(&mut data,&1i8)?;
    om.write_(&mut data,&2u8)?;
    om.write_(&mut data,&1i16)?;
    om.write_(&mut data,&2u16)?;
    om.write_(&mut data,&1i32)?;
    om.write_(&mut data,&2u32)?;
    om.write_(&mut data,&1i64)?;
    om.write_(&mut data,&2u64)?;
    om.write_(&mut data,&1f32)?;
    om.write_(&mut data,&2f64)?;

    let mut a:i8=0;
    let mut b:u8=0;
    let mut c:i16=0;
    let mut d:u16=0;
    let mut e:i32=0;
    let mut f:u32=0;
    let mut g:i64=0;
    let mut h:u64=0;
    let mut i:f32=0f32;
    let mut j:f64=0f64;

    let mut dr=DataReader::from(&data[..]);
    om.read_(&mut dr,&mut a)?;
    om.read_(&mut dr,&mut b)?;
    om.read_(&mut dr,&mut c)?;
    om.read_(&mut dr,&mut d)?;
    om.read_(&mut dr,&mut e)?;
    om.read_(&mut dr,&mut f)?;
    om.read_(&mut dr,&mut g)?;
    om.read_(&mut dr,&mut h)?;
    om.read_(&mut dr,&mut i)?;
    om.read_(&mut dr,&mut j)?;

    assert_eq!(a,1);
    assert_eq!(b,2);
    assert_eq!(c,1);
    assert_eq!(d,2);
    assert_eq!(e,1);
    assert_eq!(f,2);
    assert_eq!(g,1);
    assert_eq!(h,2);
    assert_eq!(i,1f32);
    assert_eq!(j,2f64);

    Ok(())
}

#[test]
pub fn test_buff()->Result<()>{
    let mut data=Data::new();
    let om=ObjectManager::new();
    om.write_(&mut data,&"123123")?;
    om.write_(&mut data,&vec![1u8,2,3,4,5])?;
    let x=[1u8,2,3,4,5];
    om.write_(&mut data,&&x[..])?;
    let mut dr=DataReader::from(&data[..]);
    let mut str:String="".to_string();
    let mut buff1:Vec<u8>=Default::default();
    let mut buff2:Vec<u8>=Default::default();
    om.read_(&mut dr,&mut str)?;
    om.read_(&mut dr,&mut buff1)?;
    om.read_(&mut dr,&mut buff2)?;

    assert_eq!(str,"123123");
    assert_eq!(buff1,vec![1,2,3,4,5]);
    assert_eq!(buff2,vec![1,2,3,4,5]);
    Ok(())
}


#[derive(Default,Eq, PartialEq,Debug,Clone)]
struct Foo{
    id:i32,
    name:String
}


impl ISerdeTypeId for Foo{
    #[inline(always)]
    fn type_id() -> u16 where Self: Sized {
       16
    }

    #[inline(always)]
    fn get_type_id(&self) -> u16 {
        Foo::type_id()
    }
}

impl ISerde for Foo{


    #[inline]
    fn write_to(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
        om.write_(data,&self.id)?;
        om.write_(data,&self.name)?;
        Ok(())
    }
    #[inline]
    fn read_from(&mut self, om: &ObjectManager, data:&mut DataReader)->Result<()> {
        om.read_(data, &mut self.id)?;
        om.read_(data, &mut self.name)?;
        Ok(())
    }
}

#[derive(Default,Eq, PartialEq,Debug,Clone)]
struct Foo2{
    base:Foo,
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


    #[inline]
    fn write_to(&self, om: &ObjectManager, data: &mut Data)->Result<()> {
        om.write_(data,&self.base)?;
        om.write_(data,&self.id)?;
        Ok(())
    }
    #[inline]
    fn read_from(&mut self, om: &ObjectManager, data: &mut DataReader)->Result<()> {
        om.read_(data,&mut self.base)?;
        om.read_(data, &mut self.id)?;
        Ok(())
    }
}


#[test]
pub fn test_struct()->Result<()>{
    let mut data=Data::new();
    let om=ObjectManager::new();
    let mut foo=Foo2::default();
    foo.id=1000;
    foo.base=Foo::default();
    foo.base.id=100;
    foo.base.name="123123".into();
    om.write_(&mut data,&foo)?;
    let mut dr=DataReader::from( &data[..]);
    let mut foo_clone=Foo2::default();
    om.read_(&mut dr,&mut foo_clone)?;

    assert_eq!(foo,foo_clone);
    Ok(())
}


pub fn test_collections()->Result<()>{
    {
        let mut data = Data::new();
        let om = ObjectManager::new();

        let mut foo = Foo2::default();
        foo.id = 1000;
        foo.base = Foo::default();
        foo.base.id = 100;
        foo.base.name = "123123".into();

        let p = vec![foo.clone(), foo.clone(), foo.clone(), foo];
        om.write_(&mut data, &p)?;
        let mut dr = DataReader::from(&data[..]);
        let mut b: Vec<Foo2> = Default::default();
        om.read_(&mut dr, &mut b)?;
        assert_eq!(p, b);

    }
    {
        let mut data = Data::new();
        let om = ObjectManager::new();

        let mut foo = Foo2::default();
        foo.id = 1000;
        foo.base = Foo::default();
        foo.base.id = 100;
        foo.base.name = "123123".into();

        let mut dict=HashMap::new();
        dict.insert(1,foo.clone());
        dict.insert(2,foo.clone());
        dict.insert(3,foo);

        om.write_(&mut data, &dict)?;

        let mut dr = DataReader::from(&data[..]);
        let mut b: HashMap<i32,Foo2> = Default::default();
        om.read_(&mut dr, &mut b)?;

        assert_eq!(dict, b);
    }
    {
        let mut data = Data::new();
        let om = ObjectManager::new();

        let mut foo = Foo2::default();
        foo.id = 1000;
        foo.base = Foo::default();
        foo.base.id = 100;
        foo.base.name = "123123".into();

        let mut dict=BTreeMap::new();
        dict.insert(1,foo.clone());
        dict.insert(2,foo.clone());
        dict.insert(3,foo);

        om.write_(&mut data, &dict)?;

        let mut dr = DataReader::from(&data[..]);
        let mut b: BTreeMap<i32,Foo2> = Default::default();
        om.read_(&mut dr, &mut b)?;

        assert_eq!(dict, b);
    }
    {
        let mut data = Data::new();
        let om = ObjectManager::new();


        let mut dict=HashSet::new();
        dict.insert(1);
        dict.insert(2);
        dict.insert(3);

        om.write_(&mut data, &dict)?;

        let mut dr = DataReader::from(&data[..]);
        let mut b: HashSet<i32> = Default::default();
        om.read_(&mut dr, &mut b)?;

        assert_eq!(dict, b);
    }
    {
        let mut data = Data::new();
        let om = ObjectManager::new();


        let mut dict=BTreeSet::new();
        dict.insert(1);
        dict.insert(2);
        dict.insert(3);

        om.write_(&mut data, &dict)?;

        let mut dr = DataReader::from(&data[..]);
        let mut b: BTreeSet<i32> = Default::default();
        om.read_(&mut dr, &mut b)?;

        assert_eq!(dict, b);
    }
    Ok(())
}


