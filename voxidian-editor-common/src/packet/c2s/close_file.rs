use super::*;


#[derive(Debug)]
pub struct CloseFileC2SPacket {
    pub id : u32
}

impl PacketMeta for CloseFileC2SPacket {
    const PREFIX : u8 = 3;
}

impl PacketEncode for CloseFileC2SPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(&self.id);
    }
}

impl PacketDecode for CloseFileC2SPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            id : buf.read_decode()?
        })
    }
}
