use std::io::Write;

use bf_cell_gen::bfcg::compiler::{compiler_pos::CompilerPos, compiler_error::CompilerErrorType, };
use bf_cell_gen::bfcg::compiler::compiler_option::{MemInitType};
use bf_cell_gen::bfcg::disasm::std_disasm::{std_disasm, StdDisasmInfo};
use bf_cell_gen::bfcg::vm::hardware_info::HardwareInfo;
use bf_cell_gen::logistics;


#[test]
fn compiler_test_error_01(){
    let path = "examples/compile_test/must_error/while_error_01.bf-ext";

    let result = 
    logistics::interpreter_compile(path, MemInitType::BeforeCode, None);

    if let Err(x) = result { 
        if let CompilerErrorType::ClosedWhileWithoutOpen = x.err_type {  
            let stack = x.err_stack_pos;
            assert_eq!(stack.len(), 1);
            assert_eq!(stack[0].pos, Some(CompilerPos{ line:5, symb: 9, blocked_pos: 0 }));
        }
        else { panic!("must be CET::ClosedWhileWithoutOpen") }
    } else { panic!("must be error!") }
}


#[test]
fn compiler_test_ok_01() {
    let path = "examples/compile_test/must_success/while_ok_01.bf-ext";

    let result = 
    logistics::interpreter_compile(path, MemInitType::BeforeCode, None);

    if let Err(_) = result { panic!("must be ok!") }
    let result = result.ok().unwrap();
    assert_eq!(result.get_ref_program().len(), 40);
}


#[test]
#[allow(unused)]
fn compiler_test_u8_std_cmd_01() {
    let path = "examples/compile_test/must_success/opcode_std_cmd_01.bf-ext";

    let hardware_info = HardwareInfo{ max_port_amount: 64, max_jump_size: 1 << 16, default_cem_port: 1, default_com_port: 2, };

    let result = 
    logistics::std_compile(path, MemInitType::BeforeCode, None, &hardware_info);

    if let Err(_) = result { panic!("must be ok"); } 

    let ok = result.ok().unwrap();
    std::fs::create_dir_all("target/tmp");
    let code = ok.get_ref_program();

    let mut disasm_info = StdDisasmInfo::new();
    disasm_info.std_init();
    let disasm = std_disasm(code.into_iter(), &disasm_info); 
    if let Ok(x) = disasm { 
        //let mut file = std::fs::File::create("examples/compile_test/must_success/disasm_by_compiler_test_u8_std_cmd_01.disasm").ok().unwrap();
        let mut file = std::fs::File::create("target/tmp/u8_std_cmd_01.disasm").ok().unwrap();
        if file.write(x.as_bytes()).is_err() { panic!("cant write in file") };
    } else if let Err(err) = disasm {
        println!("DIS ASM ERROR: {}", err);
    }

    let mut file = std::fs::File::create("target/tmp/u8_std_cmd_01.bin").ok().unwrap();
    if file.write_all(code).is_err() { panic!("cant write in file") };
}


