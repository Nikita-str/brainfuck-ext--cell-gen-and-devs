use std::marker::PhantomData;

use super::cmd_compiler::CmdCompiler;


pub struct CompilerOption<CC, T>
where CC: CmdCompiler<T>,
{
    pub phantom: PhantomData<T>,
    pub only_macros: bool,
    pub can_dir_mem_init: bool,
    pub cmd_compiler: Option<CC>,
}

impl<CC, T> CompilerOption<CC, T>
where CC: CmdCompiler<T>
{
    pub fn new_only_macro() -> Self{
        Self {
            phantom: PhantomData,
            only_macros: true,
            can_dir_mem_init: false,
            cmd_compiler: None,
        }
    }
}