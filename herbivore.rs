//mod test_traits;
//use test_traits::Beast;


use beast_traits::Beast;

use std::{thread, time::Duration, convert::TryInto};

pub struct Herbivore {
    id: String,
    pos: (i32,i32),
    dir: i32,
    speed: i32,
    energy: i32,
    mapsize: i32,
}

impl Herbivore {
    pub fn new(id: String, pos: (i32,i32), mapsize: i32) -> Herbivore {
        Herbivore {
            id: id, 
            pos: pos, 
            dir: 0, 
            speed: 0, 
            energy: 100, 
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
    fn set_pos(&mut self, pos: (i32,i32)) {
        self.pos = pos;
    }
    fn get_pos(&self) -> (i32,i32) {
        self.pos.clone()
    }
    fn set_dir(&mut self, dir: i32) {
        self.dir = dir;
    }
    fn get_dir(&self) -> i32 {
        self.dir.clone()
    }


}



pub fn main(h: Herbivore, delay: i32) {

    println!("id: {:?}, pos: {:?}, dir: {:?}", h.get_id(), h.get_pos(), h.get_dir());
    println!("id: {:?},", h.get_id());

    let new_name: String = format!("{}-1",h.get_id());
    if new_name.len() > 10 {
        loop{
            thread::sleep(Duration::from_millis(delay.try_into().unwrap()));
            //println!("id: {:?},", h.get_id());
        }

    } else {
        let nh: Herbivore = Herbivore::new(new_name, (1,1), h.mapsize.clone());
        thread::spawn(move || {main(nh, delay)});
        
    }

        

    
}