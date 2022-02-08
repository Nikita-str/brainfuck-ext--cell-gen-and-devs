
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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

    pub fn more_than_one_uni_port_name(port_name: &str) -> bool {
        match port_name {
            "win" | "display" | "screen" | "window" => true,
            "com" | "cmd-mem" | "command-memory" => true,
            "cem" | "cell-mem" | "cell-memory" => true,
            _ => false,
        }
    }

    pub fn  port_name_uniform(port_name: &str) -> String{
        match port_name {
            "win" | "display" | "screen" | "window" => String::from("screen"),
            "com" | "cmd-mem" | "command-memory" => String::from("com"),
            "cem" | "cell-mem" | "cell-memory" => String::from("cem"),
            _ => port_name.to_string(),
        }
    }

}


impl ToString for Port {
    fn to_string(&self) -> String {
        match self {
            Self::Number(x) => format!("[{}]", x), 
            Self::Name(x) => format!("[{}]", x), 
            Self::Any => format!("[ANY]"), 
        }
    }
}