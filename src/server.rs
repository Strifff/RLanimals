use serde_json::{json, Value};
use std::io::{prelude::*};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::time::Duration;
use std::{fs, thread};

//use std::time::Duration;

//use json::object;
//use serde_json::{Map, Value};

use crate::conc::{BeastUpdate, MainServer};
use crate::mpsc::Sender;

pub struct Server {
    mapsize: i32,
    main_handle: Sender<MainServer>,
}

impl Server {
    pub fn new(mapsize: i32, main_handle: Sender<MainServer>) -> Server {
        Server {
            mapsize: mapsize,
            main_handle: main_handle,
        }
    }
}

pub fn main(server: Server, delay: i32) {
    // init main with server
    let (server_tx, server_rx) = mpsc::channel::<MainServer>();
    let world_empty: Vec<((f64, f64), String, String, i32, i32, i32, f64, Sender<BeastUpdate>)> = Vec::new();
    let msg = MainServer {
        msg_type: "main update".to_owned(),
        msg_data: 2,
        handle_send: server_tx.clone(),
        world: None,
        entries: 0,
    };

    let _ = server.main_handle.send(msg);

    thread::spawn(|| server_loop(server));
    
    let mut world = {
        let input = std::fs::read_to_string("src/webpages/world_copy.json").unwrap();
        serde_json::from_str::<Value>(&input).unwrap()
    };
    //let world_blanc = world.clone();

    // loop
    loop {
        let received = &server_rx;
        for msg in received.try_iter() {

            //world = world_blanc.clone();

            //println!("SERVER RECEIVED: {:?} messages", msg.entries);
            let mut entry_vec: Vec<Value> = Vec::new();
            for entry in msg.world.unwrap() {
                let entry_json = json!({
                    "beast":  entry.2,
                    "pos_x":  entry.0.0,
                    "pos_y":  entry.0.1,
                    "dir":    entry.3,
                    "fov":    entry.4,
                    "sight_range": entry.5,
                    "speed":  entry.6,
                });
                entry_vec.push(entry_json);

            }
            //todo add fields for hv, cv and plant
            world["entries"] = Value::Array(entry_vec);
           
        }

        std::fs::write(
            "src/webpages/world.json",
            serde_json::to_string_pretty(&world).unwrap(),
        ).unwrap();

        //delay
        thread::sleep(Duration::from_millis(delay.try_into().unwrap()));
    }
}

fn server_loop(server: Server) {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    //println!("handeling connection");
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(o) => o,
        Err(_) => return,
    };

    //println!("[RECEIVED]: {:?}", String::from_utf8_lossy(&buffer));
    let get_index = b"GET / HTTP/1.1\r\n";
    let get_js = b"GET /index.js HTTP/1.1\r\n";
    let perform_calc = b"GET /calc-new-state HTTP/1.1\r\n";
    let graph = b"GET /graph HTTP/1.1\r\n";
    let update = b"update world \r\n";

    let (status_line, filename, content_type) = if buffer.starts_with(get_index) {
        ("HTTP/1.1 200 OK", "index.html", "text/html")
    } else if buffer.starts_with(get_js) {
        ("HTTP/1.1 200 OK", "index.js", "text/html")
    } else if buffer.starts_with(perform_calc) {
        ("HTTP/1.1 200 OK", "world.json", "application/json")
    } else if buffer.starts_with(graph) {
        ("HTTP/1.1 200 OK", "index.html", "text/html")
    } else if buffer.starts_with(update) {
        ("HTTP/1.1 200 OK", "index.html", "text/html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html", "text/html")
    };

    if content_type.to_owned() == "application/json" {
        let path = format!("src/webpages/{}", filename);
        let contents = { read_json_loop(path, 1)
            /*let input = std::fs::read_to_string(path).unwrap();
            match serde_json::from_str::<Value>(&input) {
                Ok(o) => {o}
                Err(e) => {
                    println!("Error readin json: {:?}", e); 
                    let input = std::fs::read_to_string("src/webpages/world_copy.json").unwrap();
                    serde_json::from_str::<Value>(&input).unwrap()
                }
            }*/
        };

        let _ = serde_json::to_writer(stream, &contents);

    } else {
        let path = format!("src/webpages/{}", filename);
        let contents = fs::read_to_string(path).unwrap();
        let response: String =
        format!(
            "{} \r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n{}",
            status_line,
            contents.len(),
            content_type.to_owned(),
            contents
        );
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

}


fn read_json_loop(path: String, iteration: i32) -> Value {
    let input = std::fs::read_to_string(&path).unwrap();
    match serde_json::from_str::<Value>(&input) {
        Ok(o) => {o}
        Err(e) => {
            if iteration > 10 {
                //println!("Tried to read json {:?} times, error: {:?}", iteration, e);
            }
            if iteration > 30 {
                println!("Returned empty world");
                let input_empty = std::fs::read_to_string("src/webpages/world_copy.json").unwrap();
                return serde_json::from_str::<Value>(&input_empty).unwrap();
            }
            read_json_loop(path, iteration+1)
        }
    }
}