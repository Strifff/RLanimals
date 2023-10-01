use crate::beast_traits::{Actor, Beast};
use std::collections::HashMap;
use std::process;
use std::{convert::TryInto, /*cmp::Ordering,*/ thread, time::Duration};
use tch::kind::{FLOAT_CPU, INT64_CPU};
use tch::{nn, nn::Module, nn::OptimizerConfig, nn::VarStore, Kind, Tensor};

use crate::conc::{BeastUpdate, Msg};
use crate::genAlg;
use crate::mpsc::Sender;
use crate::A2C::{ActorCritic, States};
use nanoid::nanoid;
use rand::Rng;
use serde_json::{json, Value};
use std::sync::mpsc;

// make environment discrete
use crate::DELAY;
use crate::{CHILD_THRESH, ENERGY_MAX, MAPSIZE, MARGIN};
use crate::{DEG_TO_RAD, RAD_TO_DEG};
use crate::{NN_RAYS, NN_RAY_DR, NN_RAY_LEN, N_STATES_SELF, N_TYPES};

//todo memory range 1.5x vision range, same amount of steps
// forget objects far away
const MEM_RADIUS: i32 = ((NN_RAY_LEN as f64 + 1.5) * NN_RAY_DR as f64) as i32;
const EAT_RANGE: i32 = 50;
// food to spawn child
const SCORE_EAT: i32 = 100;
const SCORE_SURVIVE: i32 = 2;
const SCORE_DIE: i32 = -500;
const SCORE_ENERGY: f64 = -0.05;

pub struct Herbivore {
    id: String,
    alive: bool,
    pos: (f64, f64),
    dir: i32,
    speed_base: f64,
    speed_curr: f64,
    fov: i32,
    sight_range: i32,
    energy: f64,
    eaten: i32,
    gen: i32,
    main_handle: Sender<Msg>,
    cause_of_death: String,
}

impl Herbivore {
    pub fn new(
        id: String,
        pos: (f64, f64),
        fov: i32,
        speed: f64,
        gen: i32,
        handle: Sender<Msg>,
    ) -> Herbivore {
        Herbivore {
            id: id,
            alive: true,
            pos: pos,
            dir: {
                let mut rng = rand::thread_rng();
                //random direction with increments of 15
                15 * rng.gen_range(0..24)
            },
            speed_base: speed,
            speed_curr: speed,
            energy: ENERGY_MAX,
            fov: fov,
            sight_range: {
                let sr = 1000.0 * (1.0 / fov as f64).sqrt();
                sr as i32
            },
            eaten: 0,
            gen: gen,
            main_handle: handle,
            cause_of_death: String::from("unknown"),
        }
    }
}

