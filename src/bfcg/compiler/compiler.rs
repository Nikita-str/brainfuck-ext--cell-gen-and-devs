use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::Chars;

use super::cmd_compiler::CmdCompiler;
use super::compiler_error::{CompilerError, CompilerErrorType};
use super::compiler_option::CompilerOption;
use super::compiler_pos::CompilerPos;

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

pub struct CompilerInfo<T>{
    mem_init: Option<Vec<u8>>,
    program: Vec<T>,
    devs: HashMap<usize, String>, // map port to dev name
    macros: HashMap<String, String>, // map name of macros to code
}

impl<T> CompilerInfo<T>{
    pub fn new() -> Self { 
        Self {
            mem_init: None,
            program: vec![],
            devs: HashMap::new(),
            macros: HashMap::new(),
        }
    }

    pub fn add_macro(&mut self, macro_name: String, macro_cmds: String) -> bool {
        if self.macros.contains_key(&macro_name) { false }
        else {  
            self.macros.insert(macro_name, macro_cmds);
            true
        }
    }

    pub fn add_include_file(&mut self, include: Self) -> bool {
        for (macro_name, macro_cmds) in include.macros {
            if !self.add_macro(macro_name, macro_cmds) { return false }
        }
        true
    }
}

pub(super) const MACRO_USE_CHAR: char = '%';
pub(super) const SETTINGS_CHAR: char = '\'';
pub(super) const COMMENT_LINE: char = ';';
pub(super) const NEXT_LINE: char = '\n';
pub(super) const SHARP: char = '#';

fn skip_line(param: &mut InnerCompilerParam){
    loop {
        match param.next() {
            Some(super::compiler::NEXT_LINE) | None => break,
            _ => {}
        }
    }
}

enum SharpParse{
    UnexpectedEOF,
    EmptyName,
    FileInclude(String),
    Macros{macro_name: String, macro_cmds: String},

    Temp(String),
} 

impl SharpParse{
    fn is_temp(&self) -> bool { if let Self::Temp(_) = self { true } else { false } }
    fn temp_to_string(self) -> Option<String> { if let Self::Temp(x) = self { Some(x) } else { None } } 

    fn to_file_include(self) -> Self { 
        match self {
            Self::UnexpectedEOF | Self::EmptyName | Self::FileInclude(_) => self,
            Self::Temp(s) => Self::FileInclude(s),
            Self::Macros{ .. } => panic!("macro can't transform into include"), 
        }
    }
}

fn parse_until_sharp(param: &mut InnerCompilerParam) -> SharpParse{
    let mut ret_str = String::new();
    loop {
        let c = match param.next() {
            None => { return SharpParse::UnexpectedEOF }
            Some(super::compiler::COMMENT_LINE) => skip_line(param),
            Some(super::compiler::SHARP) => { break },
            Some(c) if c.is_whitespace() => { },
            Some(c) => { ret_str.push(c); },
        };        
    }

    if ret_str.len() == 0 { SharpParse::EmptyName }
    else { SharpParse::Temp(ret_str) }
}

fn parse_sharp(param: &mut InnerCompilerParam) -> SharpParse{
    let mut c = None;
    loop {
        match param.next() {
            None => { return SharpParse::UnexpectedEOF }
            Some(super::compiler::COMMENT_LINE) => { skip_line(param) },
            Some(c) if c.is_whitespace() => { },
            Some(ok) => { c = Some(ok); break },
        };
    }
    let c = c.unwrap(); // must always ok; but if in loop exist algo error => exception

    if c == super::compiler::SHARP { return SharpParse::to_file_include(parse_until_sharp(param)) }
        
    let macro_name =  parse_until_sharp(param);
    let macro_name =
    if !macro_name.is_temp() { return macro_name } 
    else { macro_name.temp_to_string().unwrap() };

    let macro_cmds =  parse_until_sharp(param);
    let macro_cmds =
    if !macro_cmds.is_temp() { return macro_cmds } 
    else { macro_cmds.temp_to_string().unwrap() };

    SharpParse::Macros{ macro_name, macro_cmds }
}

type CE = CompilerError;

// TODO: type of error insted of (); if return None => no errors
pub fn compile<CC, T>(file_name: String, mut option: CompilerOption<CC, T>) -> Result<CompilerInfo<T>, CompilerError>
where CC: CmdCompiler<T>,
{
    let file_as_string = std::fs::read_to_string(&file_name);
    if file_as_string.is_err() { return Err(CompilerError::new_file_open_err(file_name)) }
    let file_as_string = file_as_string.unwrap();
    let chars = file_as_string.chars();

    let mut ret = CompilerInfo::new();
    let mut param = InnerCompilerParam::new(chars);

    loop {
        match param.next() {
            None => {
                //TODO !!!!

                break
            }
            Some(super::compiler::COMMENT_LINE) => { 
                skip_line(&mut param); 
            }

            Some(c) if c.is_whitespace() => { }

            Some(super::compiler::SHARP) => { 
                match parse_sharp(&mut param) {
                    SharpParse::EmptyName => return Err(CE::new_empty_name(param.get_pos(), file_name)),
                    SharpParse::UnexpectedEOF => return Err(CE::new_unexp_eof(param.get_pos(), file_name)),
                    SharpParse::Temp(_) => panic!("here must not be temp"),
                    SharpParse::FileInclude(include) => {
                        let include_compile = compile(include, CompilerOption::<CC, T>::new_only_macro());
                        let include_compile = if let Err(mut err) = include_compile { 
                            err.add_err_pos(param.get_pos(), file_name);
                            return Err(err)
                        } else {
                            include_compile.ok().unwrap()
                        };
                        if !ret.add_include_file(include_compile) {
                            return Err(CE::new_already_defined(param.get_pos(), file_name))
                        }
                    }
                    SharpParse::Macros{ macro_name, macro_cmds } => {
                        if !ret.add_macro(macro_name, macro_cmds) {
                            return Err(CE::new_already_defined(param.get_pos(), file_name))
                        }
                    }
                }
            }

            Some(_) if option.only_macros => {
                return Err(CE::new_code_in_macros(param.get_pos(), file_name)) 
            }

            Some(super::compiler::SETTINGS_CHAR) => {
                todo!("todo")
            }

            Some(super::compiler::MACRO_USE_CHAR) => {
                todo!("todo")
            }

            Some(c) => {
                let cc = &mut option.cmd_compiler;
                if let Some(mut err) = cc.as_mut().unwrap().cmd_compile(c, param.get_pos()){
                    err.add_err_pos(param.get_pos(), file_name);
                    return Err(err)
                }
            }
        }
    }

    Ok(ret)
}