pub mod test {
    use xxlib_builder::*;
    use xxlib::*;

    #[derive(build, Debug)]
    #[cmd(typeid(106), compatible(true))]
    pub struct Foo {
        #[cmd(default(10))]
        pub id: i32,
        #[cmd(default("123123"))]
        pub name: String,
        pub child: Weak<Foo2>
    }

    #[derive(build, Debug)]
    #[cmd(typeid(107))]
    pub struct Foo2 {
        pub base: SharedPtr<Foo>,
        #[cmd(default(100))]
        pub id: u64
    }


    pub fn register_pkg_objs() {
        xxlib::ObjectManager::register::<Foo>(stringify!(Foo));
        xxlib::ObjectManager::register::<Foo2>(stringify!(Foo2));
    }
}

use xxlib::*;
use test::*;
use sharedptr::unsafe_def::IGetMutUnchecked;

#[test]
pub fn test()->anyhow::Result<()>{
    register_pkg_objs();

    let mut foo=Foo::default();
    foo.name="123123".to_string();
    foo.id=100;

    let mut foo2=Foo2::default();
    foo2.id=1000;
    foo2.base=SharedPtr::new(foo);

    let foo2_ptr=SharedPtr::new(foo2);
    unsafe {
        foo2_ptr.base.get_mut_unchecked().child = foo2_ptr.weak().ok_or_else(|| anyhow::anyhow!("is none"))?;
    }

    let mut data=Data::new();
    let om=ObjectManager::new();

    om.write_to(&mut data,&foo2_ptr)?;
    let mut dr=DataReader::from( &data[..]);
    let ptr= om.read_ptr(&mut dr)?.cast::<Foo2>()?;

    assert_eq!(FOO2_ID,ptr.get_type_id());
    assert_eq!(foo2_ptr.id,ptr.id);
    assert_eq!(foo2_ptr.base.id,ptr.base.id);
    assert_eq!(foo2_ptr.base.name,ptr.base.name);
    assert_eq!(foo2_ptr.id,ptr.base.child.upgrade().ok_or_else(||anyhow::anyhow!("none"))?.id);

    let x= filter_ids("Foo");
    assert_eq!(x,&[106,107]);
    Ok(())
}