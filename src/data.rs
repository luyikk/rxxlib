use anyhow::*;
use std::ops::{Deref, DerefMut};
use std::mem::size_of;

pub trait WNumberFixed {
    fn write(&self,data:&mut Data);
    fn write_at(&self,idx:usize,data:&mut Data)->Result<()>;
}

pub trait WNumberVar {
    fn write(&self,data:&mut Data);
}

#[derive(Debug,Default)]
pub struct Data{
    buff:Vec<u8>
}


macro_rules! impl_number_fixed {
    ($type:ty) => (
    impl WNumberFixed for $type{
        #[cfg(not(feature = "BigEndian"))]
        #[inline]
        fn write(&self, data: &mut Data) {
            unsafe{
                let size=size_of::<$type>();
                let len=data.check_reserve(size);
                std::ptr::copy_nonoverlapping(self.to_le_bytes().as_ptr(), data.as_mut_ptr().add(len),size);
                data.buff.set_len(len.wrapping_add(size));
            }
        }
        #[cfg(not(feature = "BigEndian"))]
         #[inline]
        fn write_at(&self, idx:usize,data: &mut Data)->Result<()>{
            unsafe{
                let size=size_of::<$type>();
                ensure!(idx.wrapping_add(size)<=data.len(),"idx too max {}>{}",idx.wrapping_add(size),data.len());
                std::ptr::copy_nonoverlapping(self.to_le_bytes().as_ptr(), data.as_mut_ptr().add(idx),size);
                Ok(())
            }
        }

        #[cfg(feature = "BigEndian")]
        #[inline]
        fn write(&self, data: &mut Data) {
            unsafe{
                let size=size_of::<$type>();
                let len=data.check_reserve(size);
                std::ptr::copy_nonoverlapping(self.to_be_bytes().as_ptr(), data.as_mut_ptr().add(len),size);
                data.buff.set_len(len.wrapping_add(size));
            }
        }
        #[cfg(feature = "BigEndian")]
         #[inline]
        fn write_at(&self, idx:usize,data: &mut Data)->Result<()>{
            unsafe{
                let size=size_of::<$type>();
                ensure!(idx.wrapping_add(size)<=data.len(),"idx too max {}>{}",idx.wrapping_add(size),data.len());
                std::ptr::copy_nonoverlapping(self.to_be_bytes().as_ptr(), data.as_mut_ptr().add(idx),size);
                Ok(())
            }
        }
    });
}

impl_number_fixed!(u8);
impl_number_fixed!(i8);
impl_number_fixed!(i16);
impl_number_fixed!(u16);
impl_number_fixed!(i32);
impl_number_fixed!(u32);
impl_number_fixed!(i64);
impl_number_fixed!(u64);
impl_number_fixed!(f32);
impl_number_fixed!(f64);

impl WNumberVar for u16{
    #[inline]
    fn write(&self, data: &mut Data) {
        let mut value=*self;
        let size=compute_raw_varint64_size(value as u64);
        let current_len=data.check_reserve(size);
        unsafe {
            let mut len: usize = 1;
            let mut ptr = data.as_mut_ptr().add(current_len);
            while value >= 1 << 7 {
                ptr.write((value & 0x7f | 0x80) as u8);
                ptr=ptr.offset(1);
                len +=1;
                value >>= 7;
            }
            ptr.write(value as u8);
            data.set_len(current_len+ len);
        }
    }
}
impl WNumberVar for i16{
    #[inline]
    fn write(&self, data: &mut Data) {
        WNumberVar::write(&zig_zag_encode_u16(self), data);
    }
}
impl WNumberVar for u32{
    #[inline]
    fn write(&self, data: &mut Data) {
        let mut value=*self;
        let size=compute_raw_varint32_size(value);
        let current_len=data.check_reserve(size);
        unsafe {
            let mut len: usize = 1;
            let mut ptr = data.as_mut_ptr().add(current_len);
            while value >= 1 << 7 {
                ptr.write((value & 0x7f | 0x80) as u8);
                ptr=ptr.offset(1);
                len +=1;
                value >>= 7;
            }
            ptr.write(value as u8);
            data.set_len(current_len+ len);
        }
    }
}
impl WNumberVar for i32{
    #[inline]
    fn write(&self, data: &mut Data) {
        WNumberVar::write(&zig_zag_encode_u32(self), data);
    }
}
impl WNumberVar for u64{
    #[inline]
    fn write(&self, data: &mut Data) {
        let mut value=*self;
        let size=compute_raw_varint64_size(value);
        let current_len=data.check_reserve(size);
        unsafe {
            let mut len: usize = 1;
            let mut ptr = data.as_mut_ptr().add(current_len);
            while value >= 1 << 7 {
                ptr.write((value & 0x7f | 0x80) as u8);
                ptr=ptr.offset(1);
                len +=1;
                value >>= 7;
            }
            ptr.write(value as u8);
            data.set_len(current_len+ len);
        }
    }
}
impl WNumberVar for i64{
    #[inline]
    fn write(&self, data: &mut Data) {
        WNumberVar::write(&zig_zag_encode_u64(self), data);
    }
}

