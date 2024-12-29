#![feature(decl_macro)]


use std::io;
use std::net::SocketAddr;
use async_std::task::spawn_blocking;
use rouille::{ Request, Response };


pub struct EditorServer(());
impl EditorServer {

    pub async fn start<S : Into<SocketAddr>>(bind_addr : S) -> io::Result<()> {
        let bind_addr = bind_addr.into();
        spawn_blocking(move || {
            rouille::start_server(bind_addr, Self::request_handler)
        }).await
    }

    fn request_handler(req : &Request) -> Response {
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
                if let Some(instance_id) = url.strip_prefix("/editor/") {
                    return Response::html(include_str!("assets/template/editor.html"))
                }
            }

        }

        // 4040
        Response::html(include_str!("assets/template/404.html")).with_status_code(404)
    }

}


macro try_route_asset( $url:expr, $mime:tt, $path:tt ) {
    if ($url == concat!("/assets", $path)) {
        return Response::from_data($mime, include_bytes!(concat!("assets", $path)));
    }
}
