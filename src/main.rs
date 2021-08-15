use tokio_native_tls::TlsStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use url::Url;

use std::io::Write;
use std::error::Error;
use std::fs::{self, OpenOptions};

use gemini_hacking as gh;
use gh::response_codes::*;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const SHOUTBOX: &str = "shoutbox.gmi";

#[tokio::main]
async fn main() -> Result<()> {
    let acceptor = gh::create_tls_acceptor()?;
    let listener = gh::create_tcp_listener().await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let stream = acceptor.accept(stream).await?;
        handle_request(stream).await?;
    }
}

async fn handle_request(mut stream: TlsStream<TcpStream>) -> Result<()> {
    let mut request = Vec::with_capacity(gh::MAX_HEADER_LENGTH);
    stream.read_buf(&mut request).await?;

    let request = String::from_utf8(request)?;
    eprintln!("[*] Request: {:?}", request);

    let url = Url::parse(&request)?;

    let response = match url.path() {
        "/shoutbox" => { render_shoutbox() }, 
        "/shout" => { shout(&url) },
        _ => { redirect_to_shoutbox() },
    };

    stream.write_all(&response).await?;
    
    Ok(())
}

/// show shoutbox and include link to shout
fn render_shoutbox() -> Vec<u8> {
    let shoutbox = fs::read_to_string(SHOUTBOX)
        .expect("in render_shoutbox(): failed to read shoutbox.gmi");
    gh::create_response(SUCCESS, Some("text/gemini"), Some(&shoutbox))
}

/// shout if there is a msg or redirect to shoutbox
fn shout(request: &Url) -> Vec<u8> {
    match request.query() {
        None => gh::create_response(INPUT, Some("Shout something!"), None),
        Some(message) => {
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(SHOUTBOX)
                // BAD. Maybe return a gemini error to client when it fails
                .expect("in shout(): failed to open shoutbox file"); 
            file.write(format!("{}\n", message).as_bytes())
                .expect("in shout(): failed to write to shoutbox file");
            gh::create_response(SUCCESS, Some("/shoutbox"), None)
        }
    }
}

fn redirect_to_shoutbox() -> Vec<u8> {
    gh::create_response(REDIRECT_PERMANENT, Some("/shoutbox"), None)
}