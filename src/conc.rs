//use crate::beast_traits::Beast;
use crate::mpsc::{Sender/*,Receiver*/};
pub struct Msg {
    pub id: String,
    pub alive: bool,
    pub beast: String,
    pub pos: (f64, f64),
    pub dir: i32,
    pub speed: f64,
    pub handle: Sender<BeastUpdate>,
}

pub struct BeastUpdate {
    pub kill: bool,
    pub world: Vec<((f64, f64), String, String, i32, f64, Sender<BeastUpdate>)>, 
}

//todo impl Msg for many msg types