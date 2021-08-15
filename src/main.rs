use std::error::Error;
use tokio_native_tls::TlsStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use url::Url;

use std::io::Write;
use std::fs::{self, OpenOptions};

use gemini_hacking as gh;

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
    let mut request = Vec::with_capacity(1026);
    stream.read_buf(&mut request).await?;

    let request = String::from_utf8(request)?;
    println!("Request: {:?}", request);

    let url = Url::parse(&request)?;

    let response = match url.path() {
        // show shoutbox and include link to shout
        "/shoutbox" => { render_shoutbox() }, 
        // shout if no msg or redirect to shoutbox
        "/shout" => { shout(&url) },
        // redir to /shoutbox
        _ => { redirect("/shoutbox") } 
    };

    stream.write_all(&response).await?;
    
    Ok(())
}

fn render_shoutbox() -> Vec<u8> {
    let shoutbox = fs::read_to_string(SHOUTBOX)
    .expect("[render_shoutbox()] failed to read shoutbox.gmi");
    gh::create_response(20, Some("text/gemini"), Some(&shoutbox))
}

fn shout(request: &Url) -> Vec<u8> {
    match request.query() {
        None => gh::create_response(10, Some("Shout something!"), None),
        Some(message) => {
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(SHOUTBOX)
                // BAD. Maybe return a gemini error to client when it fails
                .expect("failed to open shoutbox file in append mode"); 
            file.write(format!("{}\n", message).as_bytes())
                .expect("[shout()] failed to write to shoutbox file");
            gh::create_response(31, Some("/shoutbox"), None)
        }
    }
}

fn redirect(location: &str) -> Vec<u8> {
    gh::create_response(31, Some(location), None)
}