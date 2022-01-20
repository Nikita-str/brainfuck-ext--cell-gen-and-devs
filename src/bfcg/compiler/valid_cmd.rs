
#[derive(Clone)]
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
    Bnd, // cur_cell_value = &0x80;
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

    /*
    # in phf no reverse fn => use it makes no sense 
    # so reverse by python: 
    ss = ""
    for line in s.split('\n'):
        x = line.split('=>')
        if len(x) == 1: ss = ss + line + '\n'; continue
        (l,r) = x
        z = l.split("'")
        ss = ss + z[0] + r.strip().removeprefix('Some(').removesuffix('),') + ' => ' + "Some('" + z[1] + "'),\n"
    print(ss)

    # (maybe in rust we can make macros for it?! (for gen such pair fn))
    */

    pub fn std_to_char(self) -> char { Self::std_cmd_to_char(self) }

    pub const fn std_cmd_to_char(cmd: Self) -> char{
        match cmd {
            Self::NextCell => '>',
            Self::PrevCell => '<',
            Self::IncValue => '+',
            Self::DecValue => '-',
            Self::PrintValue => '.',
            Self::ReadValue => ',',
            Self::StartWhileNZ => '[',
            Self::EndWhileNZ => ']',

            Self::CreateCell => 'c',
            Self::DeleteCell => 'd',

            Self::TestZeroCell => 'z',
            Self::ZeroedCell => '0',
            Self::LeftShift => '*',
            Self::RightShift => '/',
            Self::And => '&',
            Self::Bnd => '^',
            Self::Clone => '$',

            // cause NumPad:
            Self::DecCoordX => '4',
            Self::DecCoordY => '8',
            Self::IncCoordX => '6',
            Self::IncCoordY => '2',

            Self::SetWinValue => '5',
            Self::RedrawWin => '@',

            Self::SetCurPort => 's',
            Self::ReadFromPort => 'r',
            Self::WriteIntoPort => 'w',
            Self::TestPort => 't',
        }
    }

    pub const fn std_parse_char(char: char) -> Option<Self>{
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
            '^' => Some(Self::Bnd),
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

    pub fn is_start_while(&self) -> bool { if let Self::StartWhileNZ = self { true } else { false } }
    pub fn is_end_while(&self) -> bool { if let Self::EndWhileNZ = self { true } else { false } }
}

