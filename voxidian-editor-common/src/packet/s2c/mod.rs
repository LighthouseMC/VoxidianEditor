mod disconnect;
pub use disconnect::*;
mod keepalive;
pub use keepalive::*;
mod login_success;
pub use login_success::*;
mod initial_state;
pub use initial_state::*;


use super::*;


packet_group!{ pub enum S2CPackets {
    Disconnect(DisconnectS2CPacket),
    Keepalive(KeepaliveS2CPacket),
    LoginSuccess(LoginSuccessS2CPacket),
    InitialState(InitialStateS2CPacket)
} }
