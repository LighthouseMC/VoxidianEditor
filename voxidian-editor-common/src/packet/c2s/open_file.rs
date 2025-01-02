use super::*;


#[derive(Debug)]
pub struct OpenFileC2SPacket {
    pub id : u32
}

impl PacketMeta for OpenFileC2SPacket {
    const PREFIX : u8 = 2;
}

impl PacketEncode for OpenFileC2SPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(&self.id);
    }
}

impl PacketDecode for OpenFileC2SPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            id : buf.read_decode()?
        })
    }
}
