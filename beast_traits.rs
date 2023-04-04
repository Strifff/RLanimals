


pub trait Beast{
    fn set_id(&mut self, id: String);
    fn get_id(&self) -> String; 

    fn set_pos(&mut self, pos: (i32,i32));
    fn get_pos(&self) -> (i32,i32);

    fn set_dir(&mut self, dir: i32);
    fn get_dir(&self) -> i32;

    //fn set_speed()

}