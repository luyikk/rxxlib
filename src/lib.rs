pub mod data;
pub mod data_read;
pub mod manager;
pub mod types;

pub use crate::data::IData;
pub use crate::data_read::IDataRead;

use bytes::{BytesMut, BufMut, Bytes, Buf};
use paste::*;
use anyhow::*;

pub trait FixedNumber{
    /// 写入到ByteMut
    fn write(self,bytes:&mut BytesMut);
    /// 写入到ByteMut指定位置,Debug 有越界判断
    fn write_at(self,idx:usize,bytes:&mut BytesMut);
    /// 读取定长数据
    fn read(bytes:&mut Bytes)->Result<Self> where Self: Sized;
}
pub trait VarNumber{
    /// 写入到ByteMut 变长
    fn write_var(self,byte:&mut BytesMut);
    /// 读取变长数字
    fn read_var(bytes:&mut Bytes)->Result<Self> where Self: Sized;
}
impl  FixedNumber for bool{
    #[inline]
    fn write(self, bytes: &mut BytesMut) {
        bytes.put_u8(if self{0}else{1});
    }
    #[inline]
    fn write_at(self, idx: usize, bytes: &mut BytesMut) {
        debug_assert!(idx+std::mem::size_of::<Self>() <= bytes.len(), "idx too max {}>{}",idx+std::mem::size_of::<Self>(),bytes.len());
        let len=bytes.len();
        unsafe{
            bytes.set_len(idx);
            bytes.put_u8(if self{0}else{1});
            bytes.set_len(len);
        }
    }
    #[inline]
    fn read(bytes: &mut Bytes) -> Result<Self> {
        ensure!(bytes.len()>std::mem::size_of::<Self>());
        if bytes.get_u8()==0{
            Ok(true)
        }else{
            Ok(false)
        }
    }
}
/// 用于实现基本数字类型定长写入宏
macro_rules! impl_fixed_number_by_one_size {
    ($type:tt) => (
    paste! {
    impl FixedNumber for $type{
        #[inline]
        fn write(self, bytes: &mut BytesMut) {
            bytes.[<put_ $type>](self);
        }
        #[inline]
        fn write_at(self,idx:usize, bytes: &mut BytesMut) {
            debug_assert!(idx+std::mem::size_of::<Self>() <= bytes.len(), "idx too max {}>{}",idx+std::mem::size_of::<Self>(),bytes.len());
            let len=bytes.len();
            unsafe{
                bytes.set_len(idx);
                bytes.[<put_ $type>](self);
                bytes.set_len(len);
            }
        }
        #[inline]
        fn read(bytes: &mut Bytes) -> Result<Self> {
            ensure!(bytes.len()>std::mem::size_of::<Self>());
            Ok(bytes.[<get_ $type>]())
        }
    }
    });
}

