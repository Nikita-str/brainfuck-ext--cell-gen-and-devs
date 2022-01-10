use crate::bfcg::{compiler::compiler::CompilerInfo, vm::port::Port, dev_emulators::dev::right_std_dev_name};

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
        for act in &self.actions {
            let suitable_action = (act.as_ref())(setting, c_info);
            if suitable_action.is_right_rule() { return suitable_action }
        }    

        return SettingActionResult::NoSatisfy
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

            let port = 
                if dev_param.len() == 0 { Port::new_any() }
                else if Port::right_port_name(&dev_param[0]) { Port::new(&dev_param[0]) }
                else { return SAR::new_error_s("wrong port name: ".to_owned() + &dev_param[0]) };
            
            if let Some(prev_name) = c_info.add_dev(port, dev_name.to_owned()) {
                return SAR::new_warning_s("device [".to_owned() + dev_name + "] stay instead device [" + &prev_name + "]" )
            } 
            return SAR::Ok
        });
    }

    fn std_action_only_name(actions: &mut Self, the_name: Vec<String>, act: impl Fn(&mut CompilerInfo<T>) + 'static){
        actions.add_action(move |setting, c_info|{
            if setting.params.len() < 1 { return SAR::new_no() }
            if !the_name.contains(&setting.params[0].param) { return SAR::new_no() }

            act(c_info);

            if setting.params[0].additional_params.len() != 0 
               || setting.params.len() != 1  { return SAR::new_warning("waste params") }
            else { return SAR::Ok }
        });        
    }

    #[allow(non_snake_case)]
    fn std_action__dis_all_dev(actions: &mut Self) {
        Self::std_action_only_name(
            actions, 
            vec!["dis-all-dev".to_owned()], 
            |c_info|{ c_info.get_mut_devs().clear() }
        );
    }

    #[allow(non_snake_case)]
    fn std_action__del_all_port_name(actions: &mut Self) {
        Self::std_action_only_name(
            actions, 
            vec!["del-all-port-name".to_owned(), "del-all-pn".to_owned()], 
            |c_info|{ c_info.clear_port_names() }
        );
    }

    pub fn add_std_actions(actions: &mut Self) {
        Self::std_action__add_dev(actions);
        Self::std_action__dis_all_dev(actions);
        Self::std_action__del_all_port_name(actions);
    }
}
