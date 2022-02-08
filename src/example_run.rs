// screen test can't be run by #[test]
// cause in #[test] used not main thread 
// so here just some win-run 

use crate::{logistics, MemInitType, HardwareInfo};



pub fn compiler_test_u8_exmp_02() {
    let path = "examples/examples/02_write_const_str_in_win.bf-ext";
    let hw_info = HardwareInfo{ max_port_amount: 64, max_jump_size: 1 << 16, default_cem_port: 1, default_com_port: 2, };

    let compiled = logistics::std_compile(path, MemInitType::BeforeCode, None, &hw_info);
    
    let c_info = 
    match compiled {
        Err(err) => { logistics::compiler_error_std_out(&err); return },
        Ok(x) => x,
    };

    let logi_run = logistics::LogisticRun::new(c_info);
    logistics::vm_run(&hw_info, logi_run);
}