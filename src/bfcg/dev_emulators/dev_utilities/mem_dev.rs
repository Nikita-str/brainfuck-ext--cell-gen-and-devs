use crate::bfcg::compiler::valid_cmd::ValidCMD;


#[repr(u8)]
pub enum CellMemDevStartAction {
    GetCellValue = 0x01, 
    SetCellValue = 0x02, 
    
    PrevCell = 0x03, 
    NextCell = 0x04,
    
    CreateCell = 0x05,
    DeleteCell = 0x06,
}

impl CellMemDevStartAction{
    pub fn from_valid_cmd(cmd: &ValidCMD) -> Self{
        match cmd {
            ValidCMD::NextCell => Self::NextCell,
            ValidCMD::PrevCell => Self::PrevCell,
            ValidCMD::CreateCell => Self::CreateCell,
            ValidCMD::DeleteCell => Self::DeleteCell,
            _ => panic!("not cell-mem-dev cmd"),
        }
    }
}


#[repr(u8)]
pub enum CmdMemDevAction {
    StartJumpBackward = 0x01, 
    StartJumpForward = 0x02, 

    BeforeEnd = 0x00, // help for clear distinguish that SE ended

    EndJumpBackward = 0x03, 
    EndJumpForward = 0x04, 
}
