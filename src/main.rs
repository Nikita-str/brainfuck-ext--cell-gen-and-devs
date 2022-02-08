
use bf_cell_gen::{logistics, LogisticParams};
use clap::StructOpt;

// 
// use bf_cell_gen::example_run::compiler_test_u8_exmp_02;


fn main() {
    let x = LogisticParams::parse();
    // let path = "examples/examples/03_win_print_alphabet.bf-ext";
    logistics::main_logistics(&x);
    return;
}
