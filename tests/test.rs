use bytes::{BytesMut, Buf, Bytes};
use xxlib::data::IData;
use xxlib::data_read::IDataRead;
use anyhow::Result;

#[test]
fn test_write(){
    let mut data=BytesMut::new();
    data.write_buf(&[1,2,3,4]);
    assert_eq!(&data[..],&[1,2,3,4]);

    data.write_buf_at(1,&[1,1,1]);
    assert_eq!(&data[..],&[1,1,1,1]);

    data.write_fixed(1u8);
    assert_eq!(&data[..],&[1,1,1,1,1]);

    data.write_fixed(1i8);
    assert_eq!(&data[..],&[1,1,1,1,1,1]);

    data.write_fixed(1u16);
    assert_eq!(&data[..],&[1,1,1,1,1,1,1,0]);

    data.write_fixed_at(1,1i32);
    assert_eq!(&data[..],&[1,1,0,0,0,1,1,0]);

    let mut data=BytesMut::new();
    data.write_var_integer(1u32);
    assert_eq!(&data[..],&[1]);

    data.write_var_integer(1i32);
    assert_eq!(&data[..],&[1,2]);

    data.write_fixed(1f32);

    let mut data=BytesMut::new();
    data.write_var_integer("123123");
    assert_eq!(format!("{:?}",&data),"b\"\\x06123123\"");

    let mut data=BytesMut::new();
    data.write_var_integer("123123".to_string());
    assert_eq!(format!("{:?}",&data),"b\"\\x06123123\"");

    let mut data=BytesMut::new();
    data.write_fixed(1u8);
    data.write_fixed(2i16);
    data.write_fixed(3i32);
    data.write_fixed(4i64);
    data.write_fixed(5f32);
    data.write_fixed(6f64);

    assert_eq!(1,data.get_u8());
    assert_eq!(2,data.get_i16_le());
    assert_eq!(3,data.get_i32_le());
    assert_eq!(4,data.get_i64_le());
    assert_eq!(5f32,data.get_f32_le());
    assert_eq!(6f64,data.get_f64_le());

}

#[test]
fn test_read()->Result<()>{
    let mut data=Bytes::from(vec![1,2,3,4,5,6]);
    let mut read =vec![0; 2];
    data.read_buf(&mut read)?;
    assert_eq!(read,[1,2]);

    let mut data=BytesMut::new();
    data.write_var_integer(123u32);
    data.write_var_integer(321i32);
    data.write_var_integer(123u64);
    data.write_var_integer(321i64);

    let mut data=data.freeze();
    let x:u32=data.read_var_integer()?;
    assert_eq!(123,x);
    let x:i32=data.read_var_integer()?;
    assert_eq!(321,x);
    let x:u64=data.read_var_integer()?;
    assert_eq!(123,x);
    let x:i64=data.read_var_integer()?;
    assert_eq!(321,x);

    let mut data=BytesMut::new();
    data.write_var_integer(vec![1,2,3,4,5]);
    data.write_var_integer("hello world");
    let mut data=data.freeze();
    let buff= data.read_var_integer::<Vec<u8>>()?;
    assert_eq!(vec![1,2,3,4,5],buff);
    let msg:String=data.read_var_integer()?;
    assert_eq!(msg,"hello world");
    Ok(())



}