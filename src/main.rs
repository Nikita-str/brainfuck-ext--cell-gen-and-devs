
fn main() {
    println!(
        "{}",
        bf_cell_gen::bfcg::compiler::compiler
            ::file_minimalize("++-[+ + +]0-\n42;24;1  5\nno ne++;;;;;#xx#z;;\n   #yy# w").ok().unwrap()
    );
    bf_cell_gen::bfcg::dev_emulators::win::Win::new(500, 200);
}