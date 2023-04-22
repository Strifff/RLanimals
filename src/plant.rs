use crate::conc::{Msg, BeastUpdate};
use crate::mpsc::{Sender/*,Receiver*/};
use std::sync::{/*Arc, Mutex,*/ mpsc};
use std::{thread, time::Duration};

use rand::Rng;

use crate::MAPSIZE;
use crate::DELAY;

const ENERGY_LOW: i32 = 10;
const ENERGY_HIGH: i32 = 30;


pub struct Plant {
    id: String,
    alive: bool,
    pos: (f64,f64),
    energy: i32,
    mapsize: i32,
    main_handle: Sender<Msg>,
}

impl Plant {
    pub fn new(
        id: String, 
        mapsize: i32,
        handle: Sender<Msg>,
    ) -> Plant {

        Plant {
            id: id,
            alive: true,
            pos: {let mut rng = rand::thread_rng();
                (rng.gen_range(10..mapsize-10) as f64,
                 rng.gen_range(10..mapsize-10) as f64)}, 
            energy: {let mut rng = rand::thread_rng();
                    rng.gen_range(ENERGY_LOW..ENERGY_HIGH)},
            mapsize: mapsize,
            main_handle: handle,
            //world: world,
        }
    }
    fn set_id(&mut self, id: String) {
        self.id = id;
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn set_pos(&mut self, pos: (f64,f64)) {
        self.pos = pos;
    }
    fn get_pos(&self) -> (f64,f64) {
        self.pos.clone()
    }
    fn eat(&mut self) -> bool {
        if self.alive {
            self.alive = false;
            return true;
        }
        false
    }
}

pub fn main(mut p: Plant) {
    let (tx, rx) = mpsc::channel::<BeastUpdate>();
    let init_msg = Msg {
        id: p.get_id(),
        alive: true,
        beast: "Plant".to_owned(),
        pos: p.get_pos(),
        dir: 0,
        fov: 0,
        sight_range: 0,
        speed: 0.0,
        handle: tx.clone(),
    };
    p.main_handle.send(init_msg).unwrap();
    //init main with static state

    'plant_loop: while p.alive {
        //receive
        let received = &rx;

        for msg in received {
            if msg.try_eat && p.alive {
                let response = BeastUpdate {
                    try_eat: false,
                    eat_result: true,
                    eat_value: p.energy,
                    response_handle: None,
                    world: None,
                };
                let _ = msg.response_handle.unwrap().send(response);
                p.alive = false;
                break 'plant_loop
            } //plant does not care about worldly things
        }

        //delay                                
        thread::sleep(Duration::from_millis((DELAY).try_into().unwrap()));
    }
    let death_msg = Msg {
        id: p.get_id(),
        alive: false,
        beast: "Plant".to_owned(),
        pos: p.get_pos(),
        dir: 0,
        fov: 0,
        sight_range: 0,
        speed: 0.0,
        handle: tx.clone(),
    };
    p.main_handle.send(death_msg).unwrap(); 
}

