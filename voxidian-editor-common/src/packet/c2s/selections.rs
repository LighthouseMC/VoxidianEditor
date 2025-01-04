use super::*;


#[derive(Debug)]
pub struct SelectionsC2SPacket {
    pub file_id    : u32,
    pub selections : Vec<SelectionRange>
}

impl PacketMeta for SelectionsC2SPacket {
    const PREFIX : u8 = 5;
}

impl PacketEncode for SelectionsC2SPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
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

impl PacketDecode for SelectionsC2SPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            file_id    : buf.read_decode()?,
            selections : {
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


#[derive(Debug)]
pub struct SelectionRange {
    pub start_line   : usize,
    pub start_column : usize,
    pub end_line     : usize,
    pub end_column   : usize
}
