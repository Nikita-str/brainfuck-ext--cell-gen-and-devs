
pub struct HardwareInfo{
    pub max_port_amount: usize, 
    pub max_jump_size: usize, 
    pub default_cem_port: usize, 
    pub default_com_port: usize,
}

impl HardwareInfo {
    pub fn from_logistic_params(params: &crate::LogisticParams) -> Self {
        HardwareInfo {
            max_port_amount: params.hardware_port_amount,
            max_jump_size: params.hardware_max_jump_size,
            default_cem_port: params.hardware_cem_port,
            default_com_port: params.hardware_com_port,
        }
    }
}