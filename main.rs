use std::{thread};

mod server;
mod beast_traits;
mod herbivore;

use herbivore::Herbivore;

const FPS: i32 = 10;
const DELAY: i32 = 1000/FPS;
const MAPSIZE: i32 = 100;
const FOV: i32 = 90;

fn main(){

    //init
    //let vb: Vec<Box<dyn Beast>> = Vec::new(); 
    


    //start server
    thread::spawn(|| {server::main()});

    let b1 = Herbivore::new("test".to_owned(), (50.0,50.0), FOV, MAPSIZE);
    thread::spawn(|| {herbivore::main(b1, DELAY)}); 

    //loop
    loop{

        //init


        //spawn hervivores


        //spawn carnivores


        //simulate


        //feedback


        //save
    }
}

/*fn take_beast(b: &dyn Beast) {
    //let vb::push(b);
}*/

