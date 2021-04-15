#[allow(unused_imports)]
#[allow(dead_code)]
#[allow(non_snake_case)]

pub mod PKG{
    use xxlib::*;
    use xxlib_builder::*;

    pub mod P{
        use xxlib::*;
        use xxlib_builder::*;

        #[derive(build, Debug)]
        #[cmd(typeid(11),compatible(false))]
        pub struct Point2{
            x:f32,
            y:f32
        }
        #[derive(build, Debug)]
        #[cmd(typeid(13),compatible(true))]
        pub struct Point3{
            base:Point2,
            z:f32
        }
    }
    #[derive(build, Debug)]
    #[cmd(typeid(11))]
    pub struct Base{
        pub S1:i32,
        pub S2:String,
        pub sp1:self::P::Point3,
        pub sp2:Option<self::P::Point3>,
        pub px:Option<i32>,
        pub Point2List:Vec<self::P::Point3>
    }
}

