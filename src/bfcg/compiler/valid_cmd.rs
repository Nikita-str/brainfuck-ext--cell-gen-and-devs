
pub enum ValidCMD{
    NextCell,
    PrevCell,

    IncValue,
    DecValue,

    PrintValue,
    ReadValue,

    StartWhileNZ,
    EndWhileNZ,

    // New CMD:

    // Create & Delete:
    CreateCell,
    DeleteCell,

    // cur_cell_value = if cur_cell_value == 0 { 0 } else { 1 };
    TestZeroCell,

    // for non-exponitial (from bit num len) algo for sum mul div ...:
    ZeroedCell, // cur_cell_value = 0;
    LeftShift, // (~mul 2) cur_cell_value = cur_cell_value << 1;
    RightShift,// (~int div 2) cur_cell_value = cur_cell_value >> 1;
    And, // (~int remainder 2) cur_cell_value = &1;
    Clone, // clone value of cur cell into next 

    // screen cmd:
    DecCoordX, // x-- (with win size overflow)
    DecCoordY, // y-- (with win size overflow)
    IncCoordX, // x++ (with win size overflow)
    IncCoordY, // y++ (with win size overflow)
    SetWinValue, // move cur_cell_value into cur window cell
    RedrawWin, // screen redraw

    // port cmd:
    SetCurPort, // cur_port = cur_cell_value
    ReadFromPort, // read value from port into cur_cell 
    WriteIntoPort, // write cur_cell_value into port
    TestPort, // if you can get value from port then cur_cel_value = 1; else 0;
}

impl ValidCMD{
    pub fn std_parse_char(char: char) -> Option<Self>{
        match char {
            '>' => Some(Self::NextCell),
            '<' => Some(Self::PrevCell),
            '+' => Some(Self::IncValue),
            '-' => Some(Self::DecValue),
            '.' => Some(Self::PrintValue),
            ',' => Some(Self::ReadValue),
            '[' => Some(Self::StartWhileNZ),
            ']' => Some(Self::EndWhileNZ),

            'c' => Some(Self::CreateCell),
            'd' => Some(Self::DeleteCell),

            'z' => Some(Self::TestZeroCell),
            '0' => Some(Self::ZeroedCell),
            '*' => Some(Self::LeftShift),
            '/' => Some(Self::RightShift),
            '&' => Some(Self::And),
            '$' => Some(Self::Clone),

            // cause NumPad:
            '4' => Some(Self::DecCoordX), 
            '8' => Some(Self::DecCoordY),
            '6' => Some(Self::IncCoordX),
            '2' => Some(Self::IncCoordY),

            '5' => Some(Self::SetWinValue),
            '@' => Some(Self::RedrawWin),

            's' => Some(Self::SetCurPort),
            'r' => Some(Self::ReadFromPort),
            'w' => Some(Self::WriteIntoPort),
            't' => Some(Self::TestPort),
            
            _ => None,
        }
    }
}

