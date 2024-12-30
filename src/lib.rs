#![feature(
    decl_macro,
    mpmc_channel
)]


mod ws;
use ws::*;


use voxidian_database::VoxidianDB;
use std::net::SocketAddr;
use std::cell::LazyCell;
use std::sync::{ mpsc, mpmc, Mutex };
use std::error::Error;
use std::collections::VecDeque;
use std::time::Duration;
use rouille::{ Server, Request, Response };
use rouille::websocket::{ self, Websocket };
use const_format::str_replace;


// Rouille was not designed for async, at all. However, I can't find ANY non-tokio webserver libraries.
static WSR_TXRX : Mutex<LazyCell<(mpmc::Sender<mpsc::Receiver<Websocket>>, mpmc::Receiver<mpsc::Receiver<Websocket>>)>> = Mutex::new(LazyCell::new(|| mpmc::channel()));


pub struct EditorServer {
    server  : Server<fn(&Request) -> Response>,
    wsr_rx  : mpmc::Receiver<mpsc::Receiver<Websocket>>,
    ws_rxs  : VecDeque<mpsc::Receiver<Websocket>>,
    clients : VecDeque<WSClient>
}
impl EditorServer {


    pub unsafe fn start<S : Into<SocketAddr>>(bind_addr : S) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let bind_addr = bind_addr.into();
        Ok(Self {
            server  : Self::create_server(bind_addr, Self::accept_request),
            wsr_rx  : WSR_TXRX.lock().unwrap().1.clone(),
            ws_rxs  : VecDeque::new(),
            clients : VecDeque::new()
        })
    }
    // Forces a type-coersion.
    #[inline(always)]
    fn create_server<S : Into<SocketAddr>, F : Fn(&Request) -> Response + Send + Sync + 'static>(bind_addr : S, f : F) -> Server<F> {
        Server::new(bind_addr.into(), f).unwrap()
    }


    fn accept_request(req : &Request) -> Response {
        let url = req.url().strip_suffix("").map_or_else(|| req.url(), |url| url.to_string());
        println!("{} {}", req.method(), url);
        if (req.method() == "GET") {

            // Root
            if (url == "/") {
                return Response::html(include_str!("assets/template/index.html"))
            }

            // Robots.txt
            if (url == "/robots.txt") {
                return Response::text(include_str!("assets/template/robots.txt"))
            }

            // Assets
            try_route_asset!(url, "image/png", "/image/logo_transparent.png");
            try_route_asset!(url, "font/ttf", "/font/dejavu_sans_mono.ttf");

            // Editor
            if (url.starts_with("/editor")) {
                if let Some(_) = url.strip_prefix("/editor/") { // TODO: instance_id
                    const EDITORWS0 : &'static str = include_str!("assets/template/editor.ws.js");
                    const EDITORWS  : &'static str = str_replace!(EDITORWS0, "{{VOXIDIAN_EDITOR_NAME}}", env!("CARGO_PKG_NAME"));
                    const EDITOR0   : &'static str = include_str!("assets/template/editor.html");
                    const EDITOR1   : &'static str = str_replace!(EDITOR0, "{{VOXIDIAN_EDITOR_VERSION}}", env!("CARGO_PKG_VERSION"));
                    const EDITOR2   : &'static str = str_replace!(EDITOR1, "{{VOXIDIAN_EDITOR_COMMIT}}", env!("VOXIDIAN_EDITOR_COMMIT"));
                    const EDITOR3   : &'static str = str_replace!(EDITOR2, "{{VOXIDIAN_EDITOR_COMMIT_HASH}}", env!("VOXIDIAN_EDITOR_COMMIT_HASH"));
                    const EDITOR    : &'static str = str_replace!(EDITOR3, "{{EMBED_EDITOR_WS_JS}}", EDITORWS);
                    return Response::html(EDITOR);
                }
            }
            if (url == "/ws") {
                return match (websocket::start(req, Some(env!("CARGO_PKG_NAME")))) {
                    Err(err) => Response::text(format!("400 Bad Request: {}", err)).with_status_code(400),
                    Ok((resp, websocket)) => {
                        let _ = WSR_TXRX.lock().unwrap().0.send(websocket);
                        resp
                    }
                };
            }

        }

        // 404
        Response::html(include_str!("assets/template/404.html")).with_status_code(404)
    }


    pub fn update_ws(&mut self, db : &VoxidianDB) {
        self.server.poll_timeout(Duration::ZERO);
        while let Ok(ws_rxs) = self.wsr_rx.try_recv() {
            self.ws_rxs.push_back(ws_rxs);
        }
        self.ws_rxs.retain(|ws_rx| {
            if let Ok(client) = ws_rx.try_recv() {
                self.clients.push_back(WSClient::new(client));
                false
            } else { true }
        });
        self.clients.retain(|client| ! client.just_closed());
    }


}


macro try_route_asset( $url:expr, $mime:tt, $path:tt ) {
    if ($url == concat!("/assets", $path)) {
        return Response::from_data($mime, include_bytes!(concat!("assets", $path)));
    }
}
