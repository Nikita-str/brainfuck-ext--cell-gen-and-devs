use crate::bfcg::compiler::valid_cmd::ValidCMD;


#[repr(u8)]
pub enum WinDevStartAction {
    DecCoordX = 4, // num pad ←
    DecCoordY = 8, // num pad ↑
    IncCoordX = 6, // num pad →
    IncCoordY = 2, // num pad ↓
    
    RedrawWin = 13, 

    SetWinValue = 42, 
}

impl WinDevStartAction{
    pub fn from_valid_cmd(cmd: &ValidCMD) -> Self{
        match cmd {
            ValidCMD::DecCoordX => Self::DecCoordX,
            ValidCMD::DecCoordY => Self::DecCoordY,
            ValidCMD::IncCoordX => Self::IncCoordX,
            ValidCMD::IncCoordY => Self::IncCoordY,

            ValidCMD::RedrawWin => Self::RedrawWin,
            
            ValidCMD::SetWinValue => Self::SetWinValue,
            _ => panic!("not cell-mem-dev cmd"),
        }
    }

    pub fn need_additional_input(&self) -> bool {
        match self {
            Self::DecCoordX | Self::DecCoordY | Self::IncCoordX | Self::IncCoordY | Self::RedrawWin => false,
            Self::SetWinValue => true,  
        }
    }
}