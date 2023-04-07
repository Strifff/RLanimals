use std::{thread, time::Duration, collections::HashMap};

mod server;
mod beast_traits;
mod herbivore;
mod conc;

use std::sync::{Arc, Mutex,mpsc};

use herbivore::Herbivore;
use crate::beast_traits::Beast;
use crate::conc::Msg;
use arc_swap::ArcSwap;

const FPS: i32 = 60;
const DELAY: i32 = 1000/FPS;
const MAPSIZE: i32 = 100;
const FOV: i32 = 90;

fn main(){

    // init world:
    // ID -> State
    let mut world: HashMap<String, (String, (f64, f64), i32, f64)> = HashMap::new();
    // pos -> state
    let mut world_reverse: Vec<((f64, f64), String, String, i32, f64)>  = Vec::new();
    let mut world_rev_temp: Vec<((f64, f64), String, String, i32, f64)>  = Vec::new();
    
    
    //start server
    thread::spawn(|| {server::main()});

    //todo world with 2x capacity

    let (tx, rx) = mpsc::channel::<Msg>();

    let b1 = Herbivore::new(
        "test".to_owned(),
        (50.0,50.0), 
        FOV,
        2.0, 
        MAPSIZE,
        tx.clone(),
        //&mut world_reverse,
    );

    thread::spawn(|| {herbivore::main(b1, DELAY)}); 

    /*let b2 = Herbivore::new(
        "test2".to_owned(),
        (50.0,50.0), 
        FOV,
        1.0, 
        MAPSIZE,
        tx.clone(),
    );

    thread::spawn(|| {herbivore::main(b2, DELAY)}); */

    'update: loop{
        // receive beast states
        let received = &rx;

        for msg in received.try_iter() {
            world.insert(msg.id, (msg.beast, msg.pos, msg.dir, msg.speed));
        }

        // update world
        world_rev_temp.clear();
        for k in world.keys() {
            let entry = world.get(k).unwrap();
            let id = k.clone();
            let beast = entry.0.clone();
            world_rev_temp.push((entry.1, id, beast, entry.2, entry.3 ));
        }
        world_reverse = world_rev_temp.clone();

        for e in world_reverse {
            println!{"Entry: {:?}", e};
        }

        // share world


        // delay
        thread::sleep(Duration::from_millis(DELAY.try_into().unwrap()));

    }
}

fn import_beast(b: &impl Beast) {
    //println!("test");
    println!("id: {:?}, pos: {:?}", b.get_id(), b.get_pos());
}

