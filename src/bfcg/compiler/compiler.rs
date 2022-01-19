use std::str::Chars;

use crate::bfcg::compiler::dif_part_helper::settings::Setting;

use super::comand_compiler::CmdCompiler;
use super::compiler_error::{CompilerError};
use super::compiler_inter_info::CompilerInterInfo;
use super::compiler_option::{CompilerOption, CanCompile};
use super::compiler_pos::CompilerPos;
use super::compiler_info::CompilerInfo;
use super::compiler_warning::CompilerWarning;

// ---------------------------------------------------------------------
// JUST FN:

pub fn file_minimalize(str: &str) -> Result<String, ()>{
    let mut comment_until_next_line = false;
    let mut err = None;

    let ret = str.chars().filter(|c|{
        if err.is_some() { return false }
        match c {
            ';' => { 
                comment_until_next_line = true; 
                false
            }
            x if x.is_ascii_whitespace() => {
                if *x == '\n' { comment_until_next_line = false; }
                false
            }
            x if x.is_ascii_graphic() => {
                !comment_until_next_line
            }
            _ => { 
                if !comment_until_next_line { err = Some(()); } 
                false 
            }
        }
    }).collect();

    if err.is_some() { return Err(err.unwrap()) }

    Ok(ret)
}

// ---------------------------------------------------------------------
// (inner) STRUCT: 

struct InnerCompilerParam<'a>{
    chars: Chars<'a>,
    pos: CompilerPos,
}

impl<'a> InnerCompilerParam<'a>{
    pub fn new(chars: Chars<'a>) -> Self{ 
        Self {
            chars,
            pos: CompilerPos::new(),
        }
    }

    pub fn next(&mut self) -> Option<char> {
        let c = self.chars.next();
        self.pos.maybe_add_char(c);
        c
    }

    pub fn get_pos(&self) -> CompilerPos { self.pos.clone() }
}

// ---------------------------------------------------------------------
// CONSTS: 

pub(super) const MACRO_USE_CHAR: char = '%';
pub(super) const SETTINGS_CHAR: char = '\'';
pub(super) const COMMENT_LINE: char = ';';
pub(super) const NEXT_LINE: char = '\n';
pub(super) const SHARP: char = '#';

type CE = CompilerError;

// ---------------------------------------------------------------------
// USEFUL FNs: 

fn skip_line(param: &mut InnerCompilerParam){
    loop {
        match param.next() {
            Some(super::compiler::NEXT_LINE) | None => break,
            _ => {}
        }
    }
}

fn parse_until_char(param: &mut InnerCompilerParam, first_char: Option<char>, until_char: char) -> Option<String>{
    let mut ret_str = String::new();
    if let Some(x) = first_char { ret_str.push(x); }
    loop {
        match param.next() {
            None => { return None }
            Some(super::compiler::COMMENT_LINE) => skip_line(param),
            Some(c) if c == until_char => { break },
            Some(c) if c.is_whitespace() => { },
            Some(c) => { ret_str.push(c); },
        };
    }
    
    Some(ret_str)
}

// ---------------------------------------------------------------------
// PARSER OF "#...#": 

enum SharpParse{
    UnexpectedEOF,
    EmptyName,
    BadMacroName(char),
    CantCompileMacro,
    CantCompileSetting,
    FileInclude(String, CanCompile),
    Macros{macro_name: String, macro_cmds: String},

    Temp(String),
} 

impl SharpParse{
    fn is_error(&self) -> bool {  
        match self {
            Self::BadMacroName(_) | Self::UnexpectedEOF | Self::EmptyName => true,
            _ => false,
        }
    }

    fn is_temp(&self) -> bool { if let Self::Temp(_) = self { true } else { false } }
    fn temp_to_string(self) -> Option<String> { if let Self::Temp(x) = self { Some(x) } else { None } } 

    fn to_file_include(self, can_compile: CanCompile) -> Self { 
        match self {
            Self::BadMacroName(_) | Self::UnexpectedEOF 
            | Self::CantCompileMacro | Self::CantCompileSetting
            | Self::EmptyName | Self::FileInclude(_, _) => self,
            Self::Temp(s) => { Self::FileInclude(s, can_compile) }
            Self::Macros{ .. } => panic!("macro can't transform into include"), 
        }
    }

    fn parse_until_sharp(param: &mut InnerCompilerParam, first_char: Option<char>) -> SharpParse{
        let ret_str = parse_until_char(param, first_char, super::compiler::SHARP);
        if let None = ret_str { return SharpParse::UnexpectedEOF }

        let ret_str = ret_str.unwrap();
        if ret_str.len() == 0 { SharpParse::EmptyName }
        else { SharpParse::Temp(ret_str) }
    }