impl Beast for Herbivore {
    fn set_id(&mut self, id: String) {
        self.id = id;
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn set_pos(&mut self, pos: (f64, f64)) {
        self.pos = pos;
    }
    fn get_pos(&self) -> (f64, f64) {
        self.pos.clone()
    }
    fn set_dir(&mut self, dir: i32) {
        self.dir = dir;
    }
    fn get_dir(&self) -> i32 {
        self.dir.clone()
    }

    fn set_speed1(&mut self) {
        self.speed_curr = self.speed_base;
        self.forward();
    }
    fn set_speed2(&mut self) {
        self.speed_curr = self.speed_base * 2 as f64;
        self.forward();
    }
    fn set_speed3(&mut self) {
        self.speed_curr = self.speed_base * 3 as f64;
        self.forward();
    }
    fn get_speed(&self) -> f64 {
        self.speed_curr.clone()
    }
    fn get_speed_base(&self) -> f64 {
        self.speed_base.clone()
    }
    fn forward(&mut self) {
        let dir_rad: f64 = self.dir as f64 * 3.141593 / 180.0;
        let x = self.pos.0 + self.speed_curr * dir_rad.cos();
        let y = self.pos.1 + self.speed_curr * dir_rad.sin();

        self.pos = self.in_bounds(x, y);
        self.consume_energy();
    }
    fn left(&mut self) {
        self.dir = (self.dir + 15) % 360;
        self.forward();
    }
    fn right(&mut self) {
        self.dir = (self.dir - 15) % 360;
        if self.dir < 0 {
            self.dir += 360;
        }
        self.forward();
    }
    fn back(&mut self) {
        let save_speed = self.speed_curr;
        self.speed_curr = -self.speed_base;
        self.forward();
        self.speed_curr = save_speed;
    }
    fn consume_energy(&mut self) {
        let speed: f64 = self.speed_curr;
        self.energy = self.energy - speed * speed / 2.0;
        self.starve();
    }
    fn in_bounds(&self, x: f64, y: f64) -> (f64, f64) {
        let vec: Vec<f64> = vec![x, y]
            .into_iter()
            .map(|val| {
                if val + MARGIN as f64 > MAPSIZE as f64 {
                    (MAPSIZE - MARGIN) as f64
                } else if (val - MARGIN as f64) < 0.0 {
                    (0 + MARGIN) as f64
                } else {
                    val as f64
                }
            })
            .collect();

        (vec[0], vec[1])
    }
    fn get_fov(&self) -> i32 {
        self.fov
    }
    fn get_ros(&self) -> i32 {
        self.sight_range
    }
    fn starve(&mut self) {
        if self.energy < 0.0 {
            self.alive = false;
            self.cause_of_death = String::from("starved");
        }
    }
    fn kill(&mut self) -> bool {
        if self.alive {
            self.alive = false;
            self.cause_of_death = String::from("eaten");
            return true;
        }
        false
    }
}

pub fn main(mut h: Herbivore) {
    let (tx, rx) = mpsc::channel::<BeastUpdate>();
    let receiver = tx.clone();

    let mut world: Vec<(
        (f64, f64),
        String,
        String,
        i32,
        i32,
        i32,
        f64,
        Sender<BeastUpdate>,
    )> = Vec::new();

    let mut rng = rand::thread_rng();
    //(msg.beast, msg.pos, msg.dir, msg.speed, msg.handle)
    let mut memory: HashMap<String, (String, (f64, f64), i32, f64, Sender<BeastUpdate>)> =
        HashMap::new();

    //let mut signals_nn: [[[f32; 12]; 24]; 4] = [[[0.0; NN_RAY_LEN]; NN_RAYS]; N_TYPES];
    let mut signals_nn: [[[f32; NN_RAY_LEN]; NN_RAYS]; N_TYPES] =
        [[[0.0; NN_RAY_LEN]; NN_RAYS]; N_TYPES];

    let mut keys_to_remove: Vec<String> = Vec::new();

    let mut vs = VarStore::new(tch::Device::Cpu);
    vs.load("src/nn/weights/herbi/herbi_ac").unwrap();

    let herbivore_ac = ActorCritic::new(
        &vs,
        (NN_RAYS * NN_RAY_LEN * N_TYPES + N_STATES_SELF) as i64,
        7,
    );

    let mut training_states: Vec<States> = Vec::new();

    // state = pos, dir, speed vs base, energy vs max, eaten vs child_thresh
    let mut self_state: ((f64, f64), i32, f64, f64, f64) = ((0.0, 0.0), 0, 1.0, 1.0, 0.0);
    let mut mem: Vec<(String, (f64, f64), i32, f64)> = Vec::new();
    let mut action: i64 = 0;
    let mut reward: f64 = 0.0;

    let mut state = States {
        state: self_state,
        memory: mem,
        action: action,
        reward: reward,
        state_new: self_state,
    };
    let mut memory_vec: Vec<(String, (f64, f64), i32, f64)> = Vec::new();

    let mut first_it: bool = true;
    'herb_loop: while h.alive {
        let received = &rx;

        world.clear();
        for msg in received.try_iter() {
            if msg.try_eat && h.alive {
                //todo respond to carnivore
                reward += SCORE_DIE as f64;
                state.reward = reward;
                training_states.push(state.clone());
                break 'herb_loop;
            } else if msg.eat_result {
                h.eaten += msg.eat_value;
                h.energy += msg.eat_value as f64 * 5.0;
                if h.eaten > CHILD_THRESH {
                    spawn_child(&h, h.gen, h.main_handle.clone());
                    h.eaten -= CHILD_THRESH;
                }
                reward += SCORE_EAT as f64;
            } else {
                world = msg.world.unwrap();
            }
        }

        let energy_reward = h.get_speed() * h.get_speed() * SCORE_ENERGY;
        reward += energy_reward;

        state.reward = reward;

        //submit state
        if !first_it {
            training_states.push(state.clone());
            reward = 0.0;
        }

        //take action
        world.retain(|(other_pos, id, _, _, _, _, _, _)| {
            *id != h.get_id()            // cant see self
            &&                           //     and
            in_view(&h,  *other_pos)
        }); // other in field of view

        //memory as hashmap to overwrite the old position
        for entry in &world {
            let id = entry.1.clone();
            let actor = entry.2.clone();
            let handle = entry.7.clone();
            // VALUE: (msg.actor/beast, msg.pos, msg.dir, msg.speed, msg.handle)
            memory.insert(id, (actor, entry.0, entry.3, entry.6, handle));
        }

        // clear mamory from entries further away than RADIUS
        //todo clear memory after time, forgetting
        memory.retain(|key, entry| point_within_radius(h.pos, entry.1, MEM_RADIUS));

        memory_vec.clear();
        for key in memory.keys() {
            let entry = memory.get(key).unwrap();
            let beast = entry.0.to_owned();
            let mem_learn: (String, (f64, f64), i32, f64) = (beast, entry.1, entry.2, entry.3);
            memory_vec.push(mem_learn);
        }
        state.memory = memory_vec.clone();

        for key in memory.keys() {
            let entry = memory.get(key).unwrap();
            if point_within_radius(h.pos, entry.1, EAT_RANGE) && entry.0 == "Plant" {
                // herbivore can only act on plants
                let eat_msg = BeastUpdate {
                    try_eat: true,
                    eat_result: false,
                    eat_value: 0,
                    response_handle: Some(tx.clone()),
                    world: None,
                    cull: false,
                };
                match entry.4.send(eat_msg) {
                    Ok(o) => {} //result doesn't matter, cant unwrap
                    Err(e) => {}
                }
                keys_to_remove.push(key.clone());
            }
        }

        for key in &keys_to_remove {
            memory.remove(key);
        }
        keys_to_remove.clear();

        //reset signals
        signals_nn
            .iter_mut()
            .for_each(|m| m.iter_mut().for_each(|m| *m = [0.0; NN_RAY_LEN]));

        add_border(&mut signals_nn, h.pos, h.dir);

        for key in memory.keys() {
            let entry = memory.get(key).unwrap();

            let d = distance_index(h.pos, entry.1);
            if d > NN_RAY_LEN - 1 {
                continue;
            } //memory larger than vision
            let r = ray_direction_index(h.pos, h.dir, entry.1);
            let mut index = 100; // make it crash if error
            if entry.0 == "Wall" {
                index = 0
            }
            if entry.0 == "Plant" {
                index = 1
            }
            if entry.0 == "Herbivore" {
                index = 2
            }
            if entry.0 == "Carnivore" {
                index = 3
            }

            // set corresponding point in signal to 1 for each of the types
            signals_nn[index][r][d] = 1.0;
        }

        // input of world state
        let mut wall_tensor: Tensor = Tensor::of_slice2(&signals_nn[0]);
        let mut plant_tensor: Tensor = Tensor::of_slice2(&signals_nn[1]);
        let mut herbiv_tensor: Tensor = Tensor::of_slice2(&signals_nn[2]);
        let mut carniv_tensor: Tensor = Tensor::of_slice2(&signals_nn[3]);

        //input of self state
        let mut beast_state_tensor: Tensor = Tensor::of_slice(&[
            (h.get_speed() / h.get_speed_base()) as f32,
            (h.energy / ENERGY_MAX as f64) as f32,
            (h.eaten as f64 / CHILD_THRESH as f64) as f32,
        ]);

        // run nn
        /*let (action_prob, value) = herbivore_ac.forward(
            &wall_tensor,
            &plant_tensor,
            &beast_state_tensor
        );*/

        // gen algorithm
        let action_prob = genAlg::forward(signals_nn);


        //action = action_prob.softmax(-1, Kind::Float).multinomial(1, true).into_kind(INT64_CPU);
        //herbivore_ac.vs.get();
        //action = i64::from(action_prob.multinomial(1, true));
        state.action = action;
        state.state = self_state;
        reward += SCORE_SURVIVE as f64;

        match action {
            0 => h.set_speed1(),
            1 => h.set_speed2(),
            2 => h.set_speed3(),
            3 => h.forward(),
            4 => h.left(),
            5 => h.right(),
            6 => h.back(),
            _ => {
                println!("Error in action");
                process::exit(1);
            }
        }

        self_state = (
            h.get_pos(),
            h.get_dir(),
            h.get_speed() / h.get_speed_base(),
            h.energy / ENERGY_MAX as f64,
            h.eaten as f64 / CHILD_THRESH as f64,
        );
        state.state_new = self_state;

        //update main
        let msg = Msg {
            id: h.get_id(),
            alive: true,
            beast: "Herbivore".to_owned(),
            pos: h.get_pos(),
            dir: h.get_dir(),
            fov: h.get_fov(),
            sight_range: h.get_ros(),
            speed: h.get_speed(),
            handle: receiver.clone(),
        };

        h.main_handle.send(msg).unwrap();

        //delay
        thread::sleep(Duration::from_millis(DELAY.try_into().unwrap()));

        first_it = false;
    }
    //after death
    reward += SCORE_DIE as f64;
    state.reward = reward;
    training_states.push(state.clone());

