use std::net::{TcpListener, TcpStream};
use std::io::{prelude::*, Empty};
use std::fs;
use std::sync::mpsc;

use tracing_subscriber::fmt::format;

use crate::conc::{Main_Server, BeastUpdate};
use crate::mpsc::{Sender/*,Receiver*/};

pub struct Server {
    mapsize: i32,
    main_handle: Sender<Main_Server>,
}

impl Server {
    pub fn new(mapsize: i32, main_handle: Sender<Main_Server>) -> Server {
        Server {
            mapsize: mapsize,
            main_handle: main_handle
        }
    }
}


pub fn main(mut server: Server){
    // init main with server
    let (server_tx, server_rx) = mpsc::channel::<Main_Server>();
    let mut world_empty: Vec<((f64, f64), String, String, i32, f64, Sender<BeastUpdate>)>  = Vec::new();
    let msg = Main_Server {
        msg_type: "main update".to_owned(),
        msg_data: 2,
        handle_send: server_tx.clone(),
        world: world_empty,
    };

    let _ = server.main_handle.send(msg);
    
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // loop
    loop {
        let received = &server_rx;
        for msg in received.try_iter() {
            println!("SERVER RECEIVED:");
            for entry in msg.world {
                println!("Entry: {:?}", entry);
            }
        }

        for stream in listener.incoming().into_iter(){
            let stream = stream.set_nonblocking(true).unwrap();
    
            handle_conenction(stream);
        }
    }
}

fn handle_conenction(mut stream: TcpStream){
    println!("handeling connection");
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    println!("[RECEIVED]: {:?}", String::from_utf8_lossy(&buffer));

    let get_index = b"GET / HTTP/1.1\r\n";
    let get_js = b"GET /index.js HTTP/1.1\r\n";
    let perform_calc = b"GET /calc-new-state HTTP/1.1\r\n";

    let (status_line, filename, content_type) =
    if buffer.starts_with(get_index){
        ("HTTP/1.1 200 OK", "index.html", "text/html")
    } else if buffer.starts_with(get_js){
        ("HTTP/1.1 200 OK", "index.js", "text/html")
    } else if buffer.starts_with(perform_calc){
        ("HTTP/1.1 200 OK", "test.json", "application/json")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html", "text/html")
    };

    let path = format!("src/webpages/{}",filename);
    let contents = fs::read_to_string(path).unwrap();

    let response = format!(
        "{} \r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        content_type,
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}