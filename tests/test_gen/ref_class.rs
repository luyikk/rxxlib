use xxlib::*;
use xxlib_builder::*;

#[allow(dead_code)]
const  MD5:&'static str="#*MD5<63df4158d595fbeb7d1151d535a36ef5>*#";

#[allow(dead_code,non_snake_case)]
pub fn CodeGen_Ref_class(){
    ObjectManager::register::<PKG_Ref_TypeName>(stringify!(PKG_Ref_TypeName));
}

/// Test Ref
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(typeid(1222),compatible(false))]
pub struct PKG_Ref_TypeName{
    #[cmd(default("null"))]
    pub name:String,
}