mod server;
mod beast_traits;
mod herbivore;
mod conc;
mod plant;
mod A2C;

use std::fs::{self, ReadDir};
use std::path::PathBuf;
use std::{thread, time::Duration, time::SystemTime, sync::mpsc, collections::HashMap, process};
use tch::{nn, nn::Module, nn::OptimizerConfig, nn::VarStore, Tensor, Kind};
use rand::{Rng};
use nanoid::nanoid;

use conc::{MainServer};
use server::Server;
use herbivore::Herbivore;
use plant::Plant;
use A2C::ActorCritic;

use crate::conc::{Msg, BeastUpdate};
use crate::mpsc::{Sender/*,Receiver*/};

const FPS: i32 = 100;
const DELAY: i32 = 1000/FPS;
const MAPSIZE: i32 = 500;
const FOV: i32 = 50;
const N_HERB: i32 = 5;
const PLANT_FREQ: i32 = 1; //set value between 1..100

//NN parameters
const NN_RAYS: usize = 24;      // directions for the input of a beast, full circle
const NN_RAY_LEN: usize = 12;   // points per ray
const NN_RAY_DR: usize = 10;    // delta-radius for each point on ray
const N_TYPES: usize = 4;       // wall, plant, herbiv., carniv.
const N_STATES_SELF: usize = 2; // curr speed, energy

//math
const DEG_TO_RAD: f64 = 3.141593 / 180.0;
const RAD_TO_DEG: f64 = 180.0 / 3.141593;


const MAX_FILES: usize = 100;



fn main(){

    // init
    let mut rng = rand::thread_rng();
    
    // world: ID -> State
    let mut world: HashMap<String, (String, (f64, f64), i32, i32, i32, f64, Sender<BeastUpdate>)> = HashMap::new();
    // world: pos -> state
    let mut world_reverse: Vec<((f64, f64), String, String, i32, i32, i32, f64, Sender<BeastUpdate>)>  = Vec::new();
  
    //start server
    let (server_tx, server_rx) = mpsc::channel::<MainServer>();
    let server = Server::new(MAPSIZE, server_tx.clone());
    thread::spawn(move || {server::main(server, DELAY)});
    let mut server_handle = server_tx.clone();
    let server_recv = &server_rx;
    if let Ok(msg) = server_recv.recv() {
        server_handle = msg.handle_send.clone();
    }

    // mailbox
    let (tx, rx) = mpsc::channel::<Msg>();

    // nn weights
    let vs_herbi = VarStore::new(tch::Device::Cpu);    
    vs_herbi.save("src/nn/weights/herbi/herbi_ac").unwrap();
    let vs_carni = VarStore::new(tch::Device::Cpu);    
    vs_carni.save("src/nn/weights/carni/carni_ac").unwrap();

    let mut iteration = 0;
    'train_loop: loop {
        if iteration != 0 {
            //todo train networks
            //train
            train("Herbivore");


            //delete old states

        }

        // reset world
        world.clear();

        // spawn herbi and carni //todo inherit traits
        spawn_herbi(tx.clone());

        //todo spawn carni

        println!("Simulation started, iteration: {:?}", iteration);
        'sim_loop: loop {
            // receive beast/plant states
            let received = &rx;
            for msg in received.try_iter() {
                if msg.alive {
                    world.insert(msg.id, (msg.beast, msg.pos, msg.dir, msg.fov, msg.sight_range, msg.speed, msg.handle));
                } else {
                    //remove only dead
                    let _ = world.remove(&msg.id);
                    //check if both herbi and carni alive
                    let mut herbi: bool = false;
                    let mut carni: bool = true; //todo change to false
                    for key in world.keys() {
                        let entry = world.get(key).unwrap();
                        if entry.0 == "Herbivore" {herbi = true}
                        if entry.0 == "Carnivore" {carni = true}
                    }
                    if !herbi || !carni {
                        println!("Simulation ended");
                        break 'sim_loop
                    }
                }
            }

            // reciver updates from server
            let received = &server_rx;
            for msg in received.try_iter() {
                println!("main received from server");
                //todo when website has gui
            }

            // update world
            world_reverse.clear();
            for k in world.keys() {
                let entry = world.get(k).unwrap();
                let id = k.clone();
                let beast = entry.0.clone();
                let handle = entry.6.clone();
                world_reverse.push((entry.1, id, beast, entry.2, entry.3, entry.4, entry.5, handle));
            }

            // share world with beasts
            for k in world.keys() {
                let entry = world.get(k).unwrap();
                let handle = (entry.6).clone();
                let msg = BeastUpdate {
                    try_eat: false,
                    eat_result: false,
                    eat_value: 0,
                    response_handle: None,
                    world: Some(world_reverse.clone()),
                };
                if entry.0 != "Plant" {
                    match handle.send(msg) {
                        Ok(_) => { /*everything is fine*/ }
                        Err(_) => { /*thread probably dead*/ }
                    }
                }
            }

            // update server
            let entries = (world_reverse.len()) as i32;
            let msg = MainServer{
                msg_type: "test test".to_owned(),
                msg_data: 1, //random data for now
                handle_send: server_tx.clone(),
                world: Some(world_reverse.clone()),
                entries: entries,
            };

            let _ = server_handle.send(msg);

            // spawn more plants
            if rng.gen_range(1..100) <= PLANT_FREQ {
                spawn_plant(tx.clone());
            }

            // delay
            thread::sleep(Duration::from_millis(DELAY.try_into().unwrap()));
        }
        iteration += 1;
    }
}