    println!(
        "{:?} died, generation: {:?}, cause of death: {}",
        h.get_id(),
        h.gen,
        h.cause_of_death
    );

    //save states for training
    let path = format!("src/nn/samples/herbi/{}", h.get_id());

    let mut data_vec: Vec<Value> = Vec::new();
    for state in training_states {
        let entry_json = json!({
            "state": state.state,
            "mem": state.memory,
            "action": state.action,
            "reward": state.reward,
            "state_new": state.state_new,
        });
        data_vec.push(entry_json);
    }
    let data_json: Value = json!({
        "speed_base":   h.get_speed_base(),
        "fov":          h.get_fov(),
        "sight_range":  h.get_ros(),
        "data":         Value::Array(data_vec),
    });

    std::fs::write(path, serde_json::to_string_pretty(&data_json).unwrap()).unwrap();

    let msg = Msg {
        id: h.get_id(),
        alive: false,
        beast: "Herbivore".to_owned(),
        pos: h.get_pos(),
        dir: h.get_dir(),
        fov: h.get_fov(),
        sight_range: h.get_ros(),
        speed: h.get_speed(),
        handle: receiver.clone(),
    };

    h.main_handle.send(msg).unwrap();
}

fn in_view(h: &Herbivore, other_pos: (f64, f64)) -> bool {
    let pos_self = h.get_pos(); // position
    let dir_self = h.get_dir(); // view direction
    let fov_self = h.get_fov(); // field of view
    let ros_self = h.get_ros(); // range of sight

    let left_dir_rad: f64 = ((dir_self + fov_self / 2) % 180) as f64 * 3.141593 / 180.0;
    let right_dir_rad: f64 = ((dir_self - fov_self / 2) % 180) as f64 * 3.141593 / 180.0;

    let left_slope = left_dir_rad.tan();
    let right_slope = right_dir_rad.tan();

    //left bound
    let left = match (dir_self + fov_self / 2) % 360 {
        0..=89 => !point_above_line(pos_self, left_slope, other_pos),
        271..=359 => !point_above_line(pos_self, left_slope, other_pos),
        91..=269 => point_above_line(pos_self, left_slope, other_pos),
        90 => pos_self.0 < other_pos.0,
        270 => pos_self.0 > other_pos.0,
        _ => false, //angle can only be 0..359
    };

    let right = match (dir_self - fov_self / 2) % 360 {
        0..=89 => point_above_line(pos_self, right_slope, other_pos),
        271..=359 => point_above_line(pos_self, right_slope, other_pos),
        91..=269 => !point_above_line(pos_self, right_slope, other_pos),
        90 => pos_self.0 > other_pos.0,
        270 => pos_self.0 < other_pos.0,
        _ => false, //angle can only be 0..359
    };

    //distance
    let distance: bool = {
        let dx = (pos_self.0 - other_pos.0).abs();
        let dy = (pos_self.1 - other_pos.1).abs();

        let d = (dx * dx + dy * dy).sqrt();

        d < ros_self as f64
    };

    left && right && distance
}

