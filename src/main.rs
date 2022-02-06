
fn main() {
    println!(
        "{}",
        bf_cell_gen::bfcg::compiler::compiler
            ::file_minimalize("++-[+ + +]0-\n42;24;1  5\nno ne++;;;;;#xx#z;;\n   #yy# w").ok().unwrap()
    );
    let x = bf_cell_gen::bfcg::dev_emulators::win::Win::new(500, 200);
    x.run();

    println!("hehe ! we go !");
    
    let x = bf_cell_gen::bfcg::dev_emulators::win::Win::new(400, 400);
    x.run();

    println!("and after second :)");
}