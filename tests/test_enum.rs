use anyhow::*;
use xxlib::*;
use xxlib_builder::*;


#[build_enum(i32)]
pub enum Flags{
    N=0,
    A=1,
    B=2,
    C=3
}

#[build_enum(u64)]
pub enum Flags2{
    N=0,
    A=1,
    B=2,
    C=3
}


#[derive(build,Debug)]
#[cmd(typeid(106),compatible(true))]
pub struct Foo{
    #[cmd(default(Flags::A))]
    id:Flags,
    #[cmd(default("123123"))]
    name:String,
    child:std::rc::Weak<Foo2>
}

#[derive(build,Debug)]
#[cmd(typeid(107))]
struct Foo2{
    base:sharedptr::Rc::SharedPtr<Foo>,
    #[cmd(default(Flags2::B))]
    id:Flags2
}

pub fn register_pkg_objs(){
    xxlib::ObjectManager::register::<Foo>();
    xxlib::ObjectManager::register::<Foo2>();
}


#[test]
pub fn test()->Result<()>{
    use sharedptr::Rc::SharedPtr;
    use xxlib::*;

    register_pkg_objs();

    let mut foo=Foo::default();
    foo.name="123123".to_string();

    let mut foo2=Foo2::default();
    foo2.base=SharedPtr::new(foo);


    let foo2_ptr=SharedPtr::new(foo2);
    unsafe {
        foo2_ptr.base.get_mut_ref().child = foo2_ptr.weak().ok_or_else(|| anyhow::anyhow!("is none"))?;
    }

    let mut data=Data::new();
    let om=ObjectManager::new();

    om.write_to(&mut data,&foo2_ptr)?;
    let mut dr=DataReader::from( &data[..]);
    let ptr= om.read_ptr(&mut dr)?.cast::<Foo2>()?;

    assert_eq!(foo2_ptr.id,ptr.id);
    assert_eq!(foo2_ptr.base.id,ptr.base.id);
    assert_eq!(foo2_ptr.base.name,ptr.base.name);
    assert_eq!(foo2_ptr.id,ptr.base.child.upgrade().ok_or_else(||anyhow::anyhow!("none"))?.id);


    Ok(())
}