use xxlib::*;
use xxlib_builder::*;

#[allow(dead_code)]
const  MD5:&'static str="#*MD5<8ab14b0821f2ef39d8537dc0f53ccd7c>*#";

#[allow(dead_code,non_snake_case)]
pub fn CodeGen_Ref_class(){
    ObjectManager::register::<PKG_Ref_TypeName>();
}

/// Test Ref
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(typeid(1222),compatible(false))]
pub struct PKG_Ref_TypeName{
    #[cmd(default("null"))]
    pub name:String,
}