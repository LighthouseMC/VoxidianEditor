#![feature(
    let_chains,
    future_join,
    async_iterator,
    mpmc_channel
)]


mod ws;
use ws::WebSocketContainer;
mod handle;
pub use handle::EditorHandle;
mod instance;
use instance::EditorInstanceManager;


use voxidian_editor_common::packet::{ PacketBuf, PrefixedPacketDecode };
use voxidian_editor_common::packet::s2c::DisconnectS2CPacket;
use voxidian_editor_common::packet::c2s::HandshakeC2SPacket;
use voxidian_database::VoxidianDB;
use std::net::SocketAddr;
use std::io;
use std::sync::{ mpmc, Arc };
use std::time::Duration;
use std::pin::pin;
use async_std::stream::StreamExt;
use async_std::future::timeout;
use async_std::task::yield_now;
use futures::poll;
use tide::{ self, Request, Response };
use tide::http::mime::{ self, Mime };
use tide_websockets::{ WebSocket, WebSocketConnection, Message };
use const_format::str_replace;


pub struct EditorServer(());
impl EditorServer {


    pub async fn run<S : Into<SocketAddr>, F : Fn(EditorHandle) -> ()>(bind_addr : S, db : Arc<VoxidianDB>, f : F) -> Result<(), io::Error> {
        let bind_addr = bind_addr.into();
        let mut server = tide::new();

        server.at("/robots.txt").get(Self::handle_robotstxt);

        server.at("/assets/image/logo_transparent.png").get(|req| Self::handle_asset(req, mime::PNG, include_bytes!("assets/image/logo_transparent.png")));

        server.at("/").get(Self::handle_root);

        server.at("/editor").get(Self::handle_editor);
        server.at("/editor/voxidian_editor_frontend.js").get(|req| Self::handle_asset(req, mime::JAVASCRIPT, include_bytes!("../voxidian-editor-frontend/pkg/voxidian_editor_frontend.js")));
        server.at("/editor/voxidian_editor_frontend_bg.wasm").get(|req| Self::handle_asset(req, mime::WASM, include_bytes!("../voxidian-editor-frontend/pkg/voxidian_editor_frontend_bg.wasm")));

        let (add_ws_tx, add_ws_rx) = mpmc::channel();
        server.at("/editor/ws").get(
            WebSocket::new(move |req, stream| Self::handle_editor_ws(req, stream, bind_addr, add_ws_tx.clone()))
                .with_protocols(&["voxidian-editor"])
        );

        server.at("*").get(Self::handle_404);

        let (handle_incoming_tx, handle_incoming_rx) = mpmc::channel();
        let (handle_outgoing_tx, handle_outgoing_rx) = mpmc::channel();
        f(EditorHandle {
            handle_incoming_tx,
            handle_outgoing_rx
        });
        let mut instance_manager = EditorInstanceManager::new(handle_incoming_rx, handle_outgoing_tx, add_ws_rx);

        let mut a = pin!(server.listen(bind_addr));
        let mut b = pin!(instance_manager.run(db));
        loop {
            let ap = poll!(&mut a);
            if (ap.is_ready()) { return a.await; }
            let bp = poll!(&mut b);
            if (bp.is_ready()) { unreachable!(); }
            yield_now().await;
        }
    }


    async fn handle_robotstxt(_ : Request<()>) -> tide::Result {
        Ok(Response::builder(200).content_type(mime::PLAIN).body(include_str!("assets/template/robots.txt")).build())
    }

    async fn handle_asset(_ : Request<()>, mime : Mime, data : &[u8]) -> tide::Result {
        Ok(Response::builder(200).content_type(mime).body(data).build())
    }


    async fn handle_root(_ : Request<()>) -> tide::Result {
        Ok(Response::builder(200).content_type(mime::HTML).body(include_str!("assets/template/root.html")).build())
    }


    async fn handle_editor(_ : Request<()>) -> tide::Result {
        const EDITOR0 : &'static str = include_str!("assets/template/editor.html");
        const EDITOR1 : &'static str = str_replace!(EDITOR0, "{{VOXIDIAN_EDITOR_VERSION}}", env!("CARGO_PKG_VERSION"));
        const EDITOR2 : &'static str = str_replace!(EDITOR1, "{{VOXIDIAN_EDITOR_COMMIT}}", env!("VOXIDIAN_EDITOR_COMMIT"));
        const EDITOR  : &'static str = str_replace!(EDITOR2, "{{VOXIDIAN_EDITOR_COMMIT_HASH}}", env!("VOXIDIAN_EDITOR_COMMIT_HASH"));
        Ok(Response::builder(200).content_type(mime::HTML).body(EDITOR).build())
    }


    async fn handle_editor_ws(req : Request<()>, mut ws : WebSocketConnection, bind_addr : SocketAddr, add_ws_tx : mpmc::Sender<(WebSocketContainer, String)>) -> tide::Result<()> {
        /*if let Some(host) = req.host() && let Ok(host) = host.parse::<SocketAddr>() && host == bind_addr {} else {
            return Err(tide::Error::from_str(403, "403 Access Forbidden"));
        }*/ // TODO: Fix this
        let session_code = match (match (timeout(Duration::from_secs(1), ws.next()).await) {
            Err(_) => Err(("Login took too long".to_string(), 408, "408 Request Timeout")),
            Ok(None) => Err(("No data".to_string(), 400, "400 Bad Request")),
            Ok(Some(Err(err))) => Err((format!("An error occured: {}", err), 400, "400 Bad Request")),
            Ok(Some(Ok(Message::Binary(data)))) => {
                let mut buf = PacketBuf::from(data);
                match (HandshakeC2SPacket::decode_prefixed(&mut buf)) {
                    Err(err) => Err((format!("An error occured: {:?}", err), 400, "400 Bad Request")),
                    Ok(handshake) => Ok(handshake.session_code)
                }
            },
            Ok(Some(Ok(_))) => Err(("Bad data format".to_string(), 400, "400 Bad Request"))
        }) {
            Err((reason, code, error)) => {
                let _ = ws.send_bytes(PacketBuf::of_encode_prefixed(DisconnectS2CPacket { reason }).into_inner()).await;
                return Err(tide::Error::from_str(code, error));
            },
            Ok(session_code) => session_code
        };

        let ws = WebSocketContainer::new(ws);

        let _ = add_ws_tx.send((ws.clone(), session_code));

        while (! ws.is_closed()) {
            yield_now().await;
        }
        Ok(())
    }


    async fn handle_404(_ : Request<()>) -> tide::Result {
        Ok(Response::builder(404).content_type(mime::HTML).body(include_str!("assets/template/404.html")).build())
    }


}
