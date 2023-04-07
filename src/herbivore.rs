use crate::beast_traits::Beast;


use std::{thread, time::Duration, convert::TryInto, collections::HashMap};

use crate::import_beast;
use crate::conc::{Msg, BeastUpdate};
use crate::mpsc::{Sender,Receiver};
use std::sync::{Arc, Mutex, mpsc};
use arc_swap::ArcSwap;

pub struct Herbivore {
    id: String,
    alive: bool,
    pos: (f64,f64),
    dir: i32,
    speed_base: f64,
    speed_curr: f64,
    fov: i32,
    energy: f64,
    mapsize: i32,
    receiver: Sender<Msg>,
    //world: &'static ArcSwap<Vec<((f64, f64), String, String, i32, f64)>>,
}

impl Herbivore {
    pub fn new(
        id: String, 
        pos: (f64,f64), 
        fov: i32, 
        speed: f64, 
        mapsize: i32,
        receiver: Sender<Msg>,
       // world: &'static ArcSwap<Vec<((f64, f64), String, String, i32, f64)>>,
    ) -> Herbivore {

        Herbivore {
            id: id,
            alive: true,
            pos: pos, 
            dir: 0, //todo rng
            speed_base: speed,
            speed_curr: speed,
            energy: 10000000000.0,
            fov: fov,
            mapsize: mapsize,
            receiver: receiver,
            //world: world,
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
    fn set_pos(&mut self, pos: (f64,f64)) {
        self.pos = pos;
    }
    fn get_pos(&self) -> (f64,f64) {
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
    }
    fn set_speed2(&mut self) {
        self.speed_curr = self.speed_base * 2 as f64;
    }
    fn set_speed3(&mut self) {
        self.speed_curr = self.speed_base * 3 as f64;
    }
    fn get_speed(&self) -> f64 {
        self.speed_curr.clone()
    }
    fn forward(&mut self) {
        let dir_rad: f64 = self.dir as f64 *3.141593/180.0;
        let x = self.pos.0 + self.speed_curr * dir_rad.cos();
        let y = self.pos.1 + self.speed_curr * dir_rad.sin();

        self.pos = self.in_bounds(x,y);
        self.consume_energy();
    }
    fn left(&mut self) {
        self.dir = (self.dir+15)%360;
        self.forward();
    }
    fn right(&mut self) {
        self.dir = (self.dir-15)%360;
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
    fn in_bounds(&self, x: f64, y: f64) -> (f64,f64) {
        let vec: Vec<f64> = vec![x,y].into_iter().map(|val| 
            {if val > self.mapsize as f64 {
                self.mapsize.clone() as f64
            } else if val < 0 as f64 {
                0 as f64
            } else { 
                val as f64
            }}).collect();
        
        (vec[0],vec[1])
    }
    fn starve(&mut self) {
        if self.energy < 0.0 {
            self.alive = true;
        }
    }
    fn kill(&mut self) -> bool {
        if self.alive {
            self.alive = false;
            return true;
        }
        false
    }
}

pub fn main(mut h: Herbivore, delay: i32) {

    //h.set_speed3();
    import_beast(&h);

    let (tx, rx) = mpsc::channel::<BeastUpdate>();
    let receiver = tx.clone();

    let mut world: Vec<((f64, f64), String, String, i32, f64, Sender<BeastUpdate>)> = Vec::new();

    while h.alive {


        /*if x == 10 && h.get_id() == "test" {
            let b1 = Herbivore::new("test-----".to_owned(), (50.0,50.0), h.fov, 1.0, h.mapsize);
            thread::spawn(move || {main(b1, delay)}); 

        }
        x += 1;*/
        //pull main 
        let received = &rx;
 
        world.clear();
        for msg in received.try_iter() {
            world = msg.world;
            if h.get_id() == "test2" {
                for entry in &world {
                    println!("Entry: {:?}", entry)
                }
            }
        }


        //take action
        h.left();

        //update main
        let mut msg = Msg{
            id:     h.get_id(),
            beast:  "Herbivore".to_owned(),
            pos:    h.get_pos(),
            dir:    h.get_dir(),
            speed:  h.get_speed(),
            handle: receiver.clone(),
        };

        h.receiver.send(msg).unwrap();

        //delay
        
        thread::sleep(Duration::from_millis(delay.try_into().unwrap()));

        //DEBUG
        //println!("id: {:?}, pos: {:?}, energy: {:?}", h.get_id(), h.get_pos(), h.energy);
 
    }

    //after death
    println!("{:?} died", h.get_id()); //todo cause of death 
}