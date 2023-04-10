use std::{thread, time::Duration, collections::HashMap};

mod server;
mod beast_traits;
mod herbivore;
mod conc;

use std::sync::{/*Arc, Mutex,*/mpsc};
use crate::mpsc::{Sender/*,Receiver*/};

use herbivore::Herbivore;
use crate::beast_traits::Beast;
use crate::conc::{Msg, BeastUpdate};
use rand::Rng;

use nanoid::nanoid;

const FPS: i32 = 10;
const DELAY: i32 = 1000/FPS;
const MAPSIZE: i32 = 100;
const FOV: i32 = 90;
const N_HERB: i32 = 3;

fn main(){

    // init
    let mut rng = rand::thread_rng();
    
    // world: ID -> State
    let mut world: HashMap<String, (String, (f64, f64), i32, f64, Sender<BeastUpdate>)> = HashMap::new();
    // world: pos -> state
    let mut world_reverse: Vec<((f64, f64), String, String, i32, f64, Sender<BeastUpdate>)>  = Vec::new();
  
    //start server
    thread::spawn(|| {server::main()});

    // todo world with 2x capacity

    // mailbox
    let (tx, rx) = mpsc::channel::<Msg>();

    // spawn Herbivores //todo make fucntion
    for _ in 0..=N_HERB {
        let id = nanoid!();
        let pos: (f64, f64) = (rng.gen_range(0.0..MAPSIZE as f64), 
                                rng.gen_range(0.0..MAPSIZE as f64));

        let h = Herbivore::new(
            id,
            pos, 
            FOV,
            2.0, 
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
                world.insert(msg.id, (msg.beast, msg.pos, msg.dir, msg.speed, msg.handle));
            } else {
                let _ = world.remove(&msg.id);
            }
        }

        // update world
        world_reverse.clear();
        for k in world.keys() {
            let entry = world.get(k).unwrap();
            let id = k.clone();
            let beast = entry.0.clone();
            let handle = entry.4.clone();
            world_reverse.push((entry.1, id, beast, entry.2, entry.3, handle));
        }

        // share world
        for k in world.keys() {
            let entry = world.get(k).unwrap();
            let handle = (entry.4).clone();
            let msg = BeastUpdate {
                kill: false,
                world: world_reverse.clone()
            };
            let _ = handle.send(msg).unwrap();
        }

        // plot
        plotter(&world);

        // delay
        thread::sleep(Duration::from_millis(DELAY.try_into().unwrap()));

    }
}

fn plotter(_: &HashMap<String, (String, (f64, f64), i32, f64, Sender<BeastUpdate>)>) {
    
    println!("plot test");
}

