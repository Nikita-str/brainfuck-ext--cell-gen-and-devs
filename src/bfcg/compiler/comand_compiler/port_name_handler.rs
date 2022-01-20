use std::collections::HashMap;
use crate::bfcg::compiler::compiler_error::CompilerErrorType;

pub trait PortNameHandler{
    fn need_port_name_handle(&self) -> bool;
    fn port_name_handle(&mut self, port_names: &HashMap<String, usize>) -> Option<CompilerErrorType>;
}