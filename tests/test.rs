use anyhow::Result;
use xxlib::data::Data;
use xxlib::data_read::DataReader;

#[test]
fn test_write()->Result<()>{
    let mut data=Data::new();
    data.write_buf(&[1,2,3,4]);
    assert_eq!(&data[..],&[1,2,3,4]);

    data.write_buf_at(1,&[1,1,1])?;
    assert_eq!(&data[..],&[1,1,1,1]);

    data.write_fixed(1u8);
    assert_eq!(&data[..],&[1,1,1,1,1]);

    data.write_fixed(1i8);
    assert_eq!(&data[..],&[1,1,1,1,1,1]);

    data.write_fixed(1u16);
    assert_eq!(&data[..],&[1,1,1,1,1,1,1,0]);

    data.write_fixed_at(1,1i32)?;
    assert_eq!(&data[..],&[1,1,0,0,0,1,1,0]);

    let mut data=Data::new();
    data.write_var_integer(1u32);
    assert_eq!(&data[..],&[1]);

    data.write_var_integer(1i32);
    assert_eq!(&data[..],&[1,2]);

    data.write_fixed(1f32);

    let mut data=Data::new();
    data.write_var_integer("123123");
    assert_eq!(&data[..],&[6, 49, 50, 51, 49, 50, 51]);

    let mut data=Data::new();
    data.write_var_integer("123123".to_string());
    assert_eq!(&data[..],&[6, 49, 50, 51, 49, 50, 51]);

    let mut data=Data::new();
    data.write_fixed(1u8);
    data.write_fixed(2i16);
    data.write_fixed(3i32);
    data.write_fixed(4i64);
    data.write_fixed(5f32);
    data.write_fixed(6f64);

    let mut rd=DataReader::from(&data[..]);
    assert_eq!(1,rd.read_fixed::<u8>()?);
    assert_eq!(2,rd.read_fixed::<i16>()?);
    assert_eq!(3,rd.read_fixed::<i32>()?);
    assert_eq!(4,rd.read_fixed::<i64>()?);
    assert_eq!(5f32,rd.read_fixed::<f32>()?);
    assert_eq!(6f64,rd.read_fixed::<f64>()?);

    Ok(())
}

#[test]
fn test_read()->Result<()>{
    let x=[1,2,3,4,5,6];
    let mut data=DataReader::from(&x[..]);
    let mut read =vec![0; 2];
    data.read_buff(&mut read)?;
    assert_eq!(read,[1,2]);

    let mut data=Data::new();
    data.write_var_integer(123u32);
    data.write_var_integer(321i32);
    data.write_var_integer(123u64);
    data.write_var_integer(321i64);

    let mut data=DataReader::from(&data[..]);
    let x:u32=data.read_var_integer()?;
    assert_eq!(123,x);
    let x:i32=data.read_var_integer()?;
    assert_eq!(321,x);
    let x:u64=data.read_var_integer()?;
    assert_eq!(123,x);
    let x:i64=data.read_var_integer()?;
    assert_eq!(321,x);

    let mut data=Data::new();
    data.write_var_integer("hello world");
    let mut data=DataReader::from(&data[..]);
    let msg:String=data.read_var_integer()?;
    assert_eq!(msg,"hello world");
    Ok(())
}