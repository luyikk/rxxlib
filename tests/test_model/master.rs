use xxlib::*;
use xxlib_builder::*;


/// Point
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(typeid(12),compatible(true))]
pub struct PKG_P_Point{
    #[cmd(default(1))]
    pub x:i32,
    #[cmd(default(2))]
    pub y:i32,
    #[cmd(default(Some(1.2)))]
    pub Z:Option<f64>
}

/// Point2
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
pub struct PKG_P_Point2{
    pub x:f32,
    pub y:f32
}
/// Point3
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(compatible(true))]
pub struct PKG_P_Point3{
    pub base:PKG_P_Point,
    pub z:f32
}
/// Player
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(compatible(true))]
pub struct PKG_P_Player{
    pub base:PKG_P_Point2,
    pub position2:Option<PKG_P_Point3>,
    pub px:Option<i32>,
    pub Point2List:Vec<PKG_P_Point2>
}
/// Base
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(typeid(12),compatible(false))]
pub struct PKG_Base{
    /// S1
    pub S1:i32,
    /// S2
    pub S2:String,
    pub sp1:PKG_P_Point3,
    pub sp2:Option<PKG_P_Point3>,
    pub sp3:Option<PKG_P_Point3>,
    pub px:Option<i32>,
    pub Point2List:Vec<PKG_P_Point3>
}
/// Foo
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(typeid(13),compatible(true))]
pub struct Foo{
    pub base:PKG_Base,
    pub P1:i32,
    pub P2:f32,
    pub P3:String,
    pub Buff:Vec<u8>,
    pub Data:Vec<u32>,
    pub Position:SharedPtr<PKG_P_Point>,
    pub Position2:SharedPtr<PKG_P_Point>,
    pub My:Weak<Foo>,
}

