use std::{thread};

mod server;
mod beast_traits;
mod herbivore;

use herbivore::Herbivore;
use crate::beast_traits::Beast;

const FPS: i32 = 10;
const DELAY: i32 = 1000/FPS;
const MAPSIZE: i32 = 100;
const FOV: i32 = 90;

fn main(){

    //init
    //let mut vb: Vec<Box<dyn Beast>> = Vec::new(); 
    


    //start server
    thread::spawn(|| {server::main()});

    let b1 = Herbivore::new("test".to_owned(), (50.0,50.0), FOV, 1.0, MAPSIZE);
    thread::spawn(|| {herbivore::main(b1, DELAY)}); 

    //vb.push(Box::new(b1));
    //loop
    loop{

        //init


        //spawn herbivores


        //spawn carnivores


        //simulate


        //feedback


        //save
    }
}

fn take_beast(b: &impl Beast) {
    //println!("test");
    println!("id: {:?}, pos: {:?}", b.get_id(), b.get_pos());
}

