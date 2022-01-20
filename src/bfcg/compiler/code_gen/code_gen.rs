use crate::bfcg::compiler::valid_cmd::ValidCMD;


/// return maximum by number of commands code gen for cell value setting
pub fn cgen_set_cell_max_cmds() -> String{ cgen_set_cell_value(0xFF, true) }

pub fn cgen_set_cell_value(value: u8, need_nullify: bool) -> String{
    let mut ret = String::new();
    add_cgen_set_cell_value(& mut ret, value, need_nullify);
    ret
}

pub fn add_cgen_set_cell_value(cgen: &mut String, mut value: u8, need_nullify: bool) {
    if need_nullify { cgen.push(ValidCMD::ZeroedCell.std_to_char()); }

    if value > 0 { 
        // just delete excess left-shift ('*')
        cgen.push(ValidCMD::IncValue.std_to_char()); 
        value >>= 1;
    }
    
    while value > 0 {
        cgen.push(ValidCMD::LeftShift.std_to_char());
        if (value & 1) == 1 { cgen.push(ValidCMD::IncValue.std_to_char()); }
        value >>= 1;
    }    
}


pub fn cgen_shift_cell_ptr(shift: i32) -> String{
    if shift == 0 { String::new() }
    else if shift > 0 { str::repeat(&ValidCMD::NextCell.std_to_char().to_string(), shift as usize) }
    else { str::repeat(&ValidCMD::PrevCell.std_to_char().to_string(), (-shift) as usize) }
}



pub fn add_cgen_cell_create(cgen: &mut String) { cgen.push(ValidCMD::CreateCell.std_to_char()); }

/// # code generate: 
/// ### before
/// CEM: `...0XX...XY` where each X and Y is any number(from 1 to 0xFF) != 0
/// 
/// CEM ptr is on Y 
/// ### after
/// CEM: `...000...00`
/// 
/// CEM ptr is on first 0
pub fn cgen_zero_while_not_zero() -> String {
    let mut ret = String::new();
    add_cgen_zero_while_not_zero(&mut ret);
    ret
}

pub fn add_cgen_zero_while_not_zero(cgen: &mut String) {
    cgen.push(ValidCMD::StartWhileNZ.std_to_char());
    cgen.push(ValidCMD::ZeroedCell.std_to_char());
    cgen.push(ValidCMD::PrevCell.std_to_char());
    cgen.push(ValidCMD::EndWhileNZ.std_to_char());
}

/// # code generate: 
/// ### before
/// CEM: `...0X[1]X[2]...X[N]` where each X\[i\] is any number(from 1 to 0xFF) != 0
/// 
/// CEM ptr is on X\[i\] 
/// ### after
/// CEM: `...0X[1]X[2]...X[N]`
/// 
/// CEM ptr is on X\[1\]
pub fn cgen_move_to_next_after_left_zero() -> String {
    let mut ret = String::new();
    add_cgen_move_to_next_after_left_zero(&mut ret);
    ret
}

pub fn add_cgen_move_to_next_after_left_zero(cgen: &mut String) {
    cgen.push(ValidCMD::StartWhileNZ.std_to_char());
    cgen.push(ValidCMD::PrevCell.std_to_char());
    cgen.push(ValidCMD::EndWhileNZ.std_to_char());
    cgen.push(ValidCMD::NextCell.std_to_char());
}

pub fn cgen_move_to_prev_before_right_zero() -> String {
    let mut ret = String::new();
    add_cgen_move_to_prev_before_right_zero(&mut ret);
    ret
}

pub fn add_cgen_move_to_prev_before_right_zero(cgen: &mut String) {
    cgen.push(ValidCMD::StartWhileNZ.std_to_char());
    cgen.push(ValidCMD::NextCell.std_to_char());
    cgen.push(ValidCMD::EndWhileNZ.std_to_char());
    cgen.push(ValidCMD::PrevCell.std_to_char());
}


pub fn add_cgen_init_se_cem<T>(cgen: &mut String, se: T, need_nullify: bool) 
where T: IntoIterator<Item = u8>
{
    let mut last = false;
    for se_num in se {
        if last { panic!("incorrect se seq") }
        add_cgen_set_cell_value(cgen, se_num, need_nullify);
        if se_num < 0x80 { last = true; }
        if !last { cgen.push(ValidCMD::NextCell.std_to_char()) }
    }
}
