use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use failure::Error;
use mio::{Events, Poll, Interest, Token};

pub struct WebServer {
    listening_socket: TcpListener,
    connections: HashMap<usize, TcpStream>,
    next_connection_id: usize,
}

const SERVER: Token = Token(0);

impl WebServer {
    pub fn new(addr: &str) -> Result<Self, Error>{
        let address: String = addr.parse().unwrap();
        let listening_socket = TcpListener::bind(&address).unwrap();
        return Ok(WebServer{
            listening_socket: listening_socket,
            connections: HashMap::new(),
            next_connection_id: 1,
        });
    }

    pub fn run(&mut self) -> Result<&str, &str> {
        let poller = Poll::new().unwrap();
        poller.registry().register(
            &self.listening_socket,
            SERVER,
            Interest::READABLE,
        )?;
        let mut events = Events::with_capacity(1024);
        let mut response = Vec::new();
        return Ok("dummy");
    }
}