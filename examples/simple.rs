extern crate mio;

use mio::*;
use mio::net::{TcpListener, TcpStream};

fn main() {
    let _twili = nx::twili::Handle::new().unwrap().unwrap();

    println!("Making tokens and parsing address...");

    // Setup some tokens to allow us to identify which event is
    // for which socket.
    const SERVER: Token = Token(0);
    const CLIENT: Token = Token(1);

    let addr = "127.0.0.1:13265".parse().unwrap();

    println!("Starting listener");

    // Setup the server socket
    let server = TcpListener::bind(&addr).unwrap();

    println!("Creating Poll");

    // Create a poll instance
    let poll = Poll::new().unwrap();

    println!("Registering server");

    // Start listening for incoming connections
    poll.register(&server, SERVER, Ready::readable(),
                  PollOpt::edge()).unwrap();

    println!("Creating client");

    // Setup the client socket
    let sock = TcpStream::connect(&addr).unwrap();

    println!("Registering client");

    // Register the socket
    poll.register(&sock, CLIENT, Ready::readable(),
                  PollOpt::edge()).unwrap();

    println!("Creating Events storage");

    // Create storage for events
    let mut events = Events::with_capacity(1024);

    println!("Starting event loop...");

    loop {
        println!("Polling");

        poll.poll(&mut events, None).unwrap();

        println!("Got event");

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    println!("Accepting connection");

                    // Accept and drop the socket immediately, this will close
                    // the socket and notify the client of the EOF.
                    let _ = server.accept();

                    println!("Server received connection!");
                }
                CLIENT => {
                    println!("Client connected!");

                    // The server just shuts down the socket, let's just exit
                    // from our event loop.
                    return;
                }
                _ => unreachable!(),
            }
        }
    }
}
