use super::*;


#[derive(Debug)]
pub struct KeepaliveC2SPacket {
    pub index : u64
}

impl PacketMeta for KeepaliveC2SPacket {
    const PREFIX : u8 = 1;
}

impl PacketEncode for KeepaliveC2SPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(&self.index);
    }
}

impl PacketDecode for KeepaliveC2SPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            index : buf.read_decode()?
        })
    }
}
