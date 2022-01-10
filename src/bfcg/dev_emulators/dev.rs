
pub trait Dev{
    // TODO: if need add [mut_]{read_byte, write_byte}

    fn read_byte(&mut self) -> u8;
    fn write_byte(&mut self, byte: u8);

    fn test_can_read_byte(&self) -> bool;

    fn have_error(&self) -> bool;
}


/// right dev name: `[a-z][a-z0-9\-]*`
pub fn right_std_dev_name(dev_name: &str) -> bool {
    dev_name.chars().next().map_or(false, |c| c.is_ascii_lowercase())  
    &&
    dev_name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || (c == '-')) 
}