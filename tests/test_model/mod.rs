pub mod master;
pub use master::*;


use xxlib::*;
use xxlib_builder::*;

/// Test Struct
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(compatible(False))]
pub struct PKG_TestStruct{

    /// test enum
    #[cmd(default(PKG_Enum_TypeId::a))]
    pub P:PKG_Enum_TypeId,

    /// test i32
    #[cmd(default(55))]
    pub a:i32,

    #[cmd(default(55))]
    pub b:i32,

    /// test f32
    #[cmd(default(5.0))]
    pub c:f32,

    /// test f64
    #[cmd(default(100))]
    pub x:f64,

    /// test string
    #[cmd(default("123123"))]
    pub sb:String,
}

/// Test Enum
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[build_enum(i32)]
pub enum PKG_Enum_TypeId{
    /// A
    a = 1,
    /// B
    b = 2,
    /// C
    c = 3,
}