use std::{thread};


mod server;
mod herbivore;


fn main(){

    //start server
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
    thread::spawn(|| {server::main()});

    thread::spawn(|| {herbivore::main()});

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

