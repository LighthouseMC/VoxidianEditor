mod handshake;
pub use handshake::*;
mod keepalive;
pub use keepalive::*;


use super::*;


packet_group!{ pub enum C2SPackets {
    Handshake(HandshakeC2SPacket),
    Keepalive(KeepaliveC2SPacket)
} }
