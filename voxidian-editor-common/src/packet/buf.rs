use crate::packet::{
    PacketEncode,
    PrefixedPacketEncode,
    PacketDecode,
    DecodeError
};
use std::{ fmt, iter, slice, vec };


#[derive(Clone, Hash, Eq, PartialEq)]
pub struct PacketBuf {
    inner    : Vec<u8>,
    read_idx : usize
}


/// Constructors
impl PacketBuf {

    pub fn new() -> Self {
        PacketBuf {
            inner    : Vec::new(),
            read_idx : 0,
        }
    }

    pub fn of_encode<T : PacketEncode>(encodable : T) -> Self {
        let mut buf = Self::new();
        encodable.encode(&mut buf);
        buf
    }

    pub fn of_encode_prefixed<T : PrefixedPacketEncode>(encodable : T) -> Self {
        let mut buf = Self::new();
        encodable.encode_prefixed(&mut buf);
        buf
    }

}
impl From<Vec<u8>> for PacketBuf {
    fn from(value : Vec<u8>) -> Self { Self {
        inner    : value,
        read_idx : 0
    } }
}


/// Deconstructors
impl PacketBuf {

    pub fn into_inner(self) -> Vec<u8> {
        self.inner.into_iter().skip(self.read_idx).collect::<Vec<_>>()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.inner.as_slice().get(self.read_idx..).unwrap_or(&[])
    }

}


/// Basic Operations
impl PacketBuf {

    #[track_caller]
    pub fn seek(&mut self, idx : usize) {
        assert!(idx <= self.inner.len(), "Seek index exceeded packet size");
        self.read_idx = idx;
    }

    #[track_caller]
    pub fn skip(&mut self, count : usize) {
        self.seek(self.read_idx + count);
    }

    pub fn remaining(&self) -> usize {
        self.inner.len() - self.read_idx
    }

    pub fn write_u8(&mut self, byte : u8) -> () {
        self.inner.push(byte);
    }

    pub fn write_u8s(&mut self, data : &[u8]) -> () {
        self.inner.extend_from_slice(data);
    }

    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        let byte = self.inner
            .get(self.read_idx).ok_or(DecodeError::EndOfBuffer)?;
        self.read_idx += 1;
        Ok(*byte)
    }

    pub fn read_u8s_const<const BYTES : usize>(&mut self) -> Result<[u8; BYTES], DecodeError> {
        if (BYTES == 0) { return Ok([0; BYTES]); }
        let out = self.inner.iter().skip(self.read_idx).cloned().array_chunks()
            .next().ok_or(DecodeError::EndOfBuffer)?;
        self.read_idx += BYTES;
        Ok(out)
    }

    pub fn read_u8s(&mut self, bytes : usize) -> Result<Vec<u8>, DecodeError> {
        if (bytes == 0) { return Ok(Vec::new()) }
        let mut out  = Vec::with_capacity(bytes);
        let mut data = self.inner.iter().skip(self.read_idx);
        for _ in 0..bytes {
            out.push(*data.next().ok_or(DecodeError::EndOfBuffer)?);
        }
        self.read_idx += bytes;
        Ok(out)
    }

}


impl<'l> IntoIterator for &'l PacketBuf {
    type Item     = &'l u8;
    type IntoIter = iter::Skip<slice::Iter<'l, u8>>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter().skip(self.read_idx)
    }
}
impl IntoIterator for PacketBuf {
    type Item     = u8;
    type IntoIter = iter::Skip<vec::IntoIter<u8>>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter().skip(self.read_idx)
    }
}
/// Iterator
impl PacketBuf {
    pub fn iter(&self) -> impl Iterator<Item = u8> { (&self).into_iter().map(|byte| *byte) }
}


/// Encode & Decode
impl PacketBuf {

    pub fn encode_write<T : PacketEncode>(&mut self, encodable : T) -> () {
        encodable.encode(self);
    }

    pub fn read_decode<T : PacketDecode>(&mut self) -> Result<T, DecodeError> {
        T::decode(self)
    }

}


impl fmt::Debug for PacketBuf {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PacketBuf(0x")?;
        for byte in self.inner.iter().skip(self.read_idx) {
            write!(f, "{:X}", byte)?;
        }
        write!(f, ")")
    }
}
