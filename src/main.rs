
use bf_cell_gen::{logistics, MemInitType, HardwareInfo};
//use bf_cell_gen::example_run::compiler_test_u8_exmp_02;



fn main() {
    let path = "examples/examples/03_win_print_alphabet.bf-ext";
    let hw_info = HardwareInfo{ max_port_amount: 64, max_jump_size: 1 << 18, default_cem_port: 1, default_com_port: 2, };

    let compiled = logistics::std_compile(path, MemInitType::BeforeCode, None, &hw_info);
    
    let c_info = 
    match compiled {
        Err(err) => { logistics::compiler_error_std_out(&err); return },
        Ok(x) => x,
    };

    let logi_run = logistics::LogisticRun::new(c_info);
    logistics::vm_run(&hw_info, logi_run);
}
