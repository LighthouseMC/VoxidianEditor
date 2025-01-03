use super::*;
use diff_match_patch_rs::{ DiffMatchPatch, Efficient, Patches };


#[derive(Debug)]
pub struct PatchFileC2SPacket {
    pub id      : u32,
    pub patches : Patches<Efficient>
}

impl PacketMeta for PatchFileC2SPacket {
    const PREFIX : u8 = 4;
}

impl PacketEncode for PatchFileC2SPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(&self.id);
        let dmp = DiffMatchPatch::new();
        buf.encode_write(&dmp.patch_to_text(&self.patches));
    }
}

impl PacketDecode for PatchFileC2SPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        let dmp = DiffMatchPatch::new();
        Ok(Self {
            id      : buf.read_decode()?,
            patches : dmp.patch_from_text(&buf.read_decode::<String>()?).map_err(|err| DecodeError::InvalidData(format!("{:?}", err)))?
        })
    }
}