#[inline(always)]
fn zig_zag_encode_u16(v: &i16) -> u16 {
    ((v << 1) ^ (v >> 15)) as u16
}
#[inline(always)]
fn zig_zag_encode_u32(v: &i32) -> u32 {
    ((v << 1) ^ (v >> 31)) as u32
}
#[inline(always)]
fn zig_zag_encode_u64(v: &i64) -> u64 {
    ((v << 1) ^ (v >> 63)) as u64
}

impl WNumberVar for String{
    #[inline]
    fn write(&self, data: &mut Data) {
        let buff=self.as_bytes();
        data.write_var_integer(&(buff.len() as u64));
        data.write_buf(&buff);
    }
}
impl WNumberVar for &str{
    #[inline]
    fn write(&self, data: &mut Data) {
        let buff=self.as_bytes();
        data.write_var_integer(&(buff.len() as u64));
        data.write_buf(&buff);
    }
}
impl WNumberFixed for bool{
    #[inline]
    fn write(&self, data: &mut Data) {
        let v=if *self {1u8}else{0u8};
        data.write_fixed(&v);
    }

    fn write_at(&self, idx: usize, data: &mut Data) -> Result<()> {
        let v=if *self {1u8}else{0u8};
        data.write_fixed_at(idx,v)
    }
}

impl Data{
    pub fn new()->Self{
        Data{
            buff:Vec::with_capacity(4096)
        }
    }

    pub fn with_capacity(cap:usize)->Data{
        Data{
            buff:Vec::with_capacity(cap)
        }
    }

    #[inline(always)]
    pub fn write_buf(&mut self,buff:&[u8]){
        unsafe{
            let size=buff.len();
            let len=self.check_reserve(size);
            std::ptr::copy_nonoverlapping(buff.as_ptr(), self.as_mut_ptr().add(len),size);
            self.set_len(len.wrapping_add(size));
        }
    }

    #[inline(always)]
    pub fn write_buf_at(&mut self,idx:usize,buff:&[u8])->Result<()>{
        let size=buff.len();
        ensure!(idx.wrapping_add(size)<=self.len(),"idx too max {}>{}",idx.wrapping_add(size),self.len());
        unsafe{
            std::ptr::copy_nonoverlapping(buff.as_ptr(), self.as_mut_ptr().add(idx),size);
        }
        Ok(())
    }

    #[inline(always)]
    pub fn write_fixed(&mut self,v:&impl WNumberFixed){
        v.write(self)
    }

    #[inline(always)]
    pub fn write_fixed_at(&mut self, idx:usize, v:impl WNumberFixed) ->Result<()>{
        v.write_at(idx,self)
    }

    #[inline(always)]
    pub fn write_var_integer(&mut self,v:&impl WNumberVar){
        v.write(self);
    }

    #[inline(always)]
    pub fn check_reserve(&mut self, size:usize) ->usize{
        let len=self.len();
        if len>self.capacity().wrapping_sub(size){
            self.reserve(len*2);
        }
        len
    }

}

impl Deref for Data{
    type Target = Vec<u8>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.buff
    }
}

impl DerefMut for Data{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buff
    }
}


/// Given `u64` value compute varint encoded length.
#[inline(always)]
pub fn compute_raw_varint64_size(value: u64) -> usize {
    if (value & (0xffffffffffffffffu64 << 7)) == 0 {
        return 1;
    }
    if (value & (0xffffffffffffffffu64 << 14)) == 0 {
        return 2;
    }
    if (value & (0xffffffffffffffffu64 << 21)) == 0 {
        return 3;
    }
    if (value & (0xffffffffffffffffu64 << 28)) == 0 {
        return 4;
    }
    if (value & (0xffffffffffffffffu64 << 35)) == 0 {
        return 5;
    }
    if (value & (0xffffffffffffffffu64 << 42)) == 0 {
        return 6;
    }
    if (value & (0xffffffffffffffffu64 << 49)) == 0 {
        return 7;
    }
    if (value & (0xffffffffffffffffu64 << 56)) == 0 {
        return 8;
    }
    if (value & (0xffffffffffffffffu64 << 63)) == 0 {
        return 9;
    }
    10
}

/// Given `u32` value compute varint encoded length.
#[inline(always)]
pub fn compute_raw_varint32_size(value: u32) -> usize {
    compute_raw_varint64_size(value as u64)
}