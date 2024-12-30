#![feature(
    let_chains
)]


mod ws;
mod handle;
use handle::EditorHandleInner;
pub use handle::EditorHandle;
mod instance;
use instance::EditorInstance;
mod session;
use session::EditorSession;


use std::net::SocketAddr;
use std::io;
use std::time::Duration;
use async_std::stream::StreamExt;
use async_std::future;
use tide::{ self, Request, Response };
use tide::http::mime::{ self, Mime };
use tide_websockets::{ WebSocket, WebSocketConnection, Message };
use const_format::str_replace;
use uuid::Uuid;


pub struct EditorServer(());
impl EditorServer {


    pub async fn run<S : Into<SocketAddr>, F : Fn(EditorHandle) -> ()>(bind_addr : S, f : F) -> Result<(), io::Error> {
        let bind_addr = bind_addr.into();
        let mut server = tide::new();

        let handle = EditorHandleInner::new();

        server.at("/robots.txt").get(Self::handle_robotstxt);

        server.at("/assets/image/logo_transparent.png").get(|req| Self::handle_asset(req, mime::PNG, include_bytes!("assets/image/logo_transparent.png")));

        server.at("/").get(Self::handle_root);

        server.at("/editor").get(Self::handle_editor);

        server.at("/editor/ws").get(
            WebSocket::new(move |req, stream| Self::handle_editor_ws(req, stream, bind_addr))
                .with_protocols(&[env!("CARGO_PKG_NAME")])
        );

        server.at("*").get(Self::handle_404);

        f(EditorHandle(handle));

        server.listen(bind_addr).await
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


    async fn handle_editor_ws(req : Request<()>, mut stream : WebSocketConnection, bind_addr : SocketAddr) -> tide::Result<()> {
        if let Some(host) = req.host() && let Ok(host) = host.parse::<SocketAddr>() && host == bind_addr {} else {
            return Err(tide::Error::from_str(403, "403 Access Forbidden"));
        }

        let session_code = {
            let bytes = Self::read_ws_message::<{ws::C2S_HANDSHAKE}>(&mut stream, Duration::from_secs(1)).await?;
            let bytes = bytes.try_into().map_err(|_| tide::Error::from_str(400, "400 Bad Request"))?;
            Uuid::from_bytes(bytes)
        };

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
