mod handshake;
pub use handshake::*;
mod keepalive;
pub use keepalive::*;
mod open_file;
pub use open_file::*;
mod close_file;
pub use close_file::*;
mod patch_file;
pub use patch_file::*;
mod selections;
pub use selections::*;


use super::*;


packet_group!{ pub enum C2SPackets {
    Handshake(HandshakeC2SPacket),
    Keepalive(KeepaliveC2SPacket),
    OpenFile(OpenFileC2SPacket),
    CloseFile(CloseFileC2SPacket),
    PatchFile(PatchFileC2SPacket),
    //Selections(SelectionsC2SPacket)
} }
