pub trait Beast{
    fn set_id(&mut self, id: String);
    fn get_id(&self)                        -> String; 

    fn set_pos(&mut self, pos: (f64,f64));
    fn get_pos(&self)                       -> (f64,f64);

    fn set_dir(&mut self, dir: i32);
    fn get_dir(&self)                       -> i32;

    fn set_speed1(&mut self);
    fn set_speed2(&mut self);
    fn set_speed3(&mut self);
    fn get_speed(&self)                     -> f64;

    fn forward(&mut self);
    fn left(&mut self);
    fn right(&mut self);
    fn back(&mut self);
    fn in_bounds(&self, x: f64, y: f64)     -> (f64,f64);

    fn get_fov(&self)                       -> i32; 
    fn get_ros(&self)                       -> i32;

    fn consume_energy(&mut self);
    fn starve(&mut self);
    fn kill(&mut self)                      -> bool;
}