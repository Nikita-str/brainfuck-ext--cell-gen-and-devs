use bf_cell_gen::bfcg::compiler::compiler_error::CompilerError;
use bf_cell_gen::bfcg::compiler::compiler_info::CompilerInfo;
use bf_cell_gen::bfcg::compiler::valid_cmd::ValidCMD;
use bf_cell_gen::bfcg::compiler::{
    compiler, 
    comand_compiler, 
    compiler_pos::CompilerPos,
    mnc_checker::HolderChekerMNC, 
    compiler_error::CompilerErrorType, 
};
use bf_cell_gen::bfcg::compiler::dif_part_helper::setting_action::SettingActions;
use bf_cell_gen::bfcg::compiler::compiler_option::{CompilerOption, MemInitType};


/// light compile means: empty def settings; empty MNC Holder; Interpreter CC;
fn get_result_light_compile(path: &str) -> Result<CompilerInfo<ValidCMD>, CompilerError>{
    let mem_init_type = MemInitType::BeforeCode;
    
    let mut set_act = SettingActions::new();
    SettingActions::add_std_actions(&mut set_act, mem_init_type);
    
    let empty_mnc_holder_checker = HolderChekerMNC::new();

    let option = CompilerOption::new(
        mem_init_type,
        comand_compiler::InterpreterCmdCompiler::new(),
        &set_act,
        vec![],
        &empty_mnc_holder_checker,
    );

    compiler::compile(path.to_owned(), option, None)
}

#[test]
fn compiler_test_error_01(){
    let path = "examples/compile_test/must_error/while_error_01.bf-ext";
    let result = get_result_light_compile(path);

    if let Err(x) = result { 
        if let CompilerErrorType::ClosedWhileWithoutOpen = x.err_type {  
            let stack = x.err_stack_pos;
            assert_eq!(stack.len(), 1);
            assert_eq!(stack[0].pos, Some(CompilerPos{ line:5, symb: 9 }));
        }
        else { panic!("must be CET::ClosedWhileWithoutOpen") }
    } else { panic!("must be error!") }
}


#[test]
fn compiler_test_ok_01() {
    let path = "examples/compile_test/must_success/while_ok_01.bf-ext";
    let result = get_result_light_compile(path);
    if let Err(_) = result { panic!("must be ok!") }
    let result = result.ok().unwrap();
    assert_eq!(result.get_ref_program().len(), 40);
}