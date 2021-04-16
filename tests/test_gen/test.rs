use xxlib::*;
use xxlib_builder::*;
use super::ref_class::*;

#[allow(dead_code)]
const  MD5:&'static str="#*MD5<67e83101f793ed7fe8ac96fe32ee91b9>*#";

#[allow(dead_code,non_snake_case)]
pub fn CodeGen_Test(){
    ObjectManager::register::<PKG_TestBase>();
    ObjectManager::register::<PKG_TestStruct2>();
    ObjectManager::register::<PKG_Base>();
    ObjectManager::register::<PKG_P_Point>();
    ObjectManager::register::<PKG_Foo>();
}

/// Test Struct
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(typeid(12),compatible(true))]
pub struct PKG_TestBase{
    #[cmd(default(1.0))]
    pub x:f32,
    #[cmd(default(2.0))]
    pub y:f32,
}

/// Test Struct
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(typeid(11),compatible(false))]
pub struct PKG_TestStruct2{
    /// Parent Class
    pub base:PKG_TestBase,
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
    #[cmd(default(100.0))]
    pub x:f64,
    /// test string
    #[cmd(default("123123"))]
    pub sb:String,
    /// test buff
    pub buff:Vec<u8>,
    /// test btreemap
    pub table:std::collections::BTreeMap<i32, String>,
    /// test hashmap
    pub hashtable:std::collections::HashMap<i32, String>,
    /// test hashset
    pub hashset:std::collections::HashSet<i32>,
    /// test weak my
    pub weak_my:Weak<PKG_TestStruct2>,
    /// test sharedptr my
    pub shard_my:SharedPtr<PKG_TestStruct2>,
}

/// Base
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(typeid(121),compatible(false))]
pub struct PKG_Base{
    /// S1
    #[cmd(default(0))]
    pub S1:i32,
    /// S2
    pub S2:String,
    pub sp1:PKG_P_Point3,
    pub sp2:Option<PKG_P_Point3>,
    pub sp3:Option<PKG_P_Point3>,
    pub px:Option<i32>,
    pub Point2List:Vec<PKG_P_Point3>,
}

/// Ponit
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(typeid(122),compatible(true))]
pub struct PKG_P_Point{
    #[cmd(default(0))]
    pub X:i32,
    #[cmd(default(0))]
    pub Y:i32,
    pub Z:Option<f64>,
}

/// Foo
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(typeid(123),compatible(true))]
pub struct PKG_Foo{
    /// Parent Class
    pub base:PKG_Base,
    #[cmd(default(0))]
    pub P1:i32,
    #[cmd(default(0.0))]
    pub P2:f32,
    pub P3:String,
    pub Buff:Vec<u8>,
    pub Data:Vec<u32>,
    pub Position:SharedPtr<PKG_P_Point>,
    pub Position2:SharedPtr<PKG_P_Point>,
    pub My:SharedPtr<PKG_Foo>,
    pub Positions:Vec<SharedPtr<PKG_P_Point>>,
}

/// Ponit2
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(compatible(false))]
pub struct PKG_P_Point2{
    #[cmd(default(0.0))]
    pub x:f32,
    #[cmd(default(0.0))]
    pub y:f32,
}

/// Ponit3
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(compatible(true))]
pub struct PKG_P_Point3{
    /// Parent Class
    pub base:PKG_P_Point2,
    #[cmd(default(0.0))]
    pub z:f32,
}

/// Test Struct
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(compatible(false))]
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
    #[cmd(default(100.0))]
    pub x:f64,
    /// test string
    #[cmd(default("123123"))]
    pub sb:String,
    /// test buff
    pub buff:Vec<u8>,
    /// test btreemap
    pub table:std::collections::BTreeMap<i32, String>,
    /// test hashmap
    pub hashtable:std::collections::HashMap<i32, String>,
    /// test hashset
    pub hashset:std::collections::HashSet<i32>,
}

/// Player
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(compatible(true))]
pub struct PKG_P_Player{
    /// Parent Class
    pub base:PKG_P_Point2,
    pub position:PKG_P_Point3,
    pub position2:Option<PKG_P_Point3>,
    pub px:Option<i32>,
    pub Point2List:Vec<PKG_P_Point2>,
}

/// Test Struct
#[allow(unused_imports,dead_code,non_snake_case,non_camel_case_types)]
#[derive(build,Debug)]
#[cmd(compatible(true))]
pub struct PKG_AA_Test_B{
    /// Parent Class
    pub base:PKG_TestStruct,
    /// test enum
    #[cmd(default(PKG_Enum_TypeId::b))]
    pub P:PKG_Enum_TypeId,
    /// test sharedptr my
    pub ptr:SharedPtr<PKG_TestStruct2>,
    /// test sharedptr xy
    pub shard_xy:SharedPtr<PKG_TestBase>,
    /// test ref
    pub ref_name:SharedPtr<PKG_Ref_TypeName>,
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