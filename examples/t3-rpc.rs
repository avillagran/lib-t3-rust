//! Minimal operator CLI: one T3 RPC over the shared transport.
//!
//! ```sh
//! T3_TOKEN=$(t3 auth session issue --token-only --ttl 10m) \
//!   cargo run --example t3-rpc -- http://127.0.0.1:3773 server.getConfig '{}'
//! ```
//! The token comes from the environment, never from the command line, so it
//! stays out of shell history and process listings.
use std::io::Read;
use t3_protocol::{Connection, Credential};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: t3-rpc <http-base-url> <rpc.tag> [payload-json | - for stdin]");
        std::process::exit(2);
    }
    let token = std::env::var("T3_TOKEN").unwrap_or_default();
    let payload = match args.get(3).map(String::as_str) {
        None => "{}".to_string(),
        Some("-") => {
            let mut raw = String::new();
            std::io::stdin().read_to_string(&mut raw).expect("stdin");
            raw
        }
        Some(inline) => inline.to_string(),
    };
    let payload: serde_json::Value = serde_json::from_str(&payload).expect("payload must be JSON");
    let result = Credential::bearer(&token)
        .and_then(|credential| Connection::new(&args[1], None, credential))
        .and_then(|connection| connection.rpc(&args[2], payload));
    match result {
        Ok(value) => println!("{}", serde_json::to_string_pretty(&value).unwrap_or_default()),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}
