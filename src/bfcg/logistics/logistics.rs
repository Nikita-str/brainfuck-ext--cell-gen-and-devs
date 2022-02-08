use std::collections::{HashMap, HashSet};
use std::io::Write;
use crate::bfcg::compiler::comand_compiler;
use crate::bfcg::compiler::compiler_error::CompilerError;
use crate::bfcg::compiler::compiler_option::CompilerOption;
use crate::bfcg::compiler::compiler_warning::CompilerWarnings;
use crate::bfcg::compiler::dif_part_helper::setting_action::SettingActions;
use crate::bfcg::compiler::mnc_checker::HolderChekerMNC;
use crate::bfcg::compiler::{compiler_info::CompilerInfo, compiler_option::MemInitType};
use crate::bfcg::dev_emulators::dev_constructor::DevCtorOk;
use crate::bfcg::dev_emulators::dev_name::DevName;
use crate::bfcg::dev_emulators::std_dev::{get_std_win_dev_type, std_win_spec_constructor};
use crate::bfcg::dev_emulators::std_dev_constructor;
use crate::bfcg::disasm::{std_disasm::std_disasm, std_disasm::StdDisasmInfo};
use crate::bfcg::general::PAD;
use crate::bfcg::vm::std_processor::{AddDeviceOk, AddDeviceErr, ProcessorRunResult};
use crate::bfcg::vm::{HardwareInfo, StdProcessor, Port};

macro_rules! ok_transform { 
    ($x:ident) => { if let Ok(x) = $x { x } else { return Err(()) } };
    ($x:expr) => { if let Ok(x) = $x { x } else { return Err(()) } };
}

macro_rules! none_transform { ($x:expr) => { if let Err(_) = $x { return } }; }




pub fn gen_binary(cinfo: &CompilerInfo<u8>, path: &str) -> Result<(), ()> {
    let mut file = ok_transform!(std::fs::File::create(path));
    let ret = ok_transform!(file.write_all(cinfo.get_ref_program())); 
    Ok(ret)
}


pub fn gen_disasm_std(cinfo: &CompilerInfo<u8>, path: &str) -> Result<(), ()> {
    let mut disasm_info = StdDisasmInfo::new();
    disasm_info.std_init();

    let disasm = ok_transform!(std_disasm(cinfo.get_ref_program().into_iter(), &disasm_info));
    let mut file = ok_transform!(std::fs::File::create(path));
    let ret = ok_transform!(file.write_all(disasm.as_bytes()));
    Ok(ret)
}

pub fn std_compile(
    path: &str,
    mem_init_type: MemInitType, 
    checker: HolderChekerMNC, 
    hardware_info: &HardwareInfo
) 
-> Result<CompilerInfo<u8>, CompilerError> 
{
    let mut set_act = SettingActions::new();
    SettingActions::add_std_actions(&mut set_act, mem_init_type);

    let option = CompilerOption::new(
        mem_init_type,
        comand_compiler::StdCmdCompiler::new(&hardware_info),
        &set_act,
        vec![],
        &checker,
    );

    crate::bfcg::compiler::compiler::compile(path.to_owned(), option, None)
}

pub fn device_ctor_warn_std_out(dev_ctor: &DevCtorOk, dev_name: &DevName) {
    if !dev_ctor.warns.is_empty() {
        println!("device <{}> construct WARNING:", dev_name.to_string());
        for x in &dev_ctor.warns { println!("{}{}", PAD, x.to_string()) }
    } 
}
pub fn compiler_error_std_out(err: &CompilerError) { println!("COMPILE ERROR:\n{}", err.to_string()); }
pub fn compiler_warn_std_out(warns: &CompilerWarnings) { println!("COMPILE WARNINGS:\n{}", warns.to_string()); }


fn device_connecting(processor: &mut StdProcessor, dev_ctor: DevCtorOk, dev_name: &DevName, port_num: usize) -> Result<(), ()> {
    match processor.add_device_boxed(dev_ctor.dev, port_num) {
        Ok(AddDeviceOk::Ok) => {}
        Ok(AddDeviceOk::OldDevDisconected) => { 
            println!("device <{}> connecting WARNING: old device disconected", dev_name.to_string()); 
        }
        Err(AddDeviceErr::TooBigPortNum) => {
            println!("device <{}> connecting ERROR: too big port number {} | help: just change hardware info", dev_name.to_string(), port_num); 
            return Err(())
        }
    }
    Ok(())
}