    fn parse_sharp(param: &mut InnerCompilerParam, can_compile: CanCompile) -> SharpParse{
        let c;
        loop {
            match param.next() {
                None => { return SharpParse::UnexpectedEOF }
                Some(super::compiler::COMMENT_LINE) => { skip_line(param) },
                Some(super::compiler::MACRO_USE_CHAR) => { return SharpParse::BadMacroName(super::compiler::MACRO_USE_CHAR) },
                Some(c) if c.is_whitespace() => { },
                Some(ok) => { c = Some(ok); break },
            };
        }
        let c = c.unwrap(); // must always ok; but if in loop exist algo error => exception

        if c == super::compiler::SHARP { 
            let to_sharp = Self::parse_until_sharp(param, None);
            if to_sharp.is_error() { return to_sharp }

            let to_sharp = to_sharp.temp_to_string().unwrap();
            if to_sharp == "!" {
                if !can_compile.can_compile_macro() { return SharpParse::CantCompileMacro }
                if !can_compile.can_compile_settings() { return SharpParse::CantCompileSetting }
                return SharpParse::to_file_include(Self::parse_until_sharp(param, None), CanCompile::MacroAndSettings) 
            }
            if to_sharp == "'" { 
                if !can_compile.can_compile_settings() { return SharpParse::CantCompileSetting }
                return SharpParse::to_file_include(Self::parse_until_sharp(param, None), CanCompile::OnlySettings) 
            }
            if !can_compile.can_compile_macro() { return SharpParse::CantCompileMacro }
            return SharpParse::FileInclude(to_sharp, CanCompile::OnlyMacros)
        }
            
        let macro_name = Self::parse_until_sharp(param, Some(c));
        let macro_name =
        if !macro_name.is_temp() { return macro_name } 
        else { macro_name.temp_to_string().unwrap() };
        if macro_name.contains(super::compiler::MACRO_USE_CHAR) { 
            return SharpParse::BadMacroName(super::compiler::MACRO_USE_CHAR) 
        }

        let macro_cmds = Self::parse_until_sharp(param, None);
        let macro_cmds =
        if !macro_cmds.is_temp() { return macro_cmds } 
        else { macro_cmds.temp_to_string().unwrap() };

        SharpParse::Macros{ macro_name, macro_cmds }
    }
}

// ---------------------------------------------------------------------
// MACROS: 

macro_rules! compile_check_error_compile_settings {
    ( $option:ident, $param:ident, $file_name:ident ) => {
        if ! $option.can_compile_settings() {
            return Err(CE::new_cant_compile_settings($param.get_pos(), $file_name)) 
        }
    }
}

macro_rules! compile_prepare_setting {
    ( $option:ident, $param:ident, $file_name:ident, $ret:ident, $setting_string:ident ) => {
        match Setting::prepare_settings(& $setting_string) {
            Err(error) => return Err(CE::new_setting_error($param.get_pos(), $file_name, error)),
            Ok(setting) => {
                let sa_res = $option.setting_action.make_setting_action(&setting, &mut $ret);

                if !sa_res.is_right_rule() {
                    return Err(CE::new_setting_action_error($param.get_pos(), $file_name, sa_res, $setting_string))
                }

                if sa_res.parent_must_process() { $ret.add_setting_for_parent(setting) }

                if let Some(warning) = sa_res.get_warining() {
                    $ret.add_warning(
                        CompilerWarning::SettingWarning{pos: $param.get_pos(), setting: $setting_string,  warning}
                    );
                }
            }
        }
    }
}

macro_rules! compile_one_cmd {
    ( $param:ident, $file_name:ident, $c:ident, $cc:ident ) => {
        if let Some(err) = $cc.cmd_compile($c, $param.get_pos()){
            return Err(CE::new(err, $param.get_pos(), $file_name))
        }
    }
}

macro_rules! compile_seq_cmd {
    ( $param:ident, $file_name:ident, $str_cmds:ident, $cc:ident ) => {
        let seq_cmds = $str_cmds.chars();
        for c in seq_cmds { compile_one_cmd!($param, $file_name, c, $cc) }
    }
}

macro_rules! compile_mem_init_if_need {
    ( $option:ident, $param:ident, $file_name:ident, $ret:ident ) => {
        if let Some(mem_init_code) = $ret.set_code_start($option.mem_init_type) {
            let cc = $option.cmd_compiler.as_mut().unwrap();
            compile_seq_cmd!($param, $file_name, mem_init_code, cc);
        }
    }
}

// ---------------------------------------------------------------------
// COMPILER + PARSER: 

