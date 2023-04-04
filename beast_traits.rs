


pub trait Beast{
    fn set_id(&mut self, id: String);
    fn get_id(&self)                        -> String; 

    fn set_pos(&mut self, pos: (i32,i32));
    fn get_pos(&self)                       -> (i32,i32);

    fn set_dir(&mut self, dir: i32);
    fn get_dir(&self)                       -> i32;

    fn set_speed1(&mut self);
    fn set_speed2(&mut self);
    fn set_speed3(&mut self);
    fn get_speed(&self)                     -> i32;

    fn forward(&mut self)                   -> (i32,i32);
    fn left(&mut self)                      -> (i32,i32);
    fn right(&mut self)                     -> (i32,i32);
    fn back(&mut self)                      -> (i32,i32);
}