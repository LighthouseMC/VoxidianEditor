use super::*;


#[derive(Debug)]
pub struct OverwriteFileS2CPacket {
    id       : u32,
    contents : FileContents
}

impl PacketMeta for OverwriteFileS2CPacket {
    const PREFIX : u8 = 4;
}

impl PacketEncode for OverwriteFileS2CPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(self.id);
        match (&self.contents) {
            FileContents::NonText    => buf.encode_write(false),
            FileContents::Text(text) => {
                buf.encode_write(true);
                buf.encode_write(text);
            },
        }
    }
}

impl PacketDecode for OverwriteFileS2CPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            id       : buf.read_decode()?,
            contents : {
                let is_text = buf.read_decode::<bool>()?;
                if (is_text) { FileContents::NonText }
                else { FileContents::Text(buf.read_decode()?) }
            }
        })
    }
}


#[derive(Debug)]
pub enum FileContents {
    NonText,
    Text(String)
}
