use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn file_minimalize(str: &str) -> Result<String, ()>{
    let mut comment_until_next_line = false;
    let mut err = None;

    let ret = str.chars().filter(|c|{
        if err.is_some() { return false }
        match c {
            ';' => { 
                comment_until_next_line = true; 
                false
            }
            x if x.is_ascii_whitespace() => {
                if *x == '\n' { comment_until_next_line = false; }
                false
            }
            x if x.is_ascii_graphic() => {
                !comment_until_next_line
            }
            _ => { 
                if !comment_until_next_line { err = Some(()); } 
                false 
            }
        }
    }).collect();

    if err.is_some() { return Err(err.unwrap()) }

    Ok(ret)
}

// TODO: type of error insted of (); if return None => no errors
pub fn parse(file_name: &str) -> Result<(), ()>{
    let file = File::open(file_name);
    if file.is_err() { return Err(()) }
    let file = file.unwrap();
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        if line.is_err() { return Err(()) }
        let line = line.unwrap();
        // TODO: parse : [STOP HERE]
    }

    Ok(())
}