fn point_above_line((x, y): (f64, f64), slope: f64, point: (f64, f64)) -> bool {
    // y = k*x + m
    let m = y - slope * x;

    point.0 * slope + m < point.1
}

fn point_within_radius(point_self: (f64, f64), point_other: (f64, f64), radius: i32) -> bool {
    let dx = (point_self.0 - point_other.0).abs();
    let dy = (point_self.1 - point_other.1).abs();
    let d = (dx * dx + dy * dy).sqrt();

    d < radius as f64
}

fn spawn_child(parent: &impl Beast, generation: i32, main_handle: Sender<Msg>) {
    let child = Herbivore::new(
        nanoid!(),
        parent.get_pos(),
        parent.get_fov(),
        parent.get_speed_base(),
        generation + 1,
        main_handle,
    );
    thread::spawn(move || main(child));
}

pub fn add_border(
    signals: &mut [[[f32; NN_RAY_LEN]; NN_RAYS]; N_TYPES],
    (pos_x, pos_y): (f64, f64),
    dir: i32,
) {
    for ray in 0..=NN_RAYS - 1 {
        let ray_dir = (dir + (ray * 360 / NN_RAYS) as i32) % 360;

        for radius in 1..=NN_RAY_LEN {
            let x = pos_x + NN_RAY_DR as f64 * (ray_dir as f64 * DEG_TO_RAD).cos() * radius as f64;
            let y = pos_y + NN_RAY_DR as f64 * (ray_dir as f64 * DEG_TO_RAD).sin() * radius as f64;
            if !in_bounds_bool(x, y) {
                signals[0][ray][radius - 1] = 1.0; //one-hot vector, wall == index 0

                break;
            }
        }
    }
}

