
#[repr(u8)]
pub enum CellMemDevStartAction {
    GetCellValue = 0x01, 
    SetCellValue = 0x02, 
    
    PrevCell = 0x03, 
    NextCell = 0x04,
    
    CreateCell = 0x05,
    DeleteCell = 0x06,
}


#[repr(u8)]
pub enum CmdMemDevStartAction {
    JumpLeft = 0x01, 
    JumpRight = 0x02, 
}
