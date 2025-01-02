use super::*;


#[derive(Debug)]
pub struct HandshakeC2SPacket {
    pub session_code : String
}

impl PacketMeta for HandshakeC2SPacket {
    const PREFIX : u8 = 0;
}

impl PacketEncode for HandshakeC2SPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(&self.session_code);
    }
}

impl PacketDecode for HandshakeC2SPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        let session_code = buf.read_decode::<String>()?;
        Ok(Self { session_code })
    }
}
