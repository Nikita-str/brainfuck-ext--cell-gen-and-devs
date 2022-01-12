use crate::bfcg::{compiler::compiler_info::CompilerInfo, vm::port::Port, dev_emulators::dev::right_std_dev_name};

use super::{settings::Setting, setting_action_result::SettingActionResult};

type SAR = SettingActionResult;

pub struct SettingActions<T>{
    actions: Vec<Box<dyn Fn(&Setting, &mut CompilerInfo<T>) -> SettingActionResult>>,
}

impl<T> SettingActions<T> {
    pub fn new() -> Self { Self{ actions: vec![] } } 

    pub fn add_action(&mut self, action: impl Fn(&Setting, &mut CompilerInfo<T>) -> SettingActionResult + 'static) {
        self.actions.push(Box::new(action))
    }

    pub fn make_setting_action(&self, setting: &Setting, c_info: &mut CompilerInfo<T>) -> SettingActionResult {
        let mut last_error = None;
        for act in &self.actions {
            let suitable_action = (act.as_ref())(setting, c_info);
            if suitable_action.is_right_rule() { return suitable_action }
            if suitable_action.is_error() { last_error = Some(suitable_action) }
        }    

        if let Some(error) = last_error { error }
        else { SAR::new_no() }
    }

    /// add rule that connect dev by 'dev[port]:dev_name'
    /// ##### not check that dev_name is valid
    #[allow(non_snake_case)]
    fn std_action__add_dev(actions: &mut Self) {
        actions.add_action(|setting, c_info|{
            if setting.params.len() < 1 { return SAR::new_no() }
            if setting.params[0].param != "dev" { return SAR::new_no() }
            if setting.params.len() == 1 { return SAR::new_error("after dev must stay dev name.\ngeneral form: 'dev[dev_port]:dev-name'") }
            if setting.params.len() > 2 { return SAR::new_error("too many parameters for dev setting") }
            if setting.params[1].param.len() == 0 { return SAR::new_error("empty device name")  }
            if setting.params[0].additional_params.len() > 1 { return SAR::new_error("too many aditional params for dev (was 'dev[x|y]' must 'dev[x]')") }
            if setting.params[1].additional_params.len() > 0 { return SAR::new_error("too many aditional params for dev-name (was 'dev[...]:x[y]' must 'dev[...]:x')") }
            
            let dev_param = &setting.params[0].additional_params;
            let dev_name = &setting.params[1].param;
            if !right_std_dev_name(dev_name) { return SAR::new_error_s("bad device name: ".to_owned() + dev_name) }

            let is_dev_num = dev_param[0].len() > 0 && dev_param[0].chars().all(|c|c.is_ascii_digit()); 

            let port = 
                if dev_param.len() == 0 { Port::new_any() }
                else if Port::right_port_name(&dev_param[0]) || is_dev_num { Port::new(&dev_param[0]) } 
                else { return SAR::new_error_s("wrong port name: ".to_owned() + &dev_param[0]) };
            
            if let Some(prev_name) = c_info.add_dev(port, dev_name.to_owned()) {
                return SAR::new_warning_s("device [".to_owned() + dev_name + "] stay instead device [" + &prev_name + "]", false )
            } 
            return SAR::new_ok(false)
        });
    }

    fn std_action_only_name(actions: &mut Self, the_name: Vec<String>, act: impl Fn(&mut CompilerInfo<T>) + 'static, need_in_parent: bool){
        actions.add_action(move |setting, c_info|{
            if setting.params.len() < 1 { return SAR::new_no() }
            if !the_name.contains(&setting.params[0].param) { return SAR::new_no() }

            act(c_info);

            if setting.params[0].additional_params.len() != 0 
               || setting.params.len() != 1  { return SAR::new_warning("waste params", need_in_parent) }
            else { return SAR::new_ok(need_in_parent) }
        });        
    }

    #[allow(non_snake_case)]
    fn std_action__dis_all_dev(actions: &mut Self) {
        Self::std_action_only_name(
            actions, 
            vec!["dis-all-dev".to_owned()], 
            |c_info|{ c_info.get_mut_devs().clear() },
            true,
        );
    }

    #[allow(non_snake_case)]
    fn std_action__del_all_port_name(actions: &mut Self) {
        Self::std_action_only_name(
            actions, 
            vec!["del-all-port-name".to_owned(), "del-all-pn".to_owned()], 
            |c_info|{ c_info.clear_port_names() },
            true,
        );
    }
    
    #[allow(non_snake_case)]
    fn std_action__param_pos_set_jw(actions: &mut Self) {
        actions.add_action(|setting, c_info|{
            if setting.params.len() < 1 { return SAR::new_no() }
            if setting.params[0].param != "param-pos--just+win" 
               || setting.params[0].param != "param-pos-jw" 
            {
                if setting.params[0].param != "param-pos-just+win" { return SAR::new_error("maybe you want to write 'param-pos--just+win' (twiced \"-\")") }
                return SAR::new_no() 
            }

            if setting.params.len() != 3 { return SAR::new_error("here must be name and two uszie params") }
            for _ in 0..3 {
                if setting.params[0].additional_params.len() != 0  { 
                    return SAR::new_error("excess additional params!") 
                }
            };

            let j_param = setting.params[1].param.parse();
            let w_param = setting.params[2].param.parse();

            if j_param.is_err() || w_param.is_err() { return SAR::new_error("param must be usize; for example: 'param-pos--just+win : 1 : 0'") }

            let (j_param, w_param) = (j_param.unwrap(), w_param.unwrap());
            if j_param == w_param { return SAR::new_error("param pos number must be different!") }

            if !c_info.get_mem_init().is_empty() { return SAR::new_error("mem already initialized, change param pos can only be done before it!") } 

            if !c_info.get_mut_mem_init().set_param_pos(j_param, w_param) {
                panic!("it must never occured!")
            }

            return SAR::new_ok(true) 
        });     
    }

