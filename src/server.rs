use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::io::{prelude::*, BufWriter};
use std::process;
use std::{fs, thread, process::Command};
use std::sync::mpsc;
use serde_json::{Result, Value, json, Map};
use rand::Rng;

//use std::time::Duration;

//use json::object;
//use serde_json::{Map, Value};

use crate::conc::{MainServer, BeastUpdate};
use crate::mpsc::{Sender/*,Receiver*/};

pub struct Server {
    mapsize: i32,
    main_handle: Sender<MainServer>,
}

impl Server {
    pub fn new(mapsize: i32, main_handle: Sender<MainServer>) -> Server {
        Server {
            mapsize: mapsize,
            main_handle: main_handle
        }
    }
}

pub fn main(server: Server){
    // init main with server
    let (server_tx, server_rx) = mpsc::channel::<MainServer>();
    let world_empty: Vec<((f64, f64), String, String, i32, f64, Sender<BeastUpdate>)>  = Vec::new();
    let msg = MainServer {
        msg_type: "main update".to_owned(),
        msg_data: 2,
        handle_send: server_tx.clone(),
        world: world_empty,
        entries: 0,
    };

    let mut rng = rand::thread_rng();

    let _ = server.main_handle.send(msg);
    
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    //needed to break listener loop
    listener.set_nonblocking(true).expect("Cannot set non-blocking");

    let mut world_json = Map::new();

    // loop
    loop {
        let received = &server_rx;
        for msg in received.try_iter() {
            println!("{:?}", msg.world.len());

            if msg.entries > 0 {
                println!("cleared");
                let _ = &world_json.clear();
            } else {
                println!("exit");
                process::exit(1);
            }

            println!("SERVER RECEIVED: {:?} messages", msg.entries);
            for entry in msg.world {
                let entry_json = json!({
                    "beast":  entry.2,
                    "pos_x":  entry.0.0,
                    "pos_y":  entry.0.1,
                    "dir":    entry.3,
                    "speed":  entry.4,
                });
                println!("id: {:?}, state: {:?}", &entry.1, &entry_json);
                world_json.insert(entry.1, entry_json);
            }
        }
        //write to file
        let file = File::create("src/webpages/world.json");
        let mut writer = BufWriter::new(file.unwrap());
        let _ = serde_json::to_writer(&mut writer, &world_json);
        writer.flush();

        let x = rng.gen_range(1..1000);

        //poll_mailbox
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    // do something with the TcpStream
                    handle_connection(s);
            
                }
                Err(ref e) /*if e.kind() == io::ErrorKind::WouldBlock*/ => {
                    // Decide if we should exit
                    break;
                    // Decide if we should try to accept a connection again
                    //continue;
                }
                Err(_) => {
                    panic!("encountered IO error");
                }
               // _ => panic!("encountered IO error: {}", e),
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream){
    println!("handeling connection");
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(o) => {o}
        Err(_) => {return}
    };

    //println!("[RECEIVED]: {:?}", String::from_utf8_lossy(&buffer));
    let get_index = b"GET / HTTP/1.1\r\n";
    let get_js = b"GET /index.js HTTP/1.1\r\n";
    //let perform_calc = b"GET /calc-new-state HTTP/1.1\r\n";
    let graph = b"GET /graph HTTP/1.1\r\n";

    let (status_line, filename, content_type) =
    if buffer.starts_with(get_index){
        ("HTTP/1.1 200 OK", "index.html", "text/html")
    } else if buffer.starts_with(get_js){
        ("HTTP/1.1 200 OK", "index.js", "text/html")
    } /*else if buffer.starts_with(perform_calc){
        ("HTTP/1.1 200 OK", "test.json", "application/json")
    }*/ else if buffer.starts_with(graph) {
        ("HTTP/1.1 200 OK", "index.html", "text/html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html", "text/html")
    };

    let path = format!("src/webpages/{}",filename);
    println!("path: {:?}", path);
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