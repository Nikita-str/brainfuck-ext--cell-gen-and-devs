use crate::bfcg::compiler::valid_cmd::ValidCMD;


#[repr(u8)]
#[derive(Debug)]
pub enum WinDevStartAction {
    DecCoordX = 4, // num pad ←
    DecCoordY = 8, // num pad ↑
    IncCoordX = 6, // num pad →
    IncCoordY = 2, // num pad ↓
    
    RedrawWin = 13, 

    GetWinValue = 14, 
    SetWinValue = 15, 
}

impl WinDevStartAction{
    pub fn from_valid_cmd(cmd: &ValidCMD) -> Self{
        match cmd {
            ValidCMD::DecCoordX => Self::DecCoordX,
            ValidCMD::DecCoordY => Self::DecCoordY,
            ValidCMD::IncCoordX => Self::IncCoordX,
            ValidCMD::IncCoordY => Self::IncCoordY,

            ValidCMD::RedrawWin => Self::RedrawWin,
            
            ValidCMD::GetWinValue => Self::GetWinValue,
            ValidCMD::SetWinValue => Self::SetWinValue,
            _ => panic!("not cell-mem-dev cmd"),
        }
    }

    pub fn need_additional_input(&self) -> bool {
        match self {
            Self::DecCoordX | Self::DecCoordY | Self::IncCoordX | Self::IncCoordY | Self::RedrawWin => false,
            Self::GetWinValue => false,
            Self::SetWinValue => true,  
        }
    }
}