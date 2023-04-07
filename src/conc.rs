use crate::beast_traits::Beast;
pub struct Msg {
    pub id: String,
    pub beast: String,
    pub pos: (f64, f64),
    pub dir: i32,
    pub speed: f64,
}