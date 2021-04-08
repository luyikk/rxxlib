use anyhow::*;
use std::ops::Deref;
use std::mem::size_of;
use std::convert::TryInto;

pub trait RNumberFixed {
    fn read(dr:&mut DataReader)->Result<Self> where Self:Sized;
}

pub trait RNumberVar{
    fn read(dr:&mut DataReader)->Result<Self> where Self:Sized;
}

macro_rules! impl_read_number_fixed {
    ($type:tt) => (
    impl RNumberFixed for $type{
        #[cfg(not(feature = "BigEndian"))]
        #[inline]
        fn read(dr: &mut DataReader) -> Result<Self> where Self: Sized {
            let size = size_of::<$type>();
            ensure!(size<=dr.len(),"read fixed error len too min:dr:{} < {}",dr.len(),size);
            let v=$type::from_le_bytes(dr[..size].try_into()?);
            dr.advance(size)?;
            Ok(v)
        }

        #[cfg(feature = "BigEndian")]
        #[inline]
        fn read(dr: &mut DataReader) -> Result<Self> where Self: Sized {
            let size = size_of::<$type>();
            ensure!(size<=dr.len(),"read fixed error len too min:dr:{} < {}",dr.len(),size);
            let v=$type::from_be_bytes(dr[..size].try_into()?);
            dr.advance(size)?;
            Ok(v)
        }
    });
}

impl_read_number_fixed!(u8);
impl_read_number_fixed!(i8);
impl_read_number_fixed!(u16);
impl_read_number_fixed!(i16);
impl_read_number_fixed!(u32);
impl_read_number_fixed!(i32);
impl_read_number_fixed!(u64);
impl_read_number_fixed!(i64);
impl_read_number_fixed!(f32);
impl_read_number_fixed!(f64);

impl RNumberVar for u16{
    #[inline]
    fn read(dr: &mut DataReader) -> Result<Self> where Self: Sized {
        let mut v=0u16;
        let mut offset = 0;
        let mut shift = 0u8;
        let mut b;
        while shift < 16 {
            ensure!(offset != dr.len(),"read var number,offset:{} > bytes length:{}",offset,dr.len());
            b = dr[offset];
            offset += 1;
            v |= ((b & 0x7F) as u16) << shift;
            if b & 0x80 == 0 {
                dr.buff=&dr.buff[offset..];
                return Ok(v);
            }
            shift += 7;
        }
        bail!("not read var number too end")
    }
}

impl RNumberVar for i16{
    #[inline]
    fn read(dr: &mut DataReader) -> Result<Self> where Self: Sized {
        Ok(zig_zag_decode_i16(RNumberVar::read(dr)?))
    }
}

impl RNumberVar for u32{
    #[inline]
    fn read(dr: &mut DataReader) -> Result<Self> where Self: Sized {
        let mut v=0u32;
        let mut offset = 0;
        let mut shift = 0u8;
        let mut b;
        while shift < 32 {
            ensure!(offset != dr.len(),"read var number,offset:{} > bytes length:{}",offset,dr.len());
            b = dr[offset];
            offset += 1;
            v |= ((b & 0x7F) as u32) << shift;
            if b & 0x80 == 0 {
                dr.buff=&dr.buff[offset..];
                return Ok(v);
            }
            shift += 7;
        }
        bail!("not read var number too end")
    }
}

impl RNumberVar for i32{
    #[inline]
    fn read(dr: &mut DataReader) -> Result<Self> where Self: Sized {
        Ok(zig_zag_decode_i32(RNumberVar::read(dr)?))
    }
}

impl RNumberVar for u64{
    #[inline]
    fn read(dr: &mut DataReader) -> Result<Self> where Self: Sized {
        let mut v=0u64;
        let mut offset = 0;
        let mut shift = 0u8;
        let mut b;
        while shift < 64 {
            ensure!(offset != dr.len(),"read var number,offset:{} > bytes length:{}",offset,dr.len());
            b = dr[offset];
            offset += 1;
            v |= ((b & 0x7F) as u64) << shift;
            if b & 0x80 == 0 {
                dr.buff=&dr.buff[offset..];
                return Ok(v);
            }
            shift += 7;
        }
        bail!("not read var number too end")
    }
}

impl RNumberVar for i64{
    #[inline]
    fn read(dr: &mut DataReader) -> Result<Self> where Self: Sized {
        Ok(zig_zag_decode_i64(RNumberVar::read(dr)?))
    }
}

impl RNumberVar for String{
    #[inline]
    fn read(dr: &mut DataReader) -> Result<Self> where Self: Sized {
        let len:u64= dr.read_var_integer()?;
        let mut buff=vec![0;len as usize];
        dr.read_buff(&mut buff[..])?;
        Ok(String::from_utf8(buff)?)
    }
}

#[inline]
fn zig_zag_decode_i16(v: u16) -> i16 {
    ((v >> 1) as i16) ^ (-((v & 1) as i16))
}
#[inline]
fn zig_zag_decode_i32(v: u32) -> i32 {
    ((v >> 1) as i32) ^ (-((v & 1) as i32))
}
#[inline]
fn zig_zag_decode_i64(v: u64) -> i64 {
    ((v >> 1) as i64) ^ (-((v & 1) as i64))
}

#[derive(Debug)]
pub struct DataReader<'a>{
    buff:&'a [u8],
    original_len:usize
}

impl<'a> From<&'a [u8]> for DataReader<'a>{
    fn from(buff: &'a [u8]) -> Self {
        DataReader{
            buff,
            original_len:buff.len()
        }
    }
}

impl<'a> Deref for DataReader<'a>{
    type Target = &'a [u8];
    fn deref(&self) -> &Self::Target {
        &self.buff
    }
}


impl <'a> DataReader<'a>{
    #[inline]
    pub fn advance(&mut self,cnt: usize)->Result<()>{
        ensure!(self.len() >= cnt,"advance error,cnt:{} > len:{}",cnt,self.len());
        self.buff=&self.buff[cnt..];
        Ok(())
    }

    #[inline]
    pub fn offset(&self)->usize{
        self.original_len.wrapping_sub(self.buff.len())
    }

    #[inline]
    pub fn read_buff(&mut self,buff:&mut [u8])->Result<()>{
        let size=buff.len();
        ensure!(self.len() >=size,"read buff,buff too max,current:{} input:{}",self.len(),size);
        let (copy,current)=self.buff.split_at(size);
        buff.copy_from_slice(copy);
        self.buff=current;
        Ok(())
    }

    #[inline]
    pub fn read_fixed<T:RNumberFixed>(&mut self)->Result<T>{
        T::read(self)
    }

    #[inline]
    pub fn read_var_integer<T:RNumberVar>(&mut self)->Result<T>{
        T::read(self)
    }

}


