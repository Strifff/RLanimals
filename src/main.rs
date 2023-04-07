use std::{thread, time::Duration, collections::HashMap};

mod server;
mod beast_traits;
mod herbivore;
mod conc;

use std::sync::{Arc, Mutex,mpsc};
use crate::mpsc::{Sender,Receiver};

use herbivore::Herbivore;
use crate::beast_traits::Beast;
use crate::conc::{Msg, BeastUpdate};
use arc_swap::ArcSwap;

const FPS: i32 = 1;
const DELAY: i32 = 1000/FPS;
const MAPSIZE: i32 = 100;
const FOV: i32 = 90;

//static world_arc;

fn main(){

    // init world:
    // ID -> State
    let mut world: HashMap<String, (String, (f64, f64), i32, f64, Sender<BeastUpdate>)> = HashMap::new();
    // pos -> state
    let mut world_reverse: Vec<((f64, f64), String, String, i32, f64, Sender<BeastUpdate>)>  = Vec::new();
    //let mut world_rev_temp: Vec<((f64, f64), String, String, i32, f64)>  = Vec::new();

    //let world_reverse = Arc::new(world_reverse);

    //let world_arc = ArcSwap::from(Arc::new(world_reverse.clone()));

    //let world_mutex = Arc::new(Mutex::new(world));
    
    //start server
    thread::spawn(|| {server::main()});

    //todo world with 2x capacity

    let (tx, rx) = mpsc::channel::<Msg>();

    println!("tx: {:?}", tx);

    let b1 = Herbivore::new(
        "test".to_owned(),
        (50.0,50.0), 
        FOV,
        2.0, 
        MAPSIZE,
        tx.clone(),
        //&world_arc
    );

    thread::spawn(move || {herbivore::main(b1, DELAY)}); 

    let b2 = Herbivore::new(
        "test2".to_owned(),
        (50.0,50.0), 
        FOV,
        1.0, 
        MAPSIZE,
        tx.clone(),
    );

    thread::spawn(move || {herbivore::main(b2, DELAY)});

    'update: loop{
        // receive beast states
        let received = &rx;
 
        for msg in received.try_iter() {
            world.insert(msg.id, (msg.beast, msg.pos, msg.dir, msg.speed, msg.handle));
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
        //world_reverse = world_rev_temp.clone();

        //world_arc.store(Arc::new(world_reverse.clone()));



        /*for e in world_arc.load().iter() {
            //println!{"Entry: {:?}", e};
        }*/

        // share world

        for k in world.keys() {
            let entry = world.get(k).unwrap();
            let handle = (entry.4).clone();
            let msg = BeastUpdate {
                kill: false,
                world: world_reverse.clone()
            };
            handle.send(msg).unwrap();
        }



        // delay
        thread::sleep(Duration::from_millis(DELAY.try_into().unwrap()));

    }
}

fn import_beast(b: &impl Beast) {
    //println!("test");
    println!("id: {:?}, pos: {:?}", b.get_id(), b.get_pos());
}

