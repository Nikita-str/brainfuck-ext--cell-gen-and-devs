// screen test can't be run by #[test]
// cause in #[test] used not main thread 
// so here just some win-run 


use std::io::Write;

use crate::bfcg::compiler::{
    compiler, 
    comand_compiler, 
    mnc_checker::HolderChekerMNC, 
};
use crate::bfcg::compiler::dif_part_helper::setting_action::SettingActions;
use crate::bfcg::compiler::compiler_option::{CompilerOption, MemInitType};
use crate::bfcg::dev_emulators::cem::DevStdCem;
use crate::bfcg::dev_emulators::com::DevStdCom;
use crate::bfcg::dev_emulators::std_dev_constructor;
use crate::bfcg::dev_emulators::win::Win;
use crate::bfcg::dev_emulators::win::dev_win::DevWin;
use crate::bfcg::disasm::std_disasm::{std_disasm, StdDisasmInfo};
use crate::bfcg::vm::{StdProcessor, Port};
use crate::bfcg::vm::hardware_info::HardwareInfo;
use crate::bfcg::vm::std_processor::{AddDeviceOk, ProcessorRunResult};



pub fn compiler_test_u8_exmp_02() {
    let path = "examples/examples/02_write_const_str_in_win.bf-ext";

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

    let mut file = std::fs::File::create("target/tmp/02_write_const_str_in_win.bin").ok().unwrap();
    if file.write_all(ok.get_ref_program()).is_err() { panic!("cant write in file") };
    /* 
    
    let mut disasm_info = StdDisasmInfo::new();
    disasm_info.std_init();
    let disasm = std_disasm(ok.get_ref_program().into_iter(), &disasm_info).ok().unwrap();
    let mut file = std::fs::File::create("target/tmp/02_write_const_str_in_win.disasm").ok().unwrap();
    if file.write_all(disasm.as_bytes()).is_err() { panic!("cant write in file") };
    */

    let mut processor = StdProcessor::new(
        hardware_info.max_port_amount, 
        hardware_info.default_com_port, 
        hardware_info.default_cem_port
    );

    let mut win = None;
    let mut win_port = None;
    for (port, dev_name) in ok.get_devs() {
        if dev_name.get_name() == "std-win" {
            let w = dev_name.get_param("w").unwrap().parse().unwrap();
            let h = dev_name.get_param("h").unwrap().parse().unwrap();
            win = Some(Win::new(w, h));
            
            win_port = Some(match port {
            Port::Number(x) => *x,
            Port::Name(x) =>  ok.get_port(x).unwrap(),
            _ => panic!(),
            });
            continue
        }

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

    let mut com = Box::new(DevStdCom::new(0x10_00_00));    
    com.init(ok.get_ref_program().iter());
    com.move_to_start();

    let cem = Box::new(DevStdCem::new(0x10_00, 0x10_00));

    let x = processor.add_device_boxed(com, hardware_info.default_com_port);
    if let Err(_) = x { panic!("must be ok") }
    if let AddDeviceOk::OldDevDisconected = x.ok().unwrap() { panic!("must be ::Ok") }

    let x = processor.add_device_boxed(cem, hardware_info.default_cem_port);
    if let Err(_) = x { panic!("must be ok") }
    if let AddDeviceOk::OldDevDisconected = x.ok().unwrap() { panic!("must be ::Ok") }

    assert!(win.is_some());
    let mut win = win.unwrap();
    let win_dev = Box::new(DevWin::new(&mut win));

    let x = processor.add_device_boxed(win_dev, win_port.unwrap());
    if let Err(_) = x { panic!("must be ok") }
    if let AddDeviceOk::OldDevDisconected = x.ok().unwrap() { panic!("must be ::Ok") }

    let run_f = move||{
        println!("[+] RUN");
        println!("-----------------------");
        let z = processor.run();
        if let ProcessorRunResult::Ok = z { }
        else { println!("{}", z.to_string()); panic!() }
        println!("\n-----------------------");
        println!("[-] RUN");
    };

    win.run(run_f);
}
