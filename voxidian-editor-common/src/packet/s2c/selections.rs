use super::*;
use super::c2s::SelectionRange;
use uuid::Uuid;


#[derive(Debug)]
pub struct SelectionsS2CPacket {
    pub client_uuid : Uuid,
    pub client_name : String,
    pub file_id     : u32,
    pub selections  : Vec<SelectionRange>
}

impl PacketMeta for SelectionsS2CPacket {
    const PREFIX : u8 = 5;
}

impl PacketEncode for SelectionsS2CPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(self.client_uuid);
        buf.encode_write(&self.client_name);
        buf.encode_write(self.file_id);
        buf.encode_write(self.selections.len() as u32);
        for selection in &self.selections {
            buf.encode_write(selection.start_line   as u32);
            buf.encode_write(selection.start_column as u32);
            buf.encode_write(selection.end_line     as u32);
            buf.encode_write(selection.end_column   as u32);
        }
    }
}

impl PacketDecode for SelectionsS2CPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            client_uuid : buf.read_decode()?,
            client_name : buf.read_decode()?,
            file_id     : buf.read_decode()?,
            selections  : {
                let len = buf.read_decode::<u32>()? as usize;
                let mut out = Vec::with_capacity(len);
                for _ in 0..len { out.push(SelectionRange {
                    start_line   : buf.read_decode::<u32>()? as usize,
                    start_column : buf.read_decode::<u32>()? as usize,
                    end_line     : buf.read_decode::<u32>()? as usize,
                    end_column   : buf.read_decode::<u32>()? as usize
                }); }
                out
            }
        })
    }
}
