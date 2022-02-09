use bf_cell_gen::{logistics, LogisticParams};
use clap::StructOpt;


fn main() {
    let x = LogisticParams::parse();
    logistics::main_logistics(&x);
    return;
}
