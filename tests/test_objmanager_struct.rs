use xxlib::*;
use xxlib_builder::*;
use anyhow::*;
use sharedptr::unsafe_def::IGetMutUnchecked;


#[derive(build,Debug,Clone)]
struct Foo{
    id:i32,
    name:String,
    child:Weak<Foo2>
}

#[derive(build,Debug,Clone)]
#[cmd(typeid(32))]
struct Foo2{
    base:Vec<Foo>,
    id:u64
}


#[test]
#[cfg_attr(miri, ignore)]
pub fn test()->Result<()>{
    ObjectManager::register::<Foo2>(stringify!(Foo2));
    let mut foo=Foo::default();
    foo.name="123123".to_string();
    foo.id=100;

    let mut foo2=Foo2::default();
    foo2.id=1000;
    foo2.base=vec![foo];

    let foo2_ptr =SharedPtr::<Foo2>::new(foo2);
    unsafe {
        foo2_ptr.get_mut_unchecked().base[0].child = foo2_ptr.weak().ok_or_else(|| anyhow!("is none"))?;
    }

    let mut data=Data::new();
    let om=ObjectManager::new();

    om.write_to(&mut data,&foo2_ptr)?;
    let mut dr=DataReader::from( &data[..]);
    let ptr= om.read_ptr(&mut dr)?.cast::<Foo2>()?;

    assert_eq!(foo2_ptr.id,ptr.id);
    assert_eq!(foo2_ptr.base[0].id,ptr.base[0].id);
    assert_eq!(foo2_ptr.base[0].name,ptr.base[0].name);
    assert_eq!(foo2_ptr.id,ptr.base[0].child.upgrade().ok_or_else(||anyhow!("none"))?.id);



    Ok(())
}