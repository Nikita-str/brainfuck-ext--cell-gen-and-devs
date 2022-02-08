use bf_cell_gen::logistics;


/// must run with "--nocapture"
#[test]
fn compiler_test_u8_helwo_wowld() {
    let path = "examples/examples/01_helwo_demv_wowld.bf-ext";
    let x = logistics::LogisticParams::new_empty(path);
    logistics::main_logistics(&x); 
}


// WIN example can't run from not main thread, but in test used not main thread => can't run win-example as test
// => cant run as #[test] "02_..." & "03_..."
// so run program with "-e 2" or "-e 3"
