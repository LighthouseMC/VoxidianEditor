use super::*;


#[derive(Debug)]
pub struct InitialStateS2CPacket {
    pub subserver_id          : u32,
    pub subserver_name        : String,
    pub subserver_owner_name  : String,
    pub subserver_description : String,
    pub file_entities         : Vec<FileTreeEntry>
}

impl PacketMeta for InitialStateS2CPacket {
    const PREFIX : u8 = 3;
}

impl PacketEncode for InitialStateS2CPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(&self.subserver_id);
        buf.encode_write(&self.subserver_name);
        buf.encode_write(&self.subserver_owner_name);
        buf.encode_write(&self.subserver_description);
        buf.encode_write(&(self.file_entities.len() as u32));
        for file in &self.file_entities {
            buf.encode_write(file.id);
            buf.encode_write(file.is_dir);
            buf.encode_write(&file.path);
        }
    }
}

impl PacketDecode for InitialStateS2CPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            subserver_id          : buf.read_decode::<u32>()?,
            subserver_name        : buf.read_decode::<String>()?,
            subserver_owner_name  : buf.read_decode::<String>()?,
            subserver_description : buf.read_decode::<String>()?,
            file_entities                 : {
                let     count = buf.read_decode::<u32>()? as usize;
                let mut files = Vec::with_capacity(count);
                for _ in 0..count {
                    files.push(FileTreeEntry {
                        id     : buf.read_decode::<u32>()?,
                        is_dir : buf.read_decode::<bool>()?,
                        path   : buf.read_decode::<String>()?
                    })
                }
                files
            }
        })
    }
}


#[derive(Debug)]
pub struct FileTreeEntry {
    pub id     : u32,
    pub is_dir : bool,
    pub path   : String
}
