use std::collections::{HashMap, HashSet};
use std::io::Write;

use crate::LogisticParams;
use crate::bfcg::compiler::comand_compiler::{self, PortNameHandler};
use crate::bfcg::compiler::compiler_error::CompilerError;
use crate::bfcg::compiler::compiler_option::CompilerOption;
use crate::bfcg::compiler::compiler_warning::CompilerWarnings;
use crate::bfcg::compiler::dif_part_helper::setting_action::SettingActions;
use crate::bfcg::compiler::mnc_checker::HolderChekerMNC;
use crate::bfcg::compiler::valid_cmd::ValidCMD;
use crate::bfcg::compiler::{compiler_info::CompilerInfo, compiler_option::MemInitType};
use crate::bfcg::dev_emulators::cem::DevStdCem;
use crate::bfcg::dev_emulators::cem::std_cem::{DEFAULT_CEM_MM_SIZE, DEFAULT_CEM_AM_SIZE};
use crate::bfcg::dev_emulators::com::DevStdCom;
use crate::bfcg::dev_emulators::com::com::DEFAULT_COM_MEM_SIZE;
use crate::bfcg::dev_emulators::dev_constructor::DevCtorOk;
use crate::bfcg::dev_emulators::dev_name::DevName;
use crate::bfcg::dev_emulators::std_dev::{get_std_win_dev_type, std_win_spec_constructor, StdWinDev};
use crate::bfcg::dev_emulators::std_dev_constructor;
use crate::bfcg::dev_emulators::win::dev_win::DevWin;
use crate::bfcg::disasm::{std_disasm::std_disasm, std_disasm::StdDisasmInfo};
use crate::bfcg::general::{PAD, EXTENSION};
use crate::bfcg::vm::std_processor::{AddDeviceOk, AddDeviceErr, ProcessorRunResult};
use crate::bfcg::vm::{HardwareInfo, StdProcessor, Port};


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] MACRO
macro_rules! ok_transform { 
    ($x:ident) => { if let Ok(x) = $x { x } else { return Err(()) } };
    ($x:expr) => { if let Ok(x) = $x { x } else { return Err(()) } };
}

macro_rules! none_transform { ($x:expr) => { if let Err(_) = $x { return } }; }

// [-] MACRO
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] GEN

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

// [-] GEN
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] COMPILE

fn compile_helper<T, CC>(
    path: &str,
    mem_init_type: MemInitType, 
    checker: Option<HolderChekerMNC>, 
    compiler: CC,
) -> Result<CompilerInfo<T>, CompilerError> 
where 
    CC: comand_compiler::CmdCompiler<T> + PortNameHandler
{
    let mut set_act = SettingActions::new();
    SettingActions::add_std_actions(&mut set_act, mem_init_type);

    let checker = if let Some(x) = checker { x } else { HolderChekerMNC::new() };

    let option = CompilerOption::new(
        mem_init_type,
        compiler,
        &set_act,
        vec![],
        &checker,
    );

    crate::bfcg::compiler::compiler::compile(path.to_owned(), option, None)
}

pub fn std_compile(
    path: &str,
    mem_init_type: MemInitType, 
    checker: Option<HolderChekerMNC>, 
    hardware_info: &HardwareInfo
) -> Result<CompilerInfo<u8>, CompilerError> 
{
    compile_helper(
        path, mem_init_type, checker, 
        comand_compiler::StdCmdCompiler::new(&hardware_info)
    )
}

pub fn interpreter_compile(
    path: &str, 
    mem_init_type: MemInitType, 
    checker: Option<HolderChekerMNC>
) -> Result<CompilerInfo<ValidCMD>, CompilerError> 
{
    compile_helper(
        path, mem_init_type, checker, 
        comand_compiler::InterpreterCmdCompiler::new()
    )
}

// [-] COMPILE
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] PRINT
pub fn device_ctor_warn_std_out(dev_ctor: &DevCtorOk, dev_name: &DevName) {
    if !dev_ctor.warns.is_empty() {
        println!("device <{}> construct WARNING:", dev_name.to_string());
        for x in &dev_ctor.warns { println!("{}{}", PAD, x.to_string()) }
    } 
}
pub fn compiler_error_std_out(err: &CompilerError) { println!("COMPILE ERROR:\n{}", err.to_string()); }
pub fn compiler_warn_std_out(warns: &CompilerWarnings) {
    if warns.is_empty() { return } 
    println!("COMPILE WARNINGS:\n{}", warns.to_string()); 
}


