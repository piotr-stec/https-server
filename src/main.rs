use clap::Parser;
use openssl::pkey::PKey;
use reqwest::blocking::Client;
use rustls::{Certificate, ServerConfig, ServerConnection, StreamOwned};
use rustls_pemfile::Item;
use std::error::Error;
use std::{
    fs::File,
    io::{self, prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use std::net::SocketAddr;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

fn load_tls_config() -> Arc<ServerConfig> {
    let private_key_bytes = include_bytes!("../self_signed_certs/key.pem");
    let pkey = PKey::private_key_from_pem(private_key_bytes).expect("Failed to parse private key");

    let private_key = rustls::PrivateKey(
        pkey.private_key_to_der()
            .expect("Failed to encode private key"),
    );

    let certs = load_certs("self_signed_certs/cert.pem");

    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs.unwrap(), private_key)
        .expect("bad certificate/key");

    Arc::new(config)
}

fn load_certs(path: &str) -> Result<Vec<Certificate>, io::Error> {
    let cert_file = File::open(path)?;
    let mut cert_reader = BufReader::new(cert_file);

    let mut certs = Vec::new();
    while let Some(item) = rustls_pemfile::read_one(&mut cert_reader)? {
        if let Item::X509Certificate(cert) = item {
            certs.push(Certificate(cert.to_vec()));
        }
    }

    Ok(certs)
}

// fn handle_connection(stream: TcpStream, tls_config: Arc<ServerConfig>) {
//     match ServerConnection::new(tls_config) {
//         Ok(mut server_conn) => {
//             let mut tls_stream = StreamOwned::new(server_conn, stream);

//             // Read the HTTP request
//             let buf_reader = BufReader::new(&mut tls_stream);
//             let http_request = buf_reader
//                 .lines()
//                 .take_while(|line| match line {
//                     Ok(l) => !l.is_empty(),
//                     Err(_) => false,
//                 })
//                 .collect::<Result<Vec<_>, _>>();

//             match http_request {
//                 Ok(req) => {
//                     println!("Request: {:#?}", req);
//                 }
//                 Err(e) => {
//                     eprintln!("Failed to read request: {}", e);
//                 }
//             }

//             let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, HTTPS!";
//             if let Err(e) = tls_stream.write_all(response.as_bytes()) {
//                 eprintln!("Failed to write response: {}", e);
//             }

//             // if let Err(e) = tls_stream.flush() {
//             //     eprintln!("Failed to flush stream: {}", e);
//             // }

//             // tls_stream.conn.send_close_notify();

//             // if let Err(e) = tls_stream.flush() {
//             //     eprintln!("Failed to flush after close_notify: {}", e);
//             // }

//             // drop(tls_stream); // Dropping the stream to close the connection
//         }
//         Err(e) => {
//             eprintln!("Failed to create server connection: {}", e);
//         }
//     }
// }

fn handle_connection(
    stream: TcpStream,
    tls_config: Arc<ServerConfig>,
) -> Result<(), Box<dyn Error>> {
    let server_conn = ServerConnection::new(tls_config).unwrap();
    let mut tls_stream = StreamOwned::new(server_conn, stream);

    let buf_reader = BufReader::new(&mut tls_stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");

    let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, HTTPS!";
    tls_stream.write_all(response.as_bytes()).unwrap();

    let client = Client::new();

    let response = client.get("https://www.facebook.com").send()?.text()?;

    println!("Response from website: {}", response);

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let addr = SocketAddr::from(([127, 0, 0, 1], cli.port));

    let listener = TcpListener::bind(addr).unwrap();

    let tls_config = load_tls_config();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let _ = handle_connection(stream, tls_config.clone());
    }
}
