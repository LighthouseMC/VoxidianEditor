use voxidian_editor::*;
use std::net::SocketAddr;


#[async_std::main]
async fn main() {
    EditorServer::start("127.0.0.1:8080".parse::<SocketAddr>().unwrap()).await.unwrap();
}
