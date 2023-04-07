use std::{thread};

mod server;
mod beast_traits;
mod herbivore;
mod conc;

use std::sync::mpsc;

use herbivore::Herbivore;
use crate::beast_traits::Beast;
use crate::conc::Msg;

const FPS: i32 = 10;
const DELAY: i32 = 1000/FPS;
const MAPSIZE: i32 = 100;
const FOV: i32 = 90;

fn main(){

    //init
    //let mut vb: Vec<Box<dyn Beast>> = Vec::new(); 
    
    //start server
    thread::spawn(|| {server::main()});

    //todo world with 2x capacity

    let (tx, rx) = mpsc::channel::<Msg>();

    let tx_clone = tx.clone();

    let b1 = Herbivore::new(
        "test".to_owned(),
        (50.0,50.0), 
        FOV,
        2.0, 
        MAPSIZE,
        tx_clone,
    );

    thread::spawn(|| {herbivore::main(b1, DELAY)}); 

    //vb.push(Box::new(b1));
    //loop
    loop{
        // receive beast states
        for received in &rx {
            println!("received from {:?}", received.id);
        }

        // update world

        // share world

    }
}

fn import_beast(b: &impl Beast) {
    //println!("test");
    println!("id: {:?}, pos: {:?}", b.get_id(), b.get_pos());
}

