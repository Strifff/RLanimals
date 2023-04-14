use std::{thread, time::Duration, collections::HashMap};

mod server;
mod beast_traits;
mod herbivore;
mod conc;

use std::sync::{/*Arc, Mutex,*/mpsc};
use crate::mpsc::{Sender/*,Receiver*/};

use conc::{MainServer};
use server::Server;
use herbivore::Herbivore;
//use crate::beast_traits::Beast;
use crate::conc::{Msg, BeastUpdate};
use rand::Rng;

use nanoid::nanoid;

const FPS: i32 = 10;
const DELAY: i32 = 1000/FPS;
const MAPSIZE: i32 = 500;
const FOV: i32 = 90;
const N_HERB: i32 = 20;

fn main(){

    // init
    let mut rng = rand::thread_rng();
    
    // world: ID -> State
    let mut world: HashMap<String, (String, (f64, f64), i32, i32, f64, Sender<BeastUpdate>)> = HashMap::new();
    // world: pos -> state
    let mut world_reverse: Vec<((f64, f64), String, String, i32, i32, f64, Sender<BeastUpdate>)>  = Vec::new();
  
    //start server
    let (server_tx, server_rx) = mpsc::channel::<MainServer>();
    let server = Server::new(MAPSIZE, server_tx.clone());
    thread::spawn(move || {server::main(server, DELAY)});
    let mut server_handle = server_tx.clone();
    let server_recv = &server_rx;
    if let Ok(msg) = server_recv.recv() {
        server_handle = msg.handle_send.clone();
        //println!("Msg value: {:?}", msg.msg_data);
    }

    // todo world with 2x capacity

    // mailbox
    let (tx, rx) = mpsc::channel::<Msg>();

    // spawn Herbivores //todo make fucntion
    for _ in 1..=N_HERB {
        let id = nanoid!();
        println!("Spawned: {:?}", id);
        let pos: (f64, f64) = (rng.gen_range(0.0..MAPSIZE as f64), 
                                rng.gen_range(0.0..MAPSIZE as f64));

        let h = Herbivore::new(
            id,
            pos, 
            FOV,
            rng.gen_range(1.0..3.0), 
            MAPSIZE,
            tx.clone(),
        );
        thread::spawn(move || {herbivore::main(h, DELAY)});
    }

    loop{
        // receive beast states
        let received = &rx;
        for msg in received.try_iter() {
            if msg.alive {
                world.insert(msg.id, (msg.beast, msg.pos, msg.dir, msg.fov, msg.speed, msg.handle));
            } else {
                let _ = world.remove(&msg.id);
            }
        }

        // reciver updates from server
        let received = &server_rx;
        for msg in received.try_iter() {
            println!("main received from server");
        }


        // update world
        world_reverse.clear();
        for k in world.keys() {
            let entry = world.get(k).unwrap();
            let id = k.clone();
            let beast = entry.0.clone();
            let handle = entry.5.clone();
            world_reverse.push((entry.1, id, beast, entry.2, entry.3, entry.4, handle));
        }

        // share world with beasts
        for k in world.keys() {
            let entry = world.get(k).unwrap();
            let handle = (entry.5).clone();
            let msg = BeastUpdate {
                kill: false,
                world: world_reverse.clone()
            };
            match handle.send(msg) {
                Ok(_) => {
                    //everything is fine
                }
                Err(_) => { //thread proably dead
                    //println!("send error-------------------------------------------------");
                }
            }
        }

        // update server
        let entries = (world_reverse.len()) as i32;
        let msg = MainServer{
            msg_type: "test test".to_owned(),
            msg_data: 1, //random data for now
            handle_send: server_tx.clone(),
            world: world_reverse.clone(),
            entries: entries,
        };

        let _ = server_handle.send(msg);

        // delay
        thread::sleep(Duration::from_millis(DELAY.try_into().unwrap()));

    }
}
