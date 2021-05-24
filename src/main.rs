use std::env;
use std::process;

mod server;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Wrong number of arguments");
        process::exit(1);
    }
    let mut server = server::WebServer::new(&args[1]).unwrap_or_else(|e| {
        println!("{}", e);
        panic!();
    });
    server.run().unwrap_or_else(|e| {
        println!("{}", e);
        panic!();
    });
}
