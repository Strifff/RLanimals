use crate::beast_traits::Beast;

use std::{thread, time::Duration, convert::TryInto};

pub struct Herbivore {
    id: String,
    alive: bool,
    pos: (f32,f32),
    dir: i32,
    speed: i32,
    fov: i32,
    energy: f32,
    mapsize: i32,
}

impl Herbivore {
    pub fn new(id: String, pos: (f32,f32), fov: i32, mapsize: i32) -> Herbivore {
        Herbivore {
            id: id,
            alive: true,
            pos: pos, 
            dir: 45, 
            speed: 0, 
            energy: 100.0,
            fov: fov,
            mapsize: mapsize}
    }
}

impl Beast for Herbivore {
    fn set_id(&mut self, id: String) {
        self.id = id;
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn set_pos(&mut self, pos: (f32,f32)) {
        self.pos = pos;
    }
    fn get_pos(&self) -> (f32,f32) {
        self.pos.clone()
    }
    fn set_dir(&mut self, dir: i32) {
        self.dir = dir;
    }
    fn get_dir(&self) -> i32 {
        self.dir.clone()
    }

    fn set_speed1(&mut self) {
        self.speed = 1;
    }
    fn set_speed2(&mut self) {
        self.speed = 2;
    }
    fn set_speed3(&mut self) {
        self.speed = 3;
    }
    fn get_speed(&self) -> i32 {
        self.speed.clone()
    }
    fn forward(&mut self) {
        let dir_rad: f32 = self.dir as f32 *3.141593/180.0;
        let x = self.pos.0 + (self.speed as f32)*dir_rad.cos();
        let y = self.pos.1 + (self.speed as f32)*dir_rad.sin();

        self.pos = self.in_bounds(x,y);
        let _ = self.consume_energy();
    }
    fn left(&mut self) {
        self.dir = (self.dir+15)%360;
        let _ = &self.forward();
    }
    fn right(&mut self) {
        self.dir = (self.dir-15)%360;
        let _ = &self.forward();
    }
    fn back(&mut self) {
        let save_speed = self.speed;
        self.speed = -1;
        let _ = self.forward();
        self.speed = save_speed;
    }
    fn consume_energy(&mut self) {
        let speed: f32 = self.speed as f32;
        self.energy = self.energy - speed*speed/2.0;
        let _ = self.dead();
    }
    fn in_bounds(&self, x: f32, y: f32) -> (f32,f32) {
        let vec: Vec<f32> = vec![x,y].into_iter().map(|val| 
            {if val > self.mapsize as f32 {
                self.mapsize.clone() as f32
            } else if val < 0 as f32 {
                0 as f32
            } else { 
                val as f32
            }}).collect();
        
        (vec[0],vec[1])
    }
    fn dead(&mut self) {
        if self.energy < 0.0 {
            self.alive = false
        }
    }
}

pub fn main(mut h: Herbivore, delay: i32) {

    h.set_speed3();

    while h.alive {
        //pull main 

        //take action

        //update main

        println!("id: {:?}, pos: {:?}, energy: {:?}", h.get_id(), h.get_pos(), h.energy);
        h.left();
        thread::sleep(Duration::from_millis(delay.try_into().unwrap()));
    }
    println!("{:?} is dead", h.get_id());   
}