mod handshake;
pub use handshake::*;
mod keepalive;
pub use keepalive::*;
mod open_file;
pub use open_file::*;
mod close_file;
pub use close_file::*;


use super::*;


packet_group!{ pub enum C2SPackets {
    Handshake(HandshakeC2SPacket),
    Keepalive(KeepaliveC2SPacket),
    OpenFile(OpenFileC2SPacket),
    CloseFile(CloseFileC2SPacket)
} }
