use crate::packet::{ PacketBuf, PacketMeta };
use uuid::Uuid;


pub trait PacketEncode {
    fn encode(&self, buf : &mut PacketBuf) -> ();
}

impl<T : PacketEncode> PacketEncode for &T { fn encode(&self, buf : &mut PacketBuf) -> () {
    (*self).encode(buf)
} }

macro packet_encode_num( $($types:ty),* $(,)? ) { $(
    impl PacketEncode for $types { fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.write_u8s(&self.to_be_bytes());
    } }
)* }
packet_encode_num!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);

impl PacketEncode for bool { fn encode(&self, buf : &mut PacketBuf) -> () {
    buf.write_u8(if (*self) { 1 } else { 0 });
} }

impl PacketEncode for Uuid { fn encode(&self, buf : &mut PacketBuf) -> () {
    let (msb, lsb) = self.as_u64_pair();
    buf.encode_write(msb);
    buf.encode_write(lsb);
} }

impl PacketEncode for &str { fn encode(&self, buf : &mut PacketBuf) -> () {
    buf.encode_write(self.len() as u32);
    buf.write_u8s(self.as_bytes());
} }

impl PacketEncode for String { fn encode(&self, buf : &mut PacketBuf) -> () { self.as_str().encode(buf) } }

impl<T : PacketEncode> PacketEncode for Option<T> { fn encode(&self, buf : &mut PacketBuf) -> () {
    if let Some(value) = self {
        buf.write_u8(1);
        buf.encode_write(value)
    } else {
        buf.write_u8(0);
    }
} }


pub trait PrefixedPacketEncode {
    fn encode_prefixed(&self, buf : &mut PacketBuf) -> ();
}

impl<T : PacketEncode + PacketMeta> PrefixedPacketEncode for T {
    fn encode_prefixed(&self, buf : &mut PacketBuf) -> () {
        buf.write_u8(Self::PREFIX);
        self.encode(buf);
    }
}
