mod A2C;
mod beast_traits;
mod conc;
mod genAlg;
mod herbivore;
mod plant;
mod server;

extern crate tch;

use nanoid::nanoid;
use ndarray::Array2;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serde_json::Value;
use std::fs::{self, ReadDir};
use std::io::stdout;
use std::path::PathBuf;
use std::{collections::HashMap, process, sync::mpsc, thread, time::Duration, time::SystemTime, time::Instant};
use tch::nn::init;
use tch::Device;
use tch::{nn, nn::Module, nn::OptimizerConfig, nn::VarStore, Kind, Tensor};

use conc::MainServer;
use genAlg::{choose_parents, genAlgoNN, generate_offspring, init_models_ws_bs};
use herbivore::{add_border, distance_index, ray_direction_index, Herbivore};
use plant::Plant;
use server::Server;
use A2C::ActorCritic;

use crate::conc::{BeastUpdate, Msg};
use crate::mpsc::Sender;

const FPS: i32 = 100;
const DELAY: i32 = 1000 / FPS;
const MAPSIZE: i32 = 500;
const MARGIN: i32 = 5;
const FOV: i32 = 120;
const N_HERB: i32 = 5;
const PLANT_FREQ: i32 = 3; //set value between 1..100, 0 for no food
const PLANT_START: i32 = 5;
const ENERGY_MAX: f64 = 2500.0;
const CHILD_THRESH: i32 = 50;

//NN parameters
const NN_RAYS: usize = 24; // directions for the input of a beast, full circle
const NN_RAY_LEN: usize = 12; // points per ray
const NN_RAY_DR: usize = 10; // delta-radius for each point on ray
const N_TYPES: usize = 4; // wall, plant, herbiv., carniv.
const N_STATES_SELF: usize = 2; // curr speed, energy
const GAMMA: f64 = 0.98;
const LR: f64 = 0.0001;
const ACTIONS: usize = 7;

// genetic algorithm
const MAX_WEIGHT_BIAS: usize = 1;
const MUTATION_RATE: f64 = 0.01;
const START_SPARCITY: f64 = 0.25;
const CHOOSE_MAX_FIT: f64 = 0.8;

//math
const DEG_TO_RAD: f64 = 3.141593 / 180.0;
const RAD_TO_DEG: f64 = 180.0 / 3.141593;

const MAX_FILES: usize = 25;
const RETRAIN: bool = false; //& <------- IMPORTANT ---------

