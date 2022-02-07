use std::io::Write;

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
use bf_cell_gen::bfcg::dev_emulators::cem::DevStdCem;
use bf_cell_gen::bfcg::dev_emulators::com::DevStdCom;
use bf_cell_gen::bfcg::dev_emulators::std_dev_constructor;
use bf_cell_gen::bfcg::disasm::std_disasm::{std_disasm, StdDisasmInfo};
use bf_cell_gen::bfcg::vm::{StdProcessor, Port};
use bf_cell_gen::bfcg::vm::hardware_info::HardwareInfo;
use bf_cell_gen::bfcg::vm::std_processor::{AddDeviceOk, ProcessorRunResult};


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


#[test]
#[allow(unused)]
fn compiler_test_u8_std_cmd_01() {
    let path = "examples/compile_test/must_success/opcode_std_cmd_01.bf-ext";

    let mem_init_type = MemInitType::BeforeCode;
    let mut set_act = SettingActions::new();
    SettingActions::add_std_actions(&mut set_act, mem_init_type);

    let empty_mnc_holder_checker = HolderChekerMNC::new();
    let hardware_info = HardwareInfo{ max_port_amount: 64, max_jump_size: 1 << 16, default_cem_port: 1, default_com_port: 2, };

    let option = CompilerOption::new(
        mem_init_type,
        comand_compiler::StdCmdCompiler::new(&hardware_info),
        &set_act,
        vec![],
        &empty_mnc_holder_checker,
    );

    let result = compiler::compile(path.to_owned(), option, None);
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


#[test]
fn compiler_test_u8_helwo_wowld() {
    let path = "examples/examples/helwo_demv_wowld.bf-ext";

    let mem_init_type = MemInitType::BeforeCode;
    let mut set_act = SettingActions::new();
    SettingActions::add_std_actions(&mut set_act, mem_init_type);

    let empty_mnc_holder_checker = HolderChekerMNC::new();
    let hardware_info = HardwareInfo{ max_port_amount: 64, max_jump_size: 1 << 16, default_cem_port: 1, default_com_port: 2, };

    let option = CompilerOption::new(
        mem_init_type,
        comand_compiler::StdCmdCompiler::new(&hardware_info),
        &set_act,
        vec![],
        &empty_mnc_holder_checker,
    );

    let result = compiler::compile(path.to_owned(), option, None);
    if let Err(x) = result {
        println!("{}\n\n", x.to_string()); 
        panic!("must be ok"); 
    } 
    println!("\n");

    let ok = result.ok().unwrap();

    let mut file = std::fs::File::create("target/tmp/u8_helwo_wowld.bin").ok().unwrap();
    if file.write_all(ok.get_ref_program()).is_err() { panic!("cant write in file") };
    
    let mut disasm_info = StdDisasmInfo::new();
    disasm_info.std_init();
    let disasm = std_disasm(ok.get_ref_program().into_iter(), &disasm_info).ok().unwrap();
    let mut file = std::fs::File::create("target/tmp/u8_helwo_wowld.disasm").ok().unwrap();
    if file.write_all(disasm.as_bytes()).is_err() { panic!("cant write in file") };

    let mut processor = StdProcessor::new(
        hardware_info.max_port_amount, 
        hardware_info.default_com_port, 
        hardware_info.default_cem_port
    );

    for (port, dev_name) in ok.get_devs() {
        let x = std_dev_constructor(dev_name);
        if x.is_err() { panic!("Device Constructor Error: {}", x.err().unwrap().to_string()) }
        let dev = x.ok().unwrap();
        if !dev.warns.is_empty() { panic!("not empty warns") }
        
        let port_num = match port {
            Port::Number(x) => *x,
            Port::Name(x) =>  ok.get_port(x).unwrap(),
            _ => { 
                println!("PANIC AT DEV: {:?}", dev_name.get_name());
                println!("PANIC AT PORT: {:?}", port);
                panic!("no now") 
            }
        };

        let x = processor.add_device_boxed(dev.dev, port_num);
        if let Err(_) = x { panic!("must be ok") }
        if let AddDeviceOk::OldDevDisconected = x.ok().unwrap() { panic!("must be ::Ok") }

    }


    let mut com = Box::new(DevStdCom::new(0x10_00));    
    com.init(ok.get_ref_program().iter());
    com.move_to_start();

    let cem = Box::new(DevStdCem::new(0x10_00, 0x10_00));

    let x = processor.add_device_boxed(com, hardware_info.default_com_port);
    if let Err(_) = x { panic!("must be ok") }
    if let AddDeviceOk::OldDevDisconected = x.ok().unwrap() { panic!("must be ::Ok") }

    let x = processor.add_device_boxed(cem, hardware_info.default_cem_port);
    if let Err(_) = x { panic!("must be ok") }
    if let AddDeviceOk::OldDevDisconected = x.ok().unwrap() { panic!("must be ::Ok") }

    println!("[+] RUN");
    println!("-----------------------");
    let z = processor.run();
    if let ProcessorRunResult::Ok = z { }
    else { println!("{}", z.to_string()); panic!() }
    println!("-----------------------");
    println!("[-] RUN");
}


// WIN example can't run from not main thread, but in test used not main thread => can't run win-example as test
// so check example_run.rs


#[test]
fn compiler_test_u8_sugar_mul() {
    let path = "examples/proof_of_sugarity_some_cmds/proof_mul.bf-ext";

    let mem_init_type = MemInitType::BeforeCode;
    let mut set_act = SettingActions::new();
    SettingActions::add_std_actions(&mut set_act, mem_init_type);

    let empty_mnc_holder_checker = HolderChekerMNC::new();
    let hardware_info = HardwareInfo{ max_port_amount: 64, max_jump_size: 1 << 16, default_cem_port: 1, default_com_port: 2, };

    let option = CompilerOption::new(
        mem_init_type,
        comand_compiler::StdCmdCompiler::new(&hardware_info),
        &set_act,
        vec![],
        &empty_mnc_holder_checker,
    );

    let result = compiler::compile(path.to_owned(), option, None);
    if let Err(x) = result {
        println!("{}\n\n", x.to_string()); 
        panic!("must be ok"); 
    } 
    println!("\n");

    let ok = result.ok().unwrap();

    let mut processor = StdProcessor::new(
        hardware_info.max_port_amount, 
        hardware_info.default_com_port, 
        hardware_info.default_cem_port
    );

    for (port, dev_name) in ok.get_devs() {
        let x = std_dev_constructor(dev_name);
        if x.is_err() { panic!("Device Constructor Error: {}", x.err().unwrap().to_string()) }
        let dev = x.ok().unwrap();
        if !dev.warns.is_empty() { panic!("not empty warns") }
        
        let port_num = match port {
            Port::Number(x) => *x,
            Port::Name(x) =>  ok.get_port(x).unwrap(),
            _ => { 
                println!("PANIC AT DEV: {:?}", dev_name.get_name());
                println!("PANIC AT PORT: {:?}", port);
                panic!("no now") 
            }
        };

        let x = processor.add_device_boxed(dev.dev, port_num);
        if let Err(_) = x { panic!("must be ok") }
        if let AddDeviceOk::OldDevDisconected = x.ok().unwrap() { panic!("must be ::Ok") }

    }


    let mut com = Box::new(DevStdCom::new(0x10_00));    
    com.init(ok.get_ref_program().iter());
    com.move_to_start();

    let cem = Box::new(DevStdCem::new(0x10_00, 0x10_00));

    let x = processor.add_device_boxed(com, hardware_info.default_com_port);
    if let Err(_) = x { panic!("must be ok") }
    if let AddDeviceOk::OldDevDisconected = x.ok().unwrap() { panic!("must be ::Ok") }

    let x = processor.add_device_boxed(cem, hardware_info.default_cem_port);
    if let Err(_) = x { panic!("must be ok") }
    if let AddDeviceOk::OldDevDisconected = x.ok().unwrap() { panic!("must be ::Ok") }

    println!("[+] RUN");
    println!("-----------------------");
    let z = processor.run();
    if let ProcessorRunResult::Ok = z { }
    else { println!("{}", z.to_string()); panic!() }
    println!("\n-----------------------");
    println!("[-] RUN");
}
