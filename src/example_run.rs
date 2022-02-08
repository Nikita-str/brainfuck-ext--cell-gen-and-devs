// screen test can't be run by #[test]
// cause in #[test] used not main thread 
// so here just some win-run 

use crate::logistics;

/// call it from main
pub fn compiler_test_u8_exmp_02() {
    let x = logistics::LogisticParams::new_empty_example(2);
    logistics::main_logistics(&x); 
}


/// call it from main
pub fn compiler_test_u8_exmp_03() {
    let x = logistics::LogisticParams::new_empty_example(3);
    logistics::main_logistics(&x); 
}