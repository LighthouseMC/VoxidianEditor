#![feature(
    let_chains,
    future_join,
    async_iterator
)]


mod ws;
use ws::WebSocketSender;
mod handle;
pub use handle::EditorHandle;
mod instance;
use instance::EditorInstanceManager;


use std::net::SocketAddr;
use std::io;
use std::sync::mpsc;
use std::time::Duration;
use std::pin::pin;
use async_std::stream::StreamExt;
use async_std::future;
use async_std::task::yield_now;
use futures::poll;
use tide::{ self, Request, Response };
use tide::http::mime::{ self, Mime };
use tide_websockets::{ WebSocket, WebSocketConnection, Message };
use const_format::str_replace;


pub struct EditorServer(());
impl EditorServer {


    pub async fn run<S : Into<SocketAddr>, F : Fn(EditorHandle) -> ()>(bind_addr : S, f : F) -> Result<(), io::Error> {
        let bind_addr = bind_addr.into();
        let mut server = tide::new();

        server.at("/robots.txt").get(Self::handle_robotstxt);

        server.at("/assets/image/logo_transparent.png").get(|req| Self::handle_asset(req, mime::PNG, include_bytes!("assets/image/logo_transparent.png")));

        server.at("/").get(Self::handle_root);

        server.at("/editor").get(Self::handle_editor);

        let (add_ws_tx, add_ws_rx) = mpsc::channel();
        server.at("/editor/ws").get(
            WebSocket::new(move |req, stream| Self::handle_editor_ws(req, stream, bind_addr, add_ws_tx.clone()))
                .with_protocols(&[env!("CARGO_PKG_NAME")])
        );

        server.at("*").get(Self::handle_404);

        let (handle_incoming_tx, handle_incoming_rx) = mpsc::channel();
        f(EditorHandle { handle_incoming_tx });
        let mut instance_manager = EditorInstanceManager::new(handle_incoming_rx, add_ws_rx);

        let mut a = pin!(server.listen(bind_addr));
        let mut b = pin!(instance_manager.run());
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
        const EDITORWS0 : &'static str = include_str!("assets/template/editor.ws.js");
        const EDITORWS  : &'static str = str_replace!(EDITORWS0, "{{VOXIDIAN_EDITOR_NAME}}", env!("CARGO_PKG_NAME"));
        const EDITOR0   : &'static str = include_str!("assets/template/editor.html");
        const EDITOR1   : &'static str = str_replace!(EDITOR0, "{{VOXIDIAN_EDITOR_VERSION}}", env!("CARGO_PKG_VERSION"));
        const EDITOR2   : &'static str = str_replace!(EDITOR1, "{{VOXIDIAN_EDITOR_COMMIT}}", env!("VOXIDIAN_EDITOR_COMMIT"));
        const EDITOR3   : &'static str = str_replace!(EDITOR2, "{{VOXIDIAN_EDITOR_COMMIT_HASH}}", env!("VOXIDIAN_EDITOR_COMMIT_HASH"));
        const EDITOR    : &'static str = str_replace!(EDITOR3, "{{EMBED_EDITOR_WS_JS}}", EDITORWS);
        Ok(Response::builder(200).content_type(mime::HTML).body(EDITOR).build())
    }


    async fn handle_editor_ws(req : Request<()>, mut ws : WebSocketConnection, bind_addr : SocketAddr, add_ws_tx : mpsc::Sender<(mpsc::Receiver<(u8, Vec<u8>)>, WebSocketSender, String)>) -> tide::Result<()> {
        if let Some(host) = req.host() && let Ok(host) = host.parse::<SocketAddr>() && host == bind_addr {} else {
            return Err(tide::Error::from_str(403, "403 Access Forbidden"));
        }
        let session_code = {
            let bytes = Self::read_ws_message::<{ws::C2S_HANDSHAKE}>(&mut ws, Duration::from_secs(1)).await?;
            String::from_utf8(bytes).map_err(|_| tide::Error::from_str(400, "400 Bad Request"))?
        };

        let (incoming_message_tx, incoming_message_rx) = mpsc::channel();
        let _ = add_ws_tx.send((incoming_message_rx, WebSocketSender::new(ws.clone()), session_code));

        let _ = incoming_message_tx.send((0, Vec::new()));

        while {
            if let Some(Ok(Message::Binary(mut incoming_message))) = ws.next().await {
                if (incoming_message.len() > 0) {
                    let prefix = incoming_message.remove(0);
                    matches!(incoming_message_tx.send((prefix, incoming_message)), Ok(_))
                } else { false }
            } else { false }
        } { yield_now().await }
        Ok(())
    }


    async fn read_ws_message<const PREFIX : u8>(stream : &mut WebSocketConnection, timeout : Duration) -> tide::Result<Vec<u8>> {
        let Ok(message) = future::timeout(timeout, stream.next()).await else {
            return Err(tide::Error::from_str(408, "408 Request Timeout"))
        };
        if let Some(Ok(Message::Binary(mut prefixed_data))) = message && prefixed_data.len() > 0 && prefixed_data.remove(0) == PREFIX {
            return Ok(prefixed_data);
        }
        return Err(tide::Error::from_str(400, "400 Bad Request"))
    }


    async fn handle_404(_ : Request<()>) -> tide::Result {
        Ok(Response::builder(404).content_type(mime::HTML).body(include_str!("assets/template/404.html")).build())
    }


}