pub fn compile<CC, T>(file_name: String, mut option: CompilerOption<CC, T>, inter_info: Option<CompilerInterInfo>) 
-> Result<CompilerInfo<T>, CompilerError>
where CC: CmdCompiler<T>,
{
    // TODO: file path
    let file_as_string = std::fs::read_to_string(&file_name);
    if file_as_string.is_err() { return Err(CompilerError::new_file_open_err(file_name)) }
    let file_as_string = file_as_string.unwrap();
    let chars = file_as_string.chars();

    let mut ret = CompilerInfo::new(inter_info);
    let mut param = InnerCompilerParam::new(chars);

    if option.need_processed_default_settings() {
        compile_check_error_compile_settings!(option, param, file_name);
        for setting_string in option.get_default_settings() {
            compile_prepare_setting!(option, param, file_name, ret, setting_string);
        } 
    }
    if option.need_processed_default_settings() { panic!("[ALGO ERROR] never must be here") } 


    loop {
        match param.next() {
            None => {
                if option.can_compile_code() {
                    let program = option.cmd_compiler.unwrap().get_program();
                    if let Err(err) = program { return Err(CE::new_wo_pos(err, file_name)) } 
                    else { ret.set_program(program.ok().unwrap()); }
                }
                ret.add_warning_file_name_if_need(file_name);
                return Ok(ret)
            }
            Some(super::compiler::COMMENT_LINE) => { 
                skip_line(&mut param); 
            }

            Some(c) if c.is_whitespace() => { }

            Some(super::compiler::SHARP) => { 
                match SharpParse::parse_sharp(&mut param, option.can_compile) {
                    SharpParse::CantCompileMacro => return Err(CE::new_cant_compile_macros(param.get_pos(), file_name)),
                    SharpParse::CantCompileSetting => return Err(CE::new_cant_compile_settings(param.get_pos(), file_name)),
                    SharpParse::EmptyName => return Err(CE::new_empty_name(param.get_pos(), file_name)),
                    SharpParse::UnexpectedEOF => return Err(CE::new_unexp_eof(param.get_pos(), file_name)),
                    SharpParse::BadMacroName(bad_char) => return Err(CE::new_bad_macro_name(param.get_pos(), file_name, bad_char)),
                    SharpParse::Temp(_) => panic!("here must not be temp"),
                    SharpParse::FileInclude(include, can_compile) => {
                        let include_option = option.new_only(can_compile);
                        let include_inter_info = Some(ret.get_inter_info().clone());
                        let include_compile = compile(include, include_option, include_inter_info);

                        let mut include_compile = if let Err(mut err) = include_compile { 
                            err.add_err_pos(param.get_pos(), file_name);
                            return Err(err)
                        } else {
                            include_compile.ok().unwrap()
                        };

                        // [+] processing merge include
                        if let Some((set_str, error_sar)) = ret.include_file_setting(&mut include_compile, &option, param.get_pos()) {
                            return Err(CE::new_setting_action_error(param.get_pos(), file_name, error_sar, set_str))
                        }
                        if let Some(include_error) = ret.add_include_file(include_compile, param.get_pos()) {
                            return Err(CE::new_include_error(param.get_pos(), file_name, include_error))
                        }
                        // [-] processing merge include
                    }
                    SharpParse::Macros{ macro_name, macro_cmds } => {
                        if !option.can_compile_macro() {
                            return Err(CE::new_cant_compile_macros(param.get_pos(), file_name)) 
                        }

                        if let Some(error) = option.mnc_check(&macro_name, &macro_cmds) {
                            return Err(CE::new(error, param.get_pos(), file_name))
                        }

                        let macro_code = ret.macro_transform(macro_cmds);
                        let macro_code = if let Ok(macro_code) = macro_code { macro_code }
                        else { return Err(CE::new_unknown_macros(param.get_pos(), file_name, macro_code.err().unwrap())) };

                        if !ret.add_macro(macro_name, macro_code) {
                            return Err(CE::new_already_defined(param.get_pos(), file_name))
                        }
                    }
                }
            }

            Some(super::compiler::SETTINGS_CHAR) => {
                compile_check_error_compile_settings!(option, param, file_name);

                let setting = parse_until_char(&mut param, None, super::compiler::SETTINGS_CHAR);
                let setting_string = 
                    if let Some(setting) = setting { setting } 
                    else { return Err(CE::new_unexp_eof(param.get_pos(), file_name)) };
                
                compile_prepare_setting!(option, param, file_name, ret, setting_string);
            }

            Some(_) if !option.can_compile_code() => {
                return Err(CE::new_cant_compile_code(param.get_pos(), file_name)) 
            }

            Some(super::compiler::MACRO_USE_CHAR) => {
                // say that code started (even if used macro was empty (cause it is potential error))
                // & compile mem init if need
                compile_mem_init_if_need!(option, param, file_name, ret);
                
                let macros_name = parse_until_char(&mut param, None, super::compiler::MACRO_USE_CHAR);
                if let Some(macros_name) = macros_name {
                    let macros_code = ret.get_macros_code(&macros_name);
                    if macros_code.is_none() { 
                        return Err(CE::new_unknown_macros(param.get_pos(), file_name, macros_name)) 
                    } else {
                        let macros_code = macros_code.unwrap();
                        let cc = option.cmd_compiler.as_mut().unwrap();
                        compile_seq_cmd!(param, file_name, macros_code, cc);
                    }
                } else {
                    return Err(CE::new_unexp_eof(param.get_pos(), file_name))
                }
            }

            Some(c) => {
                // say that code started & compile mem init if need
                compile_mem_init_if_need!(option, param, file_name, ret);

                let cc = option.cmd_compiler.as_mut().unwrap();
                compile_one_cmd!(param, file_name, c, cc);
            }
        }
    }
}
