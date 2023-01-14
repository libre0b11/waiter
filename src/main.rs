use std::{fs::File, path::Path};
use rouille::{Response, Request};

const CACHE_TIME_ASSETS: u64 = 31536000;
const CACHE_TIME_CONTENT: u64 = 43200;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// Address for the server to run on
   #[arg(short, long, default_value_t = String::from("localhost:4000"))]
   address: String,
}

fn main() {
    let args = Args::parse();
    println!("Now listening on {}", args.address);

    rouille::start_server(args.address, move |request| {
        let mut response = rouille::match_assets(request, ".");

        if !response.is_success() {
            if request.url() == "/" {
                response = serve_index()
            } else {
                response = serve_404()
            }
        }

        response = set_cache_time(response, request.url());
        response = set_correct_mime_type(response, request);
        response = set_server_header(response);

        response
    });
}

fn serve_index() -> Response {
    match find_index() {
        Some((filename, mime_type)) => serve_file(filename, mime_type),
        None => serve_404()
    }
}

fn find_index() -> Option<(String, String)> {
    let possible_indexes = vec![
        (String::from("index.htmd"), String::from("text/htmd")),
        (String::from("index.txt"), String::from("text/plain")),
        (String::from("index.html"), String::from("text/html")),
        (String::from("index.xml"), String::from("text/xml")),
    ];

    possible_indexes
        .iter()
        .find(|&(filename, _)| Path::new(&filename).exists())
        .cloned()
}

fn serve_file(filename: String, mime_type: String) -> Response {
    let file = File::open(filename).unwrap();
    Response::from_file(mime_type, file)
}

fn serve_404() -> Response {
    Response::text("Resource was not found on this server").with_status_code(404)
}

fn set_cache_time(response: Response, request_url: String) -> Response {
    let cache_time_in_seconds = get_cache_time_for_filetype(request_url);
    response.with_public_cache(cache_time_in_seconds)
}

fn get_cache_time_for_filetype(filename: String) -> u64 {
    if is_static_asset(filename) {
        CACHE_TIME_ASSETS
    } else {
        CACHE_TIME_CONTENT
    }
}

fn is_static_asset(filename: String) -> bool {
    let asset_file_types = vec![
        ".ico", ".jpg", ".jpeg", ".png", ".webp", ".gif", ".svg", ".woff", ".woff2",
    ];

    asset_file_types
        .iter()
        .any(|&suffix| filename.ends_with(suffix))
}

fn set_correct_mime_type(
    response: Response,
    request: &Request,
) -> rouille::Response {
    if is_htmd_file(&response, request) {
        if accepts_htmd_mime_type(request) {
            response.with_unique_header("Content-Type", "text/htmd")
        } else {
            response.with_unique_header("Content-Type", "text/plain")
        }
    } else {
        response
    }
}

fn is_htmd_file(response: &Response, request: &Request) -> bool {
    request.url().ends_with(".htmd") || current_response_has_htmd_content_type(response)
}

fn current_response_has_htmd_content_type(response: &Response) -> bool {
    let content_type = response.headers
            .iter()
            .find(|&&(ref k, _)| k.eq_ignore_ascii_case("Content-Type"))
            .map(|&(_, ref v)| &v[..])
            .unwrap();

    content_type.contains("text/htmd")
}

fn accepts_htmd_mime_type(request: &Request) -> bool {
    let accept_header = request.header("Accept").unwrap_or("*/*");
    accept_header.contains("text/htmd")
}

fn set_server_header(response: Response) -> Response {
    response.with_unique_header("Server", "waiter (Rust)")
}
