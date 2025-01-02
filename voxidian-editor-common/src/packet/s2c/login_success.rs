use super::*;


#[derive(Debug)]
pub struct LoginSuccessS2CPacket;

impl PacketMeta for LoginSuccessS2CPacket {
    const PREFIX : u8 = 2;
}

impl PacketEncode for LoginSuccessS2CPacket {
    fn encode(&self, _buf : &mut PacketBuf) -> () { }
}

impl PacketDecode for LoginSuccessS2CPacket {
    fn decode(_buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self)
    }
}
