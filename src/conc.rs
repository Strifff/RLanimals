//use std::sync::mpsc::Receiver;

//use crate::beast_traits::Beast;
use crate::mpsc::{Sender/*,Receiver*/};

pub struct Msg {
    pub id: String,
    pub alive: bool,
    pub beast: String,
    pub pos: (f64, f64),
    pub dir: i32,
    pub fov: i32,
    pub sight_range: i32,
    pub speed: f64,
    pub handle: Sender<BeastUpdate>,
}

pub struct BeastUpdate {
    pub kill: bool,
    pub world: Vec<((f64, f64), String, String, i32, i32, i32, f64, Sender<BeastUpdate>)>, 
}

//todo impl Msg for many msg types

pub struct MainServer {
    pub msg_type: String,
    pub msg_data: i32,
    pub handle_send: Sender<MainServer>,
    pub world: Vec<((f64, f64), String, String, i32, i32, i32, f64, Sender<BeastUpdate>)>,
    pub entries: i32, 
}
