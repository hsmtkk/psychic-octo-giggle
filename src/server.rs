use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader,Read,Write};
use std::str;
use failure::Error;
use mio::{Events, Poll, Interest, Token};
use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use regex::Regex;

pub struct WebServer {
    listening_socket: TcpListener,
    connections: HashMap<usize, TcpStream>,
    next_connection_id: usize,
}

const SERVER: Token = Token(0);

impl WebServer {
    pub fn new(addr: &str) -> Result<Self, Error>{
        let address: String = addr.parse()?;
        let listening_socket = TcpListener::bind(address.parse()?)?;
        return Ok(WebServer{
            listening_socket: listening_socket,
            connections: HashMap::new(),
            next_connection_id: 1,
        });
    }

    pub fn run(&mut self) -> Result<&str, Error> {
        let mut poll = Poll::new()?;
        poll.registry().register(
            &mut self.listening_socket,
            SERVER,
            Interest::READABLE,
        )?;
        let mut events = Events::with_capacity(1024);
        let mut response = Vec::new();
        loop {
            match poll.poll(&mut events, None){
                Ok(_) => {},
                Err(e) => {
                    println!("{}", e);
                    continue;
                },
            }
            for event in &events {
                match event.token(){
                    SERVER => {
                        let(stream, remote) = match self.listening_socket.accept(){
                            Ok(t) => t,
                            Err(e) => {
                                println!("{}", e);
                                continue;
                            },
                        };
                        dbg!("Connection from {}", &remote);
                        self.register_connection(&poll, stream).unwrap_or_else(|e|{
                            println!("{}", e);
                        });
                    },
                    Token(conn_id) => {
                        self.http_handler(conn_id, event, &poll, &mut response)?;
                    },
                }
            }
        }
        return Ok("dummy");
    }

    fn register_connection(&mut self, poll:&Poll, mut stream:TcpStream) -> Result<(), Error>{
        let token = Token(self.next_connection_id);
        poll.registry().register(&mut stream, token, Interest::READABLE)?;
        if self.connections.insert(self.next_connection_id, stream).is_some(){
            println!("Connection ID is already exist.");
        }
        self.next_connection_id += 1;
        return Ok(());
    }

    fn http_handler(&mut self, conn_id:usize, event:&Event, poll:&Poll, response:&mut Vec<u8>) -> Result<(), Error>{
        let stream = self.connections.get_mut(&conn_id).ok_or_else(|| failure::err_msg("Failed to get connection."))?;
        if event.is_readable(){
            dbg!("readable conn_id: {}", conn_id);
            let mut buffer = [0u8; 1024];
            let nbytes = stream.read(&mut buffer)?;
            if nbytes != 0 {
                *response = make_response(&buffer[..nbytes])?;
                poll.registry().reregister(stream, Token(conn_id), Interest::WRITABLE)?;
            }else{
                self.connections.remove(&conn_id);
            } Ok(())
        } else if event.is_writable(){
            dbg!("writable conn_id: {}", conn_id);
            stream.write_all(response)?;
            self.connections.remove(&conn_id);
            Ok(())
        } else {
            Err(failure::err_msg("Undefined event."))
        }
    }

}

const WEBROOT: &str = "/webroot";

fn make_response(buffer: &[u8]) -> Result<Vec<u8>, Error>{
    let http_pattern = Regex::new(r"(.*) (.*) HTTP/1.([0-1])\r\n.*")?;
    let captures = match http_pattern.captures(str::from_utf8(buffer)?){
        Some(cap) => cap,
        None => {
            return create_msg_from_code(400, None);
        },
    };
    let method = captures[1].to_string();
    let path = format!("{}{}{}", env::current_dir()?.display(), WEBROOT, &captures[2]);
    let _version = captures[3].to_string();
    if method == "GET"{
        let file = match File::open(path){
            Ok(file) => file,
            Err(_) => {
                return create_msg_from_code(404, None);
            },
        };
        let mut reader = BufReader::new(file);
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        return create_msg_from_code(200, Some(buf));
    } else {
        return create_msg_from_code(501, None);
    }
}

fn create_msg_from_code(status_code:u16, msg:Option<Vec<u8>>) -> Result<Vec<u8>, Error>{
    match status_code{
        200 => {
            let mut header = "HTTP/1.0 200 OK\r\nServer: mio webserver\r\n\r\n".to_string().into_bytes();
            if let Some(mut msg) = msg{
                header.append(&mut msg);
            }
            return Ok(header);
        },
        400 => {
            return Ok("HTTP/1.0 400 Bad Request\r\nServer: mio webserver\r\n\r\n".to_string().into_bytes());
        },
        404 => {
            return Ok("HTTP/1.0 404 Not Found\r\nServer mio webserver\r\n\r\n".to_string().into_bytes());
        },
        _ => {
            return Err(failure::err_msg("Undefined status code."));
        }
    }
}