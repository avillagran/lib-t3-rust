//! Local HTTP fixture: answers each connection with the next canned response
//! and returns every request (head and body) it saw. Never a real server.
#![allow(dead_code)]
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread::{self, JoinHandle},
};
pub struct Seen {
    pub head: String,
    pub body: String,
}
impl Seen {
    pub fn header(&self, name: &str) -> Option<String> {
        self.head.lines().skip(1).find_map(|l| {
            let (k, v) = l.split_once(':')?;
            k.trim().eq_ignore_ascii_case(name).then(|| v.trim().to_string())
        })
    }
}
pub fn read_request(s: &mut TcpStream) -> Seen {
    let mut head = vec![];
    let mut b = [0];
    while !head.ends_with(b"\r\n\r\n") {
        if s.read(&mut b).unwrap_or(0) == 0 {
            break;
        }
        head.push(b[0]);
    }
    let head = String::from_utf8(head).unwrap();
    let len = head
        .lines()
        .find_map(|l| {
            let (k, v) = l.split_once(':')?;
            k.eq_ignore_ascii_case("content-length").then(|| v.trim().parse::<usize>().ok())?
        })
        .unwrap_or(0);
    let mut body = vec![0; len];
    s.read_exact(&mut body).unwrap();
    Seen { head, body: String::from_utf8(body).unwrap() }
}
pub fn response(status: &str, headers: &[(&str, &str)], body: &[u8]) -> Vec<u8> {
    let mut out = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n", body.len());
    for (k, v) in headers {
        out.push_str(&format!("{k}: {v}\r\n"));
    }
    out.push_str("\r\n");
    let mut out = out.into_bytes();
    out.extend_from_slice(body);
    out
}
pub fn json(body: &str) -> Vec<u8> {
    response("200 OK", &[("Content-Type", "application/json")], body.as_bytes())
}
/// Serves `answers` in order, one connection each.
pub fn serve(answers: Vec<Vec<u8>>) -> (String, JoinHandle<Vec<Seen>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let mut seen = vec![];
        for answer in answers {
            let (mut s, _) = listener.accept().unwrap();
            seen.push(read_request(&mut s));
            let _ = s.write_all(&answer);
        }
        seen
    });
    (base, server)
}
