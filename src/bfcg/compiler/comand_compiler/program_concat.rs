use super::CmdCompiler;


pub trait ProgramConcat<T> 
where Self: CmdCompiler<T>
{
    /// concat two compiled programs 
    /// 
    /// it is generaly `default_program_concat` (p1 append p2)
    /// 
    /// but if your compiler have dirrect jump (by addr, not shift) then you cant use `default_program_concat`
    fn program_concat(p1: Vec<T>, p2: Vec<T>) -> Vec<T>;
}

pub(super) fn default_program_concat<T>(mut p1: Vec<T>, mut p2: Vec<T>) -> Vec<T>{
    p1.append(&mut p2);
    p1
}