fn device_connecting_print(add_dev_res: Result<AddDeviceOk, AddDeviceErr>, dev_name: &DevName, port_num: usize) -> Result<(), ()> {
    match add_dev_res {
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

// ----- ----- ----- ----- ----- ----- ----- ----- ----- ----- ----- ----- ----- -----
// [=][+] GEN
fn helper_gen_print<T>(c_info: &CompilerInfo<u8>, path: &str, path_folder: &str, name_of_gen: &str, new_ext: &str, gen_f: T) 
where T: Fn(&CompilerInfo<u8>, &str) -> Result<(), ()>
{
    if let Err(_) = std::fs::create_dir_all(path_folder) { 
        println!("ERROR during {} generate: can not create all folders in path: \"{}\"", name_of_gen, path_folder);
    } else {
        let mut path = std::path::Path::new(path).to_path_buf();
        if let Some(ext) = path.extension() {
            if ext != EXTENSION { println!("{} generate:<WEIRD [#1] ERROR>:supposed to not happen", name_of_gen); return } 
        }
        else { path.set_extension(EXTENSION); };
        path.set_extension(new_ext);
        if let Some(file_name) = std::path::Path::new(&path).file_name() { 
            let mut path_file = std::path::Path::new(path_folder).to_path_buf();
            path_file.push(file_name);
            let path_file = path_file.as_path().to_str().unwrap();
            if let Err(_) = (gen_f)(&c_info, &path_file) { 
                println!("ERROR during {} generate: can not create file \"{}\"", name_of_gen, path_file);
            } else { println!("[{} successfully created]", name_of_gen); }
        }
        else {println!("{} generate:<WEIRD [#2] ERROR>:supposed to not happen", name_of_gen)};
    }
}

fn bin_gen_print(c_info: &CompilerInfo<u8>, path: &str, path_bin: &str) {
    helper_gen_print(c_info, path, path_bin, "binary", "bin", gen_binary)
}

fn disasm_std_gen_print(c_info: &CompilerInfo<u8>, path: &str, path_disasm: &str) {
    helper_gen_print(c_info, path, path_disasm, "disasm", "disasm", gen_disasm_std)
}
// [=][-] GEN
// ----- ----- ----- ----- ----- ----- ----- ----- ----- ----- ----- ----- ----- -----

// [-] PRINT
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] RUN VM

fn device_connecting(processor: &mut StdProcessor, dev_ctor: DevCtorOk, dev_name: &DevName, port_num: usize) -> Result<(), ()> {
    let add_dev_res = processor.add_device_boxed(dev_ctor.dev, port_num);
    device_connecting_print(add_dev_res, dev_name, port_num)
}

pub fn vm_run(
    hw_info: &HardwareInfo,
    mut logi_run: LogisticRun<u8>,
) // TODO:MAYBE:(BETTER): return Result<(), Err> THE ERROR: PORT ERROR + DEV ERROR + ... + RUN RES if not OK
{
    let mut processor = StdProcessor::from_hardware_info(hw_info);
    
    let mut win = None;
    let mut win_port = None;
    let mut already_screen = false;
    let mut need_run_win = false;
    let mut win_device_type = vec![];

    let mut com_code_init = false;
    let mut com_port = None;

    let mut cem_code_init = false;
    let mut cem_port = None;

    let mut busy_port = HashSet::new();
    let mut dev_awaiting_for_connection = vec![];

    'dev_for:
    for (port, dev_name) in logi_run.get_devs() {
        // ---------------------------------------
        // [+] PORT:
        let port_num = match port {
            Port::Number(x) => Some(*x),
            Port::Name(x) =>  {
                let port_num = logi_run.get_port(x);

                if Port::more_than_one_uni_port_name(x) {
                    match Port::port_name_uniform(x).as_str() {
                        "com" => { 
                            if com_code_init && com_port.is_none() { println!("WARNING: reconected COM device") }
                            com_code_init = true; 
                            com_port = port_num; 
                        }
                        "cem" => { 
                            if cem_code_init && cem_port.is_none() { println!("WARNING: reconected CEM device") }
                            cem_code_init = true; 
                            cem_port = port_num; 
                        }
                        _ => {}
                    }

                }

                port_num
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
                    dev_awaiting_for_connection.push((dev_ctor, dev_name));
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
                        print_win_for_test();
                        if let Ok(x) = std_win_spec_constructor(dev_name) { win = Some(x) }
                        else { panic!("must never happen") }
                        already_screen = true; 
                    }
                    if port_num.is_some() { busy_port.insert(port_num.unwrap()); }
                    need_run_win = true;
                    win_device_type.push((x, dev_name));
                } else {
                    println!("device <{}> construct ERROR: {}", dev_name.to_string(), err.to_string());
                    return
                }
            }
        }
        // [-] DEV CTOR
        // ---------------------------------------
    }

    
    // ---------------------------------------
    // [+] CONNECTING AWAITED DEVICE:
    let mut cur_port = 3; // need more than 0; 1 and 2 is for com&cem (by default) so let cur_port = 3; 
    for (dev_ctor, dev_name) in dev_awaiting_for_connection {
        while busy_port.contains(&cur_port) { cur_port += 1; }
        none_transform!(device_connecting(&mut processor, dev_ctor, dev_name, cur_port));
        cur_port += 1;
    }

    // COM:
    if !com_code_init && com_port.is_none() {
        let new_com_port = if busy_port.contains(&hw_info.default_com_port) {
            while busy_port.contains(&cur_port) { cur_port += 1; }
            cur_port
        } else { hw_info.default_com_port };

        processor.hardware_change_com_port(new_com_port);
        if new_com_port != hw_info.default_com_port { 
            println!("!: hardware COM port was changed from {} to {}", hw_info.default_com_port, new_com_port);
            cur_port += 1;
        }

        // TODO: possibility to change MEM_SIZE
        let com = Box::new(DevStdCom::new(DEFAULT_COM_MEM_SIZE));    
        let x = processor.add_device_boxed(com, new_com_port);
        none_transform!(device_connecting_print(x, &DevName::new(String::from("com")), new_com_port));
    }

    // CEM:
    if !cem_code_init && cem_port.is_none() {
        let new_cem_port = if busy_port.contains(&hw_info.default_cem_port) {
            while busy_port.contains(&cur_port) { cur_port += 1; }
            cur_port
        } else { hw_info.default_cem_port };

        processor.hardware_change_cem_port(new_cem_port);
        if new_cem_port != hw_info.default_cem_port { 
            println!("!: hardware COM port was changed from {} to {}", hw_info.default_cem_port, new_cem_port);
            cur_port += 1;
        }
        
        // TODO: possibility to change {A|M}M_SIZE
        let cem = Box::new(DevStdCem::new(DEFAULT_CEM_MM_SIZE, DEFAULT_CEM_AM_SIZE));    
        let x = processor.add_device_boxed(cem, new_cem_port);
        none_transform!(device_connecting_print(x, &DevName::new(String::from("cem")), new_cem_port));
    }

    // WIN:
    if need_run_win {
        if win.is_none() {
            println!("ERROR: some of win-related-device is connected but screen is not!");
            return
        }
        assert!(!win_device_type.is_empty());
        assert!(win_device_type.len() == 1, "DELETE this ASSERT when realise more device (curently realised only screen and it can be only one)");

        // TODO: move out here: (it is analog for dev-creator)
        for (x, dev_name) in win_device_type {
            match x {
                StdWinDev::Screen => {
                    let win_dev = Box::new(DevWin::new(win.as_mut().unwrap()));
                    
                    let port = if let Some(port) = win_port { port }
                    else {  
                        while busy_port.contains(&cur_port) { cur_port += 1; }
                        cur_port
                    };

                    let x = processor.add_device_boxed(win_dev, port);
                    none_transform!(device_connecting_print(x, dev_name, port));

                    if win_port.is_none() { cur_port += 1; }
                }
            }
        }
    }

    // [-] CONNECTING AWAITED DEVICE
    // ---------------------------------------

    // ---------------------------------------
    // [+] COM-DEV MEM INIT:
    if let Err(x) = processor.init_memory(logi_run.take_program()) {
        println!("ERROR in COM-DEV memory init: {}", x.to_string());
        return
    }
    // [-] COM-DEV MEM INIT
    // ---------------------------------------

    // ---------------------------------------
    // [+] CREATE RUN LOOP:
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
    // [-] CREATE RUN LOOP
    // ---------------------------------------

    // ---------------------------------------
    // [+] RUN
    if need_run_win {
        let win = win.unwrap();
        win.run(run_f); 
    } else {
        run_f();
    }
    // [-] RUN
    // ---------------------------------------
}

