
pub trait Beast{
    fn set_id(&mut self, id: String);
    fn get_id(&self)                        -> String; 

    fn set_pos(&mut self, pos: (f32,f32));
    fn get_pos(&self)                       -> (f32,f32);

    fn set_dir(&mut self, dir: i32);
    fn get_dir(&self)                       -> i32;

    fn set_speed1(&mut self);
    fn set_speed2(&mut self);
    fn set_speed3(&mut self);
    fn get_speed(&self)                     -> f32;

    fn forward(&mut self);
    fn left(&mut self);
    fn right(&mut self);
    fn back(&mut self);
    fn in_bounds(&self, x: f32, y: f32)     -> (f32,f32);

    fn consume_energy(&mut self);
    fn starve(&mut self);
    fn kill(&mut self);
}