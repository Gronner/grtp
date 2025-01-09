use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

mod thread_pool;

use thread_pool::ThreadPool;

macro_rules! serve_content {
    ( $path:literal, $content_type:literal ) => {
        (
            "HTTP/1.1 200 OK",
            concat!("content-type: ", $content_type),
            include_bytes!($path).as_ref(),
        )
    };
}

macro_rules! serve_page {
    ( $path:literal, $header:expr, $footer:expr ) => {{
        let (status_line, content_type_header, contents) =
            serve_content!($path, "text/html;charset=utf-8");
        (
            status_line,
            content_type_header,
            &[$header, contents, $footer].concat()[..],
        )
    }};
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let path = request_line.split(" ").nth(1).unwrap();
    const HEADER: &[u8; 738] = include_bytes!("../site/header.html");
    const FOOTER: &[u8; 35] = include_bytes!("../site/footer.html");

    let (status_line, content_type_header, contents) = match path {
        "/" => serve_page!("../site/index.html", HEADER, FOOTER),
        "/favicon.ico" => serve_content!("../site/favicon.ico", "image/x-icon"),
        "/style.css" => serve_content!("../site/style.css", "text/css;charset=utf-8"),
        _ => (
            "HTTP/1.1 404 NOT FOUND",
            "text/html;charset=utf-8",
            &[HEADER, include_bytes!("../site/404.html").as_ref(), FOOTER].concat()[..],
        ),
    };

    let length = contents.len();
    let response =
        format!("{status_line}\r\n{content_type_header}\r\nContent-Length: {length}\r\n\r\n");

    stream
        .write_all(&[response.as_bytes(), contents].concat())
        .unwrap();
}

fn main() {
    let port = std::env::args().nth(1).unwrap_or("8080".to_string());
    let listener = TcpListener::bind(format!("127.0.1:{port}")).unwrap();
    let pool = ThreadPool::new(6);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
    println!("Shutting down.");
}
