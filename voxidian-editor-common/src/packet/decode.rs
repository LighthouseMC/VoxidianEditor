use crate::packet::{ PacketBuf, PacketMeta };
use std::mem::MaybeUninit;
use uuid::Uuid;


pub trait PacketDecode : Sized {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError>;
}


#[derive(Debug)]
pub enum DecodeError {

    /// The end of the buffer has been reached.
    EndOfBuffer,

    /// The data in the buffer could not be parsed properly.
    /// 
    /// Includes a message.
    InvalidData(String),

    /// The packet decoder did not consume the length specified in the previously received header.
    UnconsumedBuffer,

    /// The received packet ID did not match any registered packet.
    /// 
    /// Includes the ID that wasn't recognised.
    UnknownPacketPrefix(u8)

}


macro packet_decode_num( $($types:ty),* $(,)? ) { $(
    impl PacketDecode for $types { fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(<$types>::from_be_bytes(buf.read_u8s_const()?))
    } }
)* }
packet_decode_num!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);

impl PacketDecode for bool { fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
    Ok(buf.read_u8()? != 0)
} }

impl PacketDecode for Uuid { fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
    let msb = buf.read_decode::<u64>()?;
    let lsb = buf.read_decode::<u64>()?;
    Ok(Self::from_u64_pair(msb, lsb))
} }

impl PacketDecode for String { fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
    let len = buf.read_decode::<u32>()? as usize;
    Ok(String::from_utf8(buf.read_u8s(len)?).map_err(|_| DecodeError::InvalidData("String data is not valid UTF8".to_string()))?)
} }

impl<T : PacketDecode> PacketDecode for Option<T> { fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
    let is_some = buf.read_u8()? != 0;
    Ok(if (is_some) { Some(buf.read_decode::<T>()?) } else { None })
} }

impl<T : PacketDecode, const LEN : usize> PacketDecode for [T; LEN] { fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
    let mut out : Self = unsafe{ MaybeUninit::uninit().assume_init() };
    for i in 0..LEN { out[i] = buf.read_decode::<T>()?; }
    Ok(out)
} }


pub trait PrefixedPacketDecode : Sized {
    fn decode_prefixed(buf : &mut PacketBuf) -> Result<Self, DecodeError>;
}

impl<T : PacketDecode + PacketMeta> PrefixedPacketDecode for T {
    fn decode_prefixed(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        let prefix = buf.read_u8()?;
        if (prefix != Self::PREFIX) {
            return Err(DecodeError::UnknownPacketPrefix(prefix));
        }
        Self::decode(buf)
    }
}
