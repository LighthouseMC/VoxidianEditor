use super::*;
use diff_match_patch_rs::{ DiffMatchPatch, Efficient, Patches };


#[derive(Debug)]
pub struct PatchFileS2CPacket {
    pub id      : u32,
    pub patches : Patches<Efficient>
}

impl PacketMeta for PatchFileS2CPacket {
    const PREFIX : u8 = 5;
}

impl PacketEncode for PatchFileS2CPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(&self.id);
        let dmp = DiffMatchPatch::new();
        buf.encode_write(&dmp.patch_to_text(&self.patches));
    }
}

impl PacketDecode for PatchFileS2CPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        let dmp = DiffMatchPatch::new();
        Ok(Self {
            id      : buf.read_decode()?,
            patches : dmp.patch_from_text(&buf.read_decode::<String>()?).map_err(|err| DecodeError::InvalidData(format!("{:?}", err)))?
        })
    }
}
