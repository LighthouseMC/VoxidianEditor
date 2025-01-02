use super::*;


#[derive(Debug)]
pub struct DisconnectS2CPacket {
    pub reason : String
}

impl PacketMeta for DisconnectS2CPacket {
    const PREFIX : u8 = 0;
}

impl PacketEncode for DisconnectS2CPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(&self.reason);
    }
}

impl PacketDecode for DisconnectS2CPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        let reason = buf.read_decode::<String>()?;
        Ok(Self { reason })
    }
}
