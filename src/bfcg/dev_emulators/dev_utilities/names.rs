
/// right dev name: `[a-z][a-z0-9\-]*`
pub fn right_std_dev_name(dev_name: &str) -> bool {
    dev_name.chars().next().map_or(false, |c| c.is_ascii_lowercase())  
    &&
    dev_name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || (c == '-')) 
}