fn in_bounds_bool(x: f64, y: f64) -> bool {
    if x >= 0.0 && x <= MAPSIZE as f64 && y >= 0.0 && y <= MAPSIZE as f64 {
        return true;
    }
    false
}

pub fn distance_index((self_x, self_y): (f64, f64), (othr_x, othr_y): (f64, f64)) -> usize {
    let dx = self_x - othr_x;
    let dy = self_y - othr_y;
    let d = (dx.powf(2.0) + dy.powf(2.0)).sqrt();

    (d / (NN_RAY_DR as f64)).round() as usize
}

pub fn ray_direction_index(
    (self_x, self_y): (f64, f64),
    self_dir: i32,
    (othr_x, othr_y): (f64, f64),
) -> usize {
    let dx = othr_x - self_x;
    let dy = othr_y - self_y;

    let mut dir = (dy / dx).atan() * RAD_TO_DEG;
    if dx < 0.0 {
        dir = dir + 180.0
    }; // account for quadrant

    dir = dir - self_dir as f64; // relative ange

    if dir < 0.0 {
        dir += 360.0
    } // positive degrees

    dir = dir / (360.0 / NN_RAYS as f64); // size of increments

    dir.round() as usize % NN_RAYS
}

/*type Model = Box<dyn Fn(&Tensor) -> (Tensor, Tensor)>;

fn model(p: &nn::Path, nact: i64) -> Model {
    let stride = |s| nn::ConvConfig { stride: s, ..Default::default() };
    let seq = nn::seq()
        .add(nn::linear(p, (NN_RAYS*NN_RAY_LEN) as i64, 100, Default::default()))
        .add_fn(|xs| xs.relu())
        .add(nn::linear(p, 100, nact, Default::default()))
        .add_fn(|xs| xs.relu());

        /*.add(nn::conv2d(p / "c1", NSTACK, 32, 8, stride(4)))
        .add_fn(|xs| xs.relu())
        .add(nn::conv2d(p / "c2", 32, 64, 4, stride(2)))
        .add_fn(|xs| xs.relu())
        .add(nn::conv2d(p / "c3", 64, 64, 3, stride(1)))
        .add_fn(|xs| xs.relu().flat_view())
        .add(nn::linear(p / "l1", 3136, 512, Default::default()))
        .add_fn(|xs| xs.relu());*/

    let critic = nn::linear(p / "cl", 512, 1, Default::default());
    let actor = nn::linear(p / "al", 512, nact, Default::default());
    let device = p.device();
    Box::new(move |xs: &Tensor| {
        let xs = xs.to_device(device).apply(&seq);
        (xs.apply(&critic), xs.apply(&actor))
    })
}*/
