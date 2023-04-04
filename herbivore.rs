//mod test_traits;
//use test_traits::Beast;


use beast_traits::Beast;

use std::{thread, time::Duration, convert::TryInto};

pub struct Herbivore {
    id: String,
    pos: (f32,f32),
    dir: i32,
    speed: i32,
    fov: i32,
    energy: i32,
    mapsize: i32,
}

impl Herbivore {
    pub fn new(id: String, pos: (f32,f32), fov: i32, mapsize: i32) -> Herbivore {
        Herbivore {
            id: id, 
            pos: pos, 
            dir: 45, 
            speed: 0, 
            energy: 100,
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
        let x = self.pos.0 + (self.speed as f32)*((self.dir as f32 *3.141593/180.0) as f32).cos();
        let y = self.pos.1 + (self.speed as f32)*((self.dir as f32 *3.141593/180.0) as f32).sin();

        self.pos = self.in_bounds(x,y); 
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
        let x = self.pos.0 - (self.speed as f32)*(self.dir as f32).cos();
        let y = self.pos.1 - (self.speed as f32)*(self.dir as f32).sin();
        
        self.pos = self.in_bounds(x,y); 
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
}

pub fn main(mut h: Herbivore, delay: i32) {

    h.set_speed3();

    loop{
        println!("id: {:?}, pos: {:?}, dir: {:?}", h.get_id(), h.get_pos(), h.get_dir());

        h.left();

        thread::sleep(Duration::from_millis(delay.try_into().unwrap()));

    }

    /*let new_name: String = format!("{}-1",h.get_id());
    if new_name.len() > 10 {
        loop{
            thread::sleep(Duration::from_millis(delay.try_into().unwrap()));
            //println!("id: {:?},", h.get_id());
        }

    } else {
        let nh: Herbivore = Herbivore::new(new_name, (1.1,1.1), h.fov.clone(), h.mapsize.clone());
        thread::spawn(move || {main(nh, delay)});
        
    }*/    
}