use bytes::{BytesMut, BufMut};
use crate::{FixedNumber, VarNumber};

pub trait  IData{
    /// 写入一段二进制
    fn write_buf(&mut self,buff:&[u8]);
    /// 写入一段二进制到指定位置,如果DEBUG 将检查是否越界
    fn write_buf_at(&mut self,idx:usize,buff:&[u8]);
    /// 写入基本数字类型,定长
    fn write_fixed<T:FixedNumber>(&mut self,v:T);
    /// 写入基本数字类型到指定位置,定长,如果DEBUG 将检查是否越界
    fn write_fixed_at<T:FixedNumber>(&mut self,idx:usize,v:T);
    /// 写入基本数字类型,变长
    fn write_var_integer<T:VarNumber>(&mut self,v:T);
}
impl IData for BytesMut{
    #[inline]
    fn write_buf(&mut self,buff: &[u8]) {
        self.put_slice(buff)
    }
    #[inline]
    fn write_buf_at(&mut self, idx: usize, buff: &[u8]) {
        debug_assert!(idx+buff.len() <= self.len(), "idx too max {}>{}",idx+buff.len(),self.len());
        let len=self.len();
        unsafe {
            self.set_len(idx);
            self.put_slice(buff);
            self.set_len(len);
        }
    }
    #[inline]
    fn write_fixed<T:FixedNumber>(&mut self,v:T){
        v.write(self);
    }
    #[inline]
    fn write_fixed_at<T:FixedNumber>(&mut self,idx:usize,v:T){
        v.write_at(idx,self);
    }
    #[inline]
    fn write_var_integer<T: VarNumber>(&mut self, v: T) {
        v.write_var(self);
    }
}