fn print_win_for_test(){
    #[cfg(test)]
    { 
        println!("!!! YOU CANNOT RUN THIS AS TEST(run it from main):");
        println!("for running this code screen is needed but #[test] is not \"main thread\" therefore it cant be run on all platforms."); 
    }
}

// [-] RUN VM
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] (STRUCT): LOGISTIC RUN 
pub struct LogisticRun<T>{
    program: Vec<T>,
    port_names: HashMap<String, usize>,
    devs: HashMap<Port, DevName>,
    program_taked: bool,
}

impl<T> LogisticRun<T>{
    pub fn new(c_info: CompilerInfo<T>) -> Self{
        let (port_names, devs, program) = c_info.decompile();
        Self { port_names, devs, program, program_taked: false }
    }

    pub fn take_program(&mut self) -> Vec<T> {
        if self.program_taked { panic!("[ALGO ERROR]: already taked") }
        self.program_taked = true;
        std::mem::take(&mut self.program)
    }

    pub fn get_devs(&self) -> &HashMap<Port, DevName> { &self.devs }
    pub fn get_port(&self, port_name: &str) -> Option<usize> { if let Some(x) = self.port_names.get(port_name) { Some(*x) } else { None } }
}
// [-] (STRUCT): LOGISTIC RUN 
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub fn main_logistics(x: &LogisticParams) {
    let path = x.get_file(); 

    let hw_info = HardwareInfo::from_logistic_params(x);
    
    let compiled = std_compile(path, MemInitType::BeforeCode, None, &hw_info);

    let c_info = 
    match compiled {
        Err(err) => { compiler_error_std_out(&err); return },
        Ok(x) => x,
    };
    println!("[successfully compiled]");

    if x.need_bin { bin_gen_print(&c_info, path, &x.path_bin); }
    if x.need_disasm { disasm_std_gen_print(&c_info, path, &x.path_disasm); }

    compiler_warn_std_out(&c_info.get_warinings());

    let logi_run = LogisticRun::new(c_info);
    vm_run(&hw_info, logi_run);
}


