use anyhow::Result;
use data_rw::Data;
use data_rw::DataReader;

fn main()->Result<()> {
    let mut data = Data::new();
    data.write_fixed(&1i32);
    data.write_fixed(&2i64);
    data.write_fixed(&3f64);
    data.write_var_integer(&123u32);
    data.write_var_integer(&321i32);
    data.write_var_integer(&123u64);
    data.write_var_integer(&321i64);
    data.write_var_integer(&"hello world");


    let mut data = DataReader::from(&data[..]);
    let x: i32 = data.read_fixed()?;
    assert_eq!(1, x);
    let x: i64 = data.read_fixed()?;
    assert_eq!(2, x);
    let x: f64 = data.read_fixed()?;
    assert_eq!(3f64, x);
    let x: u32 = data.read_var_integer()?;
    assert_eq!(123, x);
    let x: i32 = data.read_var_integer()?;
    assert_eq!(321, x);
    let x: u64 = data.read_var_integer()?;
    assert_eq!(123, x);
    let x: i64 = data.read_var_integer()?;
    assert_eq!(321, x);
    let msg = data.read_var_str()?;
    assert_eq!(msg, "hello world");

    Ok(())
}