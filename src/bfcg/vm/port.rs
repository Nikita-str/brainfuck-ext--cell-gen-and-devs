
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Port{
    Number(usize),
    Name(String),
    Any,
}


impl Port{
    pub fn new_any() -> Self { Self::Any }

    pub fn right_port_name(port_name: &str) -> bool {
        port_name.len() > 0 
        &&
        port_name.chars().all(|c|c.is_ascii_lowercase())
    } 

    /// ## panic
    /// if port is not usize & !right_port_name(port)  
    pub fn new(port: &str) -> Self{
        if let Ok(number) = port.parse() { Self::Number(number) }
        else if Self::right_port_name(port) { Self::Name(port.to_owned()) }
        else { panic!("wrong port name") } 
    }

    pub fn to_name(self) -> String {
        if let Self::Name(x) = self { x }
        else { panic!("port is not name") }
    }
}