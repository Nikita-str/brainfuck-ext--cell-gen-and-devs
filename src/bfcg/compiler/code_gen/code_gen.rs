use crate::bfcg::compiler::valid_cmd::ValidCMD;


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