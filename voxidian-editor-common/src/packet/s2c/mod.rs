mod disconnect;
pub use disconnect::*;
mod keepalive;
pub use keepalive::*;
mod login_success;
pub use login_success::*;
mod initial_state;
pub use initial_state::*;
mod overwrite_file;
pub use overwrite_file::*;


use super::*;


packet_group!{ pub enum S2CPackets {
    Disconnect(DisconnectS2CPacket),
    Keepalive(KeepaliveS2CPacket),
    LoginSuccess(LoginSuccessS2CPacket),
    InitialState(InitialStateS2CPacket),
    OvewriteFile(OverwriteFileS2CPacket)
} }