pub fn vm_run(
    hw_info: &HardwareInfo,
    logi_run: LogisticRun<u8>,
) // TODO : NEED RET : PORT ERROR + RUN RES
{
    let mut processor = StdProcessor::from_hardware_info(hw_info);
    
    let mut win = None;
    let mut win_port = None;
    let mut already_screen = false;
    let mut need_run_win = false;
    let mut win_device_type = vec![];

    let mut com_code_init = false;
    let mut cem_code_init = false;

    let mut busy_port = HashSet::new();
    let mut dev_awaiting_for_connection = vec![];

    'dev_for:
    for (port, dev_name) in logi_run.get_devs() {
        // ---------------------------------------
        // [+] PORT:
        let port_num = match port {
            Port::Number(x) => Some(*x),
            Port::Name(x) =>  {
                if Port::more_than_one_uni_port_name(x) {
                    match Port::port_name_uniform(x).as_str() {
                        "com" => { com_code_init = true; }
                        "cem" => { cem_code_init = true; }
                        _ => {}
                    }

                }
                // TODO:CEM+COM
                logi_run.get_port(x)
            }
            Port::Any => None,
        };
        // [-] PORT
        // ---------------------------------------

        // ---------------------------------------
        // [+] DEV CTOR
        match std_dev_constructor(dev_name) {
            Ok(dev_ctor) => {
                device_ctor_warn_std_out(&dev_ctor, dev_name);

                if let Some(port_num) = port_num {
                    busy_port.insert(port_num);
                    none_transform!(device_connecting(&mut processor, dev_ctor, dev_name, port_num));
                } else {
                    dev_awaiting_for_connection.push(dev_ctor);
                }
                continue 'dev_for
            }
            Err(err) => {
                if let Some(x) = get_std_win_dev_type(dev_name) {
                    if already_screen && x.is_screen() { 
                        println!("device <{}> construct ERROR:\nscreen alredy connected!\ncurrently you can connect only one screen.", dev_name.to_string());
                        return 
                    } 
                    if x.is_screen() { 
                        win_port = port_num;
                        if let Ok(x) = std_win_spec_constructor(dev_name) { win = Some(x) }
                        else { panic!("must never happen") }
                        already_screen = true; 
                    }
                    need_run_win = true;
                    win_device_type.push(x);
                } else {
                    println!("device <{}> construct ERROR: {}", dev_name.to_string(), err.to_string());
                    return
                }
            }
        }
        // [-] DEV CTOR
        // ---------------------------------------
    }


    // TODO: HERE CONNECT REST

    let mut run_f = move||{
        println!("[+] RUN");
        println!("-----------------------");
        let z = processor.run();
        println!("\n-----------------------");
        if let ProcessorRunResult::Ok = z { 
            println!("[-] RUN");
        }
        else { 
            println!("[!!!]: {}", z.to_string());
        }
    };

    if need_run_win {
        if win.is_none() {
            println!("ERROR: some of win-related-device is connected but screen is not!");
            return
        }

        #[cfg(test)]
        { 
            println!("!!! YOU CANNOT RUN THIS AS TEST(run it from main):");
            println!("for running this code screen is needed but #[test] is not \"main thread\" therefore it cant be run on all platforms."); 
        }

        let win = win.unwrap();
        win.run(run_f); 
    } else {
        run_f();
    }
}


pub struct LogisticRun<T>{
    program: Vec<T>,
    port_names: HashMap<String, usize>,
    devs: HashMap<Port, DevName>,
}

impl<T> LogisticRun<T>{
    pub fn new(cinfo: CompilerInfo<T>) -> Self{
        let (port_names, devs, program) = cinfo.decompile();
        Self { port_names, devs, program }
    }

    pub fn get_devs(&self) -> &HashMap<Port, DevName> { &self.devs }
    pub fn get_port(&self, port_name: &str) -> Option<usize> { if let Some(x) = self.port_names.get(port_name) { Some(*x) } else { None } }
}