    #[allow(non_snake_case)]
    fn std_action__port_name(actions: &mut Self) {
        actions.add_action(|setting, c_info|{
            if setting.params.len() < 1 { return SAR::new_no() }
            if setting.params[0].param != "port-name" { return SAR::new_no() }
            
            if setting.params.len() != 3 { return SAR::new_error("here must be name and 2 params") }
            for _ in 0..3 {
                if setting.params[0].additional_params.len() != 0  { 
                    return SAR::new_error("excess additional params!") 
                }
            };

            let port_name = &setting.params[1].param;
            if !Port::right_port_name(port_name) { return SAR::new_error_s("wrong port name: ".to_owned() + port_name) }
            let port_name = Port::new(port_name).to_name();

            let port = setting.params[2].param.parse();
            if port.is_err() { return SAR::new_error_s("wrong port number: ".to_owned() + &setting.params[2].param) }
            let port = port.unwrap();

            if let Some(prev_port) = c_info.add_port(port_name, port) { 
                return SAR::new_warning_s("! changed port position from ".to_owned() + &port.to_string() + " to " + &prev_port.to_string(), false) 
            }

            return SAR::new_ok(false)
        });
    }

    #[allow(non_snake_case)]
    fn std_action__add_param(actions: &mut Self) {
        actions.add_action(|setting, c_info|{
            if setting.params.len() < 1 { return SAR::new_no() }
            if setting.params[0].param != "+param" { return SAR::new_no() }
            
            if !c_info.get_mem_init().can_add_just_param() { return SAR::new_error("cant add param (seems like position of just-param is not seted)") } 

            if setting.params.len() != 2 { return SAR::new_error("here must be name and 1 param") }
            for _ in 0..1 {
                if setting.params[0].additional_params.len() != 0  { 
                    return SAR::new_error("excess additional params!") 
                }
            };

            let num = setting.params[1].param.parse();
            if num.is_err() { return SAR::new_error_s("wrong number: ".to_owned() + &setting.params[1].param) }
            let num: usize = num.unwrap();

            if num > u8::MAX.into() { return SAR::new_error_s("too big number(must be in [0; 2^8) ): ".to_owned() + &num.to_string()) }

            if !c_info.get_mut_mem_init().add_just_param(num as u8) {
                panic!("must never here")
            }

            return SAR::new_ok(false) 
        });
    }

    #[allow(non_snake_case)]
    fn std_action__win_sz(actions: &mut Self) {
        actions.add_action(|setting, c_info|{
            if setting.params.len() < 1 { return SAR::new_no() }
            if setting.params[0].param != "win-sz" { return SAR::new_no() }
            
            if !c_info.get_mem_init().can_add_win_param() { return SAR::new_error("cant add param (seems like position of just-param is not seted)") } 

            if setting.params.len() != 1 { return SAR::new_error("here must be name and 1 param") }
            if setting.params[0].additional_params.len() != 2  { return SAR::new_error("after 'win-sz' must stay 2 additional params") }

            let x_sz = setting.params[0].additional_params[0].parse();
            let y_sz = setting.params[0].additional_params[1].parse();
            if x_sz.is_err() { return SAR::new_error("wrong win x szie") }
            if y_sz.is_err() { return SAR::new_error("wrong win y szie") }
            let mut x_sz: usize = x_sz.unwrap();
            let mut y_sz: usize = y_sz.unwrap();

            // TODO: set x y sz in c_info!

            let max_sz: usize = u8::MAX.into();
            let max_sz = max_sz + 1;
            let max_sz = max_sz * max_sz * max_sz - 1; 

            if x_sz > max_sz { return SAR::new_error("too big win x szie") }
            if y_sz > max_sz { return SAR::new_error("too big win y szie") }

            let already_in_win = c_info.get_mem_init().len_in_win_mmc() > 0;

            let mut win_mmc = vec![]; // LE x_sz:u24; y_sz:u24;

            while x_sz > 0 || win_mmc.len() < 3 {
                win_mmc.push((x_sz % (u8::MAX as usize + 1)) as u8);
                x_sz = x_sz / (u8::MAX as usize + 1);
            }
            while y_sz > 0 || win_mmc.len() < 6 {
                win_mmc.push((y_sz % (u8::MAX as usize + 1)) as u8);
                y_sz = y_sz / (u8::MAX as usize + 1);
            }

            let win_pos = c_info.get_mem_init().get_win_pos();
            c_info.get_mut_mem_init().add_bytes(win_pos, &win_mmc);

            if already_in_win { return SAR::new_warning("in windows cell already exist params!", false) }
            return SAR::new_ok(false)
        });
    }


    pub fn add_std_actions(actions: &mut Self) {
        Self::std_action__win_sz(actions);
        Self::std_action__add_dev(actions);
        Self::std_action__add_param(actions);
        Self::std_action__port_name(actions);
        Self::std_action__dis_all_dev(actions);
        Self::std_action__param_pos_set_jw(actions);
        Self::std_action__del_all_port_name(actions);
    }
}
