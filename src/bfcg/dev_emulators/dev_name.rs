use std::collections::HashMap;



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevName{
    name: String,
    params: HashMap<String, String>,
}

impl DevName{
    pub fn new(dev_name: String) -> Self {
        Self {
            name: dev_name,
            params: HashMap::new(),
        }
    }

    pub fn add_param(&mut self, param_name: String, param_value: String) -> Option<String>{
        self.params.insert(param_name, param_value)
    }
}

impl ToString for DevName {
    fn to_string(&self) -> String {
        let mut ret = self.name.clone();
        if self.params.len() > 0 {
            ret.push('<');
            for (param, value) in &self.params {
                ret += " |";
                ret += param;
                ret += " = ";
                ret += value; 
                ret += "| ";
            }
            ret.push('>');
        }
        ret
    }
}