fn spawn_plant(main_handle: Sender<Msg>) {
    let p = Plant::new(
        nanoid!(),
        MAPSIZE,
        main_handle,
    );
    thread::spawn(||plant::main(p));

}
fn spawn_herbi(main_handle: Sender<Msg>) {
    // spawn Herbivores 
    //todo inherit physical traits from best evolution
    for _ in 1..=N_HERB {
        let mut rng = rand::thread_rng();
        let h = Herbivore::new(
            nanoid!(),
            (rng.gen_range(0.0..MAPSIZE as f64),rng.gen_range(0.0..MAPSIZE as f64)),
            FOV,
            rng.gen_range(1.0..3.0),
            0, 
            main_handle.clone(),
        );
        thread::spawn(move || {herbivore::main(h)});
    }
}

fn train(beast: &str) {
    let mut vs = VarStore::new(tch::Device::Cpu);
    let mut samples: ReadDir; 
    if beast == "Herbivore" {
        vs.load("src/nn/weights/herbi/herbi_ac").unwrap();
        samples = fs::read_dir("src/nn/samples/herbi/").unwrap();
    } else if beast == "Carnivore" {
        vs.load("src/nn/weights/carni/carni_ac").unwrap();
        samples = fs::read_dir("src/nn/samples/carni/").unwrap();
    } else {
        println!("error in train");
        process::exit(1);
    }
    // discard old files
    let mut files_vec: Vec<(PathBuf, Duration)> = Vec::new();
    for sample in samples {
        let path = sample.as_ref().unwrap().path();
        let time = sample.as_ref().unwrap().metadata().unwrap().created().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap(); 
        if path.clone().into_os_string().into_string().unwrap().contains(".DS_Store") {continue}
        files_vec.push((path, time));
    }

    files_vec.sort_by(|(a,b), (c,d)| d.cmp(b));

    if files_vec.len() > MAX_FILES {
        for index in MAX_FILES..files_vec.len(){
            let (path, _) = &files_vec[index];
            let _ = fs::remove_file(path);
            files_vec.remove(index);
        }
    }
    
    let model = ActorCritic::new(
        &vs,
        (NN_RAYS*NN_RAY_LEN*N_TYPES + N_STATES_SELF) as i64,
        7
    );

    /*
    let (_, values) = model.forward(&Tensor::cat(&states, 0));
    let values = values.squeeze();
    let advantages = Tensor::cat(&rewards, 0) - values;
    let value_targets = Tensor::cat(&rewards, 0);
    

    // Compute actor and critic losses
    let log_probs = model
        .actor
        .forward(&Tensor::cat(&states, 0))
        .log_softmax(-1,Kind::Float);
    let actor_loss = -(advantages * log_probs.gather(-1, &Tensor::cat(&actions, 0), false)).mean(Kind::Float);
    let critic_loss = advantages.pow(&Tensor::of_slice(&[2])).mean(Kind::Float);
    let loss: Tensor = actor_loss;// + 0.5 * critic_loss;

    optimizer.zero_grad();
    loss.backward();
    optimizer.step();
    */
}