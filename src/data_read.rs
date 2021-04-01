use bytes::{Bytes, Buf};
use anyhow::*;
use crate::{FixedNumber, VarNumber};

pub trait IDataRead{
    ///读 定长buf 到 tar
    fn read_buf(&mut self,buff:&mut [u8])->Result<()>;
    ///读 定长 数字
    fn read_fixed<T:FixedNumber>(&mut self)->Result<T>;
    ///读 边长数字
    fn read_var_integer<T:VarNumber>(&mut self)->Result<T>;
}


impl IDataRead for Bytes{
    #[inline]
    fn read_buf(&mut self, buff: &mut [u8]) ->Result<()> {
        ensure!(self.remaining() >= buff.len(),"read buff,buff too max");
        self.copy_to_slice(buff);
        Ok(())
    }
    #[inline]
    fn read_fixed<T: FixedNumber>(&mut self) -> Result<T> {
        T::read(self)
    }
    #[inline]
    fn read_var_integer<T: VarNumber>(&mut self) -> Result<T> {
       T::read_var(self)
    }
}