use crate::beast_traits::Beast;
use std::process;
use std::{/*cmp::Ordering,*/ thread, time::Duration, convert::TryInto};

use crate::conc::{Msg, BeastUpdate};
use crate::mpsc::{Sender/*,Receiver*/};
use std::sync::{/*Arc, Mutex,*/ mpsc};
//use arc_swap::ArcSwap;
use rand::Rng;


pub struct Herbivore {
    id: String,
    alive: bool,
    pos: (f64,f64),
    dir: i32,
    speed_base: f64,
    speed_curr: f64,
    fov: i32,
    sight_range: i32,
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
            dir: {
                let mut rng = rand::thread_rng();
                //random direction with increments of 15
                15*rng.gen_range(0..24) },
            speed_base: speed,
            speed_curr: speed,
            energy: 10000.0,
            fov: fov,
            sight_range: { let sr = 1000.0*(1.0/fov as f64).sqrt();
                            sr as i32},
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
        if self.dir < 0 { self.dir += 360;}
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
    fn get_fov(&self) -> i32 {
        self.fov
    }
    fn get_ros(&self) -> i32 {
        self.sight_range
    }
    fn starve(&mut self) {
        if self.energy < 0.0 {
            self.alive = false;
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

    let (tx, rx) = mpsc::channel::<BeastUpdate>();
    let receiver = tx.clone();

    let mut world: Vec<((f64, f64), String, String, i32, i32, i32, f64, Sender<BeastUpdate>)> = Vec::new();

    let mut rng = rand::thread_rng();

    while h.alive {
        let received = &rx;
 
        world.clear();
        for msg in received.try_iter() {
            world = msg.world;
            //todo only work on last msg
        }

        //take action
        world.retain(|(other_pos,id,_,_,_,_,_,_)| 
            *id != h.get_id()            // cant see self
            &&                           //     and
            in_view(h.get_pos(), h.get_dir(), h.get_fov(), h.get_ros(), *other_pos)); // other in field of view

        
        //todo add border

        //todo discrete values for entries
        //todo position(angle, distance)
        //todo type
        //todo speed
        //todo dir

        //todo append from memory
        for entry in &world {
            println!("in view: {:?}, from pov: {:?}", entry, h.get_id());
        }
        
        let index = rng.gen_range(0..6) as i32;
        match index {
            0 => {h.set_speed1()}
            1 => {h.set_speed2()}
            2 => {h.set_speed3()}
            3 => {h.forward()}
            4 => {h.left()}
            5 => {h.right()}
            6 => {h.back()}
            _ => {}
        }

        //update main
        let msg = Msg{
            id:     h.get_id(),
            alive:  true,
            beast:  "Herbivore".to_owned(),
            pos:    h.get_pos(),
            dir:    h.get_dir(),
            fov:    h.get_fov(),
            sight_range: h.get_ros(),
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
    let msg = Msg{
        id:     h.get_id(),
        alive:  false,
        beast:  "Herbivore".to_owned(),
        pos:    h.get_pos(),
        dir:    h.get_dir(),
        fov:    h.get_fov(),
        sight_range: h.get_ros(),
        speed:  h.get_speed(),
        handle: receiver.clone(),
    };

    h.receiver.send(msg).unwrap();

}

fn in_view(pos_self: (f64, f64), dir_self: i32, fov_self: i32, ros_self: i32, other_pos: (f64, f64)) -> bool {
    
    let left_dir_rad: f64 = ((dir_self+fov_self/2)%180) as f64 *3.141593/180.0;
    let right_dir_rad: f64 = ((dir_self-fov_self/2)%180) as f64 *3.141593/180.0;
    
    let left_slope = left_dir_rad.tan();
    let right_slope = right_dir_rad.tan();

    //left bound
    let left = match (dir_self + fov_self/2)%360 {
        0..=89      => {!point_above_line(pos_self, left_slope, other_pos)}
        271..=359   => {!point_above_line(pos_self, left_slope, other_pos)}
        91..=269    => {point_above_line(pos_self, left_slope, other_pos)}
        90          => {pos_self.0 < other_pos.0} 
        270         => {pos_self.0 > other_pos.0}
        _           => false //angle can only be 0..359
    };  
    
    let right = match (dir_self - fov_self/2)%360 {
        0..=89      => {point_above_line(pos_self, right_slope, other_pos)}
        271..=359   => {point_above_line(pos_self, right_slope, other_pos)}
        91..=269    => {!point_above_line(pos_self, right_slope, other_pos)}
        90          => {pos_self.0 > other_pos.0} 
        270         => {pos_self.0 < other_pos.0}
        _           => false //angle can only be 0..359
    };  

    //distance
    let distance: bool = {
        let dx = (pos_self.0 - other_pos.0).abs();
        let dy = (pos_self.1 - other_pos.1).abs();
    
        let d = (dx*dx+dy*dy).sqrt();

        d < ros_self as f64
    };

    left && right && distance
}

fn point_above_line((x,y): (f64, f64), slope: f64, point: (f64, f64)) -> bool {
    // y = k*x + m
    let m = y - slope*x;

    point.0 * slope + m < point.1
}

