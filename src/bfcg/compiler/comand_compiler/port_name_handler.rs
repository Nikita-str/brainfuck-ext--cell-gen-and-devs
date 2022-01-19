use std::collections::HashMap;

pub trait PortNameHandler{
    fn port_name_handle(&mut self, port_names: &HashMap<String, usize>);
}

pub struct NullPortNameHandler{}
impl PortNameHandler for NullPortNameHandler{ 
    fn port_name_handle(&mut self, _: &HashMap<String, usize>) { }
}