impl_fixed_number_by_one_size!(u8);
impl_fixed_number_by_one_size!(i8);
macro_rules! impl_fixed_number_le {
    ($type:tt) => (
    paste! {
    #[cfg(not(feature = "BigEndian"))]
    impl FixedNumber for $type{
        #[inline]
        fn write(self, bytes: &mut BytesMut) {
            bytes.[<put_ $type _le>](self);
        }
        #[inline]
        fn write_at(self, idx:usize, bytes: &mut BytesMut) {
            debug_assert!(idx+std::mem::size_of::<Self>() <= bytes.len(), "idx too max {}>{}",idx+std::mem::size_of::<Self>(),bytes.len());
            let len=bytes.len();
            unsafe{
                bytes.set_len(idx);
                bytes.[<put_ $type _le>](self);
                bytes.set_len(len);
            }
        }
        #[inline]
        fn read(bytes: &mut Bytes) -> Result<Self> {
            ensure!(bytes.len()>std::mem::size_of::<Self>());
            Ok(bytes.[<get_ $type _le>]())
        }
    }

    #[cfg(feature = "BigEndian")]
    impl FixedNumber for $type{
        #[inline]
        fn write(self, bytes: &mut BytesMut) {
            bytes.[<put_ $type>](self);
        }

        #[inline]
        fn write_at(self, idx:usize, bytes: &mut BytesMut) {
            debug_assert!(idx+std::mem::size_of::<Self>() <= bytes.len(), "idx too max {}>{}",idx+std::mem::size_of::<Self>(),bytes.len());
            let len=bytes.len();
            unsafe{
                bytes.set_len(idx);
                bytes.[<put_ $type>](self);
                bytes.set_len(len);
            }
        }
        #[inline]
        fn read(bytes: &mut Bytes) -> Result<Self> {
            ensure!(bytes.len()>std::mem::size_of::<Self>());
            Ok(bytes.[<get_ $type>]())
        }
    }
    });
}
impl_fixed_number_le! (u16);
impl_fixed_number_le! (i16);
impl_fixed_number_le! (i32);
impl_fixed_number_le! (u32);
impl_fixed_number_le! (i64);
impl_fixed_number_le! (u64);
impl_fixed_number_le! (f32);
impl_fixed_number_le! (f64);
impl VarNumber for u64{
    #[inline]
    fn write_var(mut self, byte: &mut BytesMut) {
        while self >= 1 << 7 {
            byte.put_u8((self & 0x7f | 0x80) as u8);
            self = self >> 7;
        }
        byte.put_u8(self as u8);
    }
    #[inline]
    fn read_var(bytes: &mut Bytes) -> Result<Self> where Self: Sized {
        let mut v=0u64;
        let mut offset = 0;
        let mut shift = 0;

        while shift < 8 * 8 {
            ensure!(offset < bytes.len(),"read var number,offset:{} > bytes length:{}",offset,bytes.len());
            let b = bytes[offset];
            offset += 1;
            v |= ((b & 0x7F) as u64) << shift;
            if b & 0x80 == 0 {
                bytes.advance(offset);
                return Ok(v);
            }
            shift += 7;
        }
        bail!("not read var number too end")
    }
}
impl VarNumber for i64{
    #[inline]
    fn write_var(self, byte: &mut BytesMut) {
        let mut v=zig_zag_encode_u64(self);
        while v >= 1 << 7 {
            byte.put_u8((v & 0x7f | 0x80) as u8);
            v = v >> 7;
        }
        byte.put_u8(v as u8);
    }
    #[inline]
    fn read_var(bytes: &mut Bytes) -> Result<Self> where Self: Sized {
        Ok(zig_zag_decode_i64(u64::read_var(bytes)?))
    }
}
macro_rules! impl_var_number {
    (@u $type:tt) => (
    impl VarNumber for $type{
        #[inline]
        fn write_var(self, byte: &mut BytesMut) {
            let v=self as u64;
            v.write_var(byte);
        }
        #[inline]
        fn read_var(bytes: &mut Bytes) -> Result<Self> where Self: Sized {
            Ok(u64::read_var(bytes)? as Self)
        }
    });
    (@i $type:tt) => (
    impl VarNumber for $type{
        #[inline]
        fn write_var(self, byte: &mut BytesMut) {
            let v=self as i64;
            v.write_var(byte);
        }
        #[inline]
        fn read_var(bytes: &mut Bytes) -> Result<Self> where Self: Sized {
            Ok(i64::read_var(bytes)? as Self)
        }
    })
}
impl_var_number!(@u u16);
impl_var_number!(@i i16);
impl_var_number!(@u u32);
impl_var_number!(@i i32);
impl VarNumber for Vec<u8>{
    #[inline]
    fn write_var(self, byte: &mut BytesMut) {
        byte.write_var_integer(self.len() as u64);
        byte.write_buf(&self);
    }
    #[inline]
    fn read_var(bytes: &mut Bytes) -> Result<Self> where Self: Sized {
        let len=bytes.read_var_integer::<u64>()? as usize;
        let mut buff=vec![0;len];
        bytes.read_buf(&mut buff)?;
        Ok(buff)
    }
}
impl VarNumber for String{
    #[inline]
    fn write_var(self, byte: &mut BytesMut) {
        let buff=self.into_bytes();
        buff.write_var(byte);
    }
    #[inline]
    fn read_var(bytes: &mut Bytes) -> Result<Self> where Self: Sized {
        Ok(String::from_utf8( bytes.read_var_integer::<Vec<u8>>()?)?)
    }
}
impl VarNumber for &str{
    #[inline]
    fn write_var(self, byte: &mut BytesMut) {
        let buff=self.as_bytes();
        byte.write_var_integer(buff.len() as u64);
        byte.write_buf(buff);
    }
    #[inline]
    fn read_var(_bytes: &mut Bytes) -> Result<Self> where Self: Sized {
        panic!("not read &str,because bytes will be released")
    }
}
#[inline]
fn zig_zag_encode_u64(v: i64) -> u64 {
    ((v << 1) ^ (v >> 63)) as u64
}
#[inline]
fn zig_zag_decode_i64(v: u64) -> i64 {
    ((v >> 1) as i64) ^ (-((v & 1) as i64))
}