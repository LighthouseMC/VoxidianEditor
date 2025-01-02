use super::*;


#[derive(Debug)]
pub struct KeepaliveS2CPacket;

impl PacketMeta for KeepaliveS2CPacket {
    const PREFIX : u8 = 1;
}

impl PacketEncode for KeepaliveS2CPacket {
    fn encode(&self, _buf : &mut PacketBuf) -> () { }
}

impl PacketDecode for KeepaliveS2CPacket {
    fn decode(_buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self)
    }
}