fn main() {
    // init
    let mut rng = rand::thread_rng();

    // world: ID -> State
    let mut world: HashMap<String, (String, (f64, f64), i32, i32, i32, f64, Sender<BeastUpdate>)> =
        HashMap::new();
    // world: pos -> state
    let mut world_reverse: Vec<(
        (f64, f64),
        String,
        String,
        i32,
        i32,
        i32,
        f64,
        Sender<BeastUpdate>,
    )> = Vec::new();

    //start server
    let (server_tx, server_rx) = mpsc::channel::<MainServer>();
    let server = Server::new(MAPSIZE, server_tx.clone());
    thread::spawn(move || server::main(server, DELAY));
    let mut server_handle = server_tx.clone();
    let server_recv = &server_rx;
    if let Ok(msg) = server_recv.recv() {
        server_handle = msg.handle_send.clone();
    }

    // mailbox
    let (tx, rx) = mpsc::channel::<Msg>();

    // nn weights
    if RETRAIN {
        let vs_herbi = VarStore::new(tch::Device::Cpu);
        vs_herbi.save("src/nn/weights/herbi/herbi_ac").unwrap();
        let vs_carni = VarStore::new(tch::Device::Cpu);
        vs_carni.save("src/nn/weights/carni/carni_ac").unwrap();
    }

    // weights and biases extraction test
    if true {
        //tch::manual_seed(1234);
        let mut vs = nn::VarStore::new(tch::Device::Cpu);
        //vs.load("src/nn/weights/herbi/herbi_ac").unwrap();
        match vs.load("src/nn/weights/herbi/herbi_ac") {
            Ok(_) => println!("Model loaded successfully!"),
            Err(err) => eprintln!("Error loading model: {}", err),
        }

        let test_model = ActorCritic::new(
            &vs,
            (NN_RAYS * NN_RAY_LEN * N_TYPES + N_STATES_SELF) as i64,
            7,
        );

        let zero_input: [f32; NN_RAY_LEN * NN_RAYS] = [0.0; NN_RAY_LEN * NN_RAYS];

        let zero_tensor = Tensor::of_slice(&zero_input);

        let wall_biases = test_model.wall.forward(&zero_tensor);

        let mut one_hot = zero_input;
        one_hot[0] = 1.0;

        let one_hot_tensor = Tensor::of_slice(&one_hot);

        let wall_weight = test_model.wall.forward(&one_hot_tensor);

        //wall_weight.print();

        let mut linear_layer = nn::linear(vs.root(), 5, 3, Default::default());

        // Define custom weights and biases (example values).
        let custom_weights_data = [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
        ];
        let custom_biases_data = [0.0, 0.0, 0.0];

        // Create tensors for custom weights and biases.
        let custom_weights: Tensor = Tensor::of_slice(&custom_weights_data).reshape(&[3, 5]);
        let custom_biases: Tensor = Tensor::of_slice(&custom_biases_data);

        // Set the custom weights and biases for the linear layer.
        //linear_layer.set_parameters(&nn::VarStore::new(Device::Cpu), &custom_weights, &custom_biases);
        linear_layer.bs = Some(custom_biases);
        linear_layer.ws = custom_weights;

        // Create an input tensor for inference (example input).
        let input_data: [f64; 5] = [0.0, 0.0, 0.0, 0.0, 1.0];
        let input = Tensor::of_slice(&input_data);

        // Perform inference using the modified linear layer.
        let output = linear_layer.forward(&input);

        // Print the output.
        println!("Output: {:?}", output);

        let inputs = ["plant"];

        //init_models_ws_bs(inputs, "herbi");

        let parents = choose_parents("herbi");

        //let offspring = generate_offspring(parents.0, parents.1, "herbi");

        let genalg_net = genAlg::genAlgoNN::new("reee".to_string());

        let input: [f32; NN_RAYS * NN_RAY_LEN] = [0.0; NN_RAYS * NN_RAY_LEN];

        
        let input_tensor = Tensor::of_slice(&input);

        println!("Input: {:?}", input_tensor);

        let start_time = Instant::now();
        
        let output = genalg_net.forward(&input_tensor);

        let elapsed = start_time.elapsed();
        let elapsed_us = elapsed.as_micros();

        println!("Elapsed: {:?}", elapsed_us);

        println!("Output: {:?}", output);

        let action = i64::from(output.multinomial(1, true));

        println!("Action: {:?}", action);

        process::exit(1);
    }

    let mut iteration = 0;
    'train_loop: loop {
        if iteration != 0 {
            //todo train networks
            //train
            train("Herbivore");
        }

        // reset world
        world.clear();

        // spawn herbi and carni //todo inherit traits
        spawn_herbi(tx.clone());

        //todo spawn carni

        for _ in 1..=PLANT_START {
            spawn_plant(tx.clone());
        }

        println!("Simulation started, iteration: {:?}", iteration);
        let mut iteration_sim = 0;
        'sim_loop: loop {
            // receive beast/plant states
            let received = &rx;
            for msg in received.try_iter() {
                if msg.alive {
                    world.insert(
                        msg.id,
                        (
                            msg.beast,
                            msg.pos,
                            msg.dir,
                            msg.fov,
                            msg.sight_range,
                            msg.speed,
                            msg.handle,
                        ),
                    );
                } else {
                    //remove only dead
                    let _ = world.remove(&msg.id);
                    //check if both herbi and carni alive
                    let mut herbi: bool = false;
                    let mut carni: bool = true; //todo change to false
                    for key in world.keys() {
                        let entry = world.get(key).unwrap();
                        if entry.0 == "Herbivore" {
                            herbi = true
                        }
                        if entry.0 == "Carnivore" {
                            carni = true
                        }
                    }
                    if (!herbi || !carni) && iteration_sim > 25 {
                        println!("Simulation ended");
                        break 'sim_loop;
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
                world_reverse.push((
                    entry.1, id, beast, entry.2, entry.3, entry.4, entry.5, handle,
                ));
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
                    cull: false,
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
            let msg = MainServer {
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
            iteration_sim += 1;
        }
        iteration += 1;
        for key in world.keys() {
            let entry = world.get(key).unwrap();
            let cull_msg = BeastUpdate {
                try_eat: false,
                eat_result: false,
                eat_value: 0,
                response_handle: None,
                world: None,
                cull: true,
            };
            let _ = entry.6.send(cull_msg);
        }
    }
}

fn spawn_plant(main_handle: Sender<Msg>) {
    let p = Plant::new(nanoid!(), MAPSIZE, main_handle);
    thread::spawn(|| plant::main(p));
}
fn spawn_herbi(main_handle: Sender<Msg>) {
    // spawn Herbivores
    //todo inherit physical traits from best evolution
    for _ in 1..=N_HERB {
        let mut rng = rand::thread_rng();
        let h = Herbivore::new(
            nanoid!(),
            (
                rng.gen_range(0.0..MAPSIZE as f64),
                rng.gen_range(0.0..MAPSIZE as f64),
            ),
            FOV,
            1.8, //rng.gen_range(1.5..2.5),
            0,
            main_handle.clone(),
        );
        thread::spawn(move || herbivore::main(h));
    }
}

fn train(beast_type: &str) {
    let mut signals_nn: [[[f32; 12]; 24]; 4] = [[[0.0; NN_RAY_LEN]; NN_RAYS]; N_TYPES];
    let mut vs = VarStore::new(tch::Device::Cpu);
    let mut path: String = String::from("init");
    if beast_type == "Herbivore" {
        vs.load("src/nn/weights/herbi/herbi_ac").unwrap();
        //samples = fs::read_dir("src/nn/samples/herbi/").unwrap();
        path = String::from("src/nn/samples/herbi/");
    } else if beast_type == "Carnivore" {
        vs.load("src/nn/weights/carni/carni_ac").unwrap();
    } else {
        println!("error in train");
        process::exit(1);
    }
    let mut optimizer = nn::Adam::default().build(&vs, LR).unwrap();

    let mut samples: ReadDir = fs::read_dir(path.clone()).unwrap();
    // discard old files
    let mut files_vec: Vec<(PathBuf, Duration)> = Vec::new();
    for sample in samples {
        let path = sample.as_ref().unwrap().path();
        let time = sample
            .as_ref()
            .unwrap()
            .metadata()
            .unwrap()
            .created()
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        if path
            .clone()
            .into_os_string()
            .into_string()
            .unwrap()
            .contains(".DS_Store")
        {
            continue;
        }
        files_vec.push((path, time));
    }

    files_vec.sort_by(|(a, b), (c, d)| d.cmp(b));

    if files_vec.len() > MAX_FILES {
        for index in MAX_FILES..(files_vec.len()) {
            let (path, _) = &files_vec[index];
            let _ = fs::remove_file(path);
        }
        files_vec.clear();
    }

    let model = ActorCritic::new(
        &vs,
        (NN_RAYS * NN_RAY_LEN * N_TYPES + N_STATES_SELF) as i64,
        7,
    );

    samples = fs::read_dir(path).unwrap();
    'file_loop: for sample in samples {
        let path = sample.as_ref().unwrap().path();
        if path
            .into_os_string()
            .into_string()
            .unwrap()
            .contains(".DS_Store")
        {
            continue;
        }
        let mut data = {
            let input = std::fs::read_to_string(sample.unwrap().path()).unwrap();
            serde_json::from_str::<Value>(&input).unwrap()
        };
        let entries: serde_json::Value =
            serde_json::from_value(data).expect("JSON was not well-formatted");
        let fov = &entries["fov"].as_i64().unwrap();
        let ros = &entries["sight_range"].as_i64().unwrap();
        let speed_base = &entries["speed_base"].as_f64().unwrap();
        let data: &mut Vec<Value> = &mut entries["data"].as_array().unwrap().clone();
        let mut rng = rand::thread_rng();
        data.shuffle(&mut rng);

        'data_point_loop: for entry in data {
            let state = entry["state"].as_array().unwrap();
            let pos_x = state[0][0].as_f64().unwrap();
            let pos_y = state[0][1].as_f64().unwrap();
            let pos = (pos_x, pos_y);
            let dir = state[1].as_i64().unwrap();
            let speed = state[2].as_f64().unwrap().round();
            let energy = state[3].as_f64().unwrap();
            let eaten = state[4].as_f64().unwrap();
            let self_state = [
                (speed / speed_base) as f32,
                (energy / ENERGY_MAX) as f32,
                (eaten / CHILD_THRESH as f64) as f32,
            ];
            let state_new = entry["state_new"].as_array().unwrap();
            let next_pos_x = state_new[0][0].as_f64().unwrap();
            let next_pos_y = state_new[0][1].as_f64().unwrap();
            let next_pos = (next_pos_x, next_pos_y);
            let next_dir = state_new[1].as_i64().unwrap();
            let next_speed = state_new[2].as_f64().unwrap().round();
            let next_energy = state_new[3].as_f64().unwrap();
            let next_eaten = state_new[4].as_f64().unwrap();
            let next_self_state = [
                (next_speed / speed_base) as f32,
                (next_energy / ENERGY_MAX) as f32,
                (next_eaten / CHILD_THRESH as f64) as f32,
            ];
            let mem = entry["mem"].as_array().unwrap();
            let action = entry["action"].as_i64().unwrap();
            let reward = entry["reward"].as_f64().unwrap();

            /*println!("x: {:?}, y: {:?}, dir: {:?}, speed: {:?}, energy: {:?}, eaten: {:?}",
            pos_x, pos_y, dir, speed, energy, eaten);*/

            signals_nn
                .iter_mut()
                .for_each(|m| m.iter_mut().for_each(|m| *m = [0.0; NN_RAY_LEN]));

            add_border(&mut signals_nn, pos, dir as i32);

            for memory in mem {
                let beast_type = String::from(memory[0].as_str().unwrap());
                let other_pos_x = memory[1][0].as_f64().unwrap();
                let other_pos_y = memory[1][1].as_f64().unwrap();
                let other_pos = (other_pos_x, other_pos_y);
                let other_dir = memory[2].as_i64().unwrap();
                let other_speed = memory[3].as_f64().unwrap();

                let d = distance_index(pos, other_pos);
                if d > NN_RAY_LEN - 1 {
                    continue;
                } //memory larger than vision
                let r = ray_direction_index(pos, dir as i32, other_pos);
                let mut index = 100; // make it crash if error
                if beast_type == "Wall" {
                    index = 0
                }
                if beast_type == "Plant" {
                    index = 1
                }
                if beast_type == "Herbivore" {
                    index = 2
                }
                if beast_type == "Carnivore" {
                    index = 3
                }

                signals_nn[index][r][d] = 1.0;
            }

            let mut wall_tensor: Tensor = Tensor::of_slice2(&signals_nn[0]);
            let mut plant_tensor: Tensor = Tensor::of_slice2(&signals_nn[1]);
            let mut herbiv_tensor: Tensor = Tensor::of_slice2(&signals_nn[2]);
            let mut carniv_tensor: Tensor = Tensor::of_slice2(&signals_nn[3]);
            let mut self_state_tensor: Tensor = Tensor::of_slice(&self_state);

            signals_nn
                .iter_mut()
                .for_each(|m| m.iter_mut().for_each(|m| *m = [0.0; NN_RAY_LEN]));

            add_border(&mut signals_nn, next_pos, next_dir as i32);

            for memory in mem {
                let beast_type = String::from(memory[0].as_str().unwrap());
                let other_pos_x = memory[1][0].as_f64().unwrap();
                let other_pos_y = memory[1][1].as_f64().unwrap();
                let other_pos = (other_pos_x, other_pos_y);
                let other_dir = memory[2].as_i64().unwrap();
                let other_speed = memory[3].as_f64().unwrap();

                let d = distance_index(pos, other_pos);
                if d > NN_RAY_LEN - 1 {
                    continue;
                } //memory larger than vision
                let r = ray_direction_index(next_pos, next_dir as i32, other_pos);
                let mut index = 100; // make it crash if error
                if beast_type == "Wall" {
                    index = 0
                }
                if beast_type == "Plant" {
                    index = 1
                }
                if beast_type == "Herbivore" {
                    index = 2
                }
                if beast_type == "Carnivore" {
                    index = 3
                }

                signals_nn[index][r][d] = 1.0;
            }

            let mut next_wall_tensor: Tensor = Tensor::of_slice2(&signals_nn[0]);
            let mut next_plant_tensor: Tensor = Tensor::of_slice2(&signals_nn[1]);
            let mut next_herbiv_tensor: Tensor = Tensor::of_slice2(&signals_nn[2]);
            let mut next_carniv_tensor: Tensor = Tensor::of_slice2(&signals_nn[3]);
            let mut next_self_state_tensor: Tensor = Tensor::of_slice(&next_self_state);

            let (action_probs, value) =
                model.forward(&wall_tensor, &plant_tensor, &self_state_tensor);
            let (_, next_value) = model.forward(
                &next_wall_tensor,
                &next_plant_tensor,
                &next_self_state_tensor,
            );

            let log_probs = action_probs.log_softmax(-1, Kind::Double);
            let action_prob = log_probs.index_select(-1, &Tensor::from(action)).squeeze();

            let target = Tensor::from(reward) + Tensor::from(GAMMA) * next_value - value;

            let action_log_prob = log_probs.index_select(-1, &Tensor::from(action)).squeeze();

            let loss_actor = -(action_log_prob * &target);
            let loss_critic = &target * &target;
            let loss = loss_actor + loss_critic;

            optimizer.zero_grad();
            loss.backward();
            optimizer.step();
        }
    }

    if beast_type == "Herbivore" {
        vs.save("src/nn/weights/herbi/herbi_ac").unwrap();
    }
    if beast_type == "Carnivore" {
        vs.save("src/nn/weights/carni/carni_ac").unwrap();
    }

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
