use std::{thread};

mod server;
mod beast_traits;
mod herbivore;

//use beast_traits::Beast;
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
    /*thread::spawn(||{
        for i in 1..10{
            println!("{} from thread", i);
            thread::sleep(Duration::from_millis(1));
        }
    });

    for i in 1..25{
        println!("{} from main", i);
        thread::sleep(Duration::from_millis(2));
    }*/

    //thread::spawn(|| {herbivore::main("1".to_owned())});

    let b1 = Herbivore::new("test".to_owned(), (50.0,50.0), FOV, MAPSIZE);
    thread::spawn(|| {herbivore::main(b1, DELAY)}); 

    //let b2 = Herbivore::new("test2".to_owned(), (2.2,2.2), FOV, MAPSIZE);
    //thread::spawn(|| {herbivore::main(b2, DELAY)});



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

