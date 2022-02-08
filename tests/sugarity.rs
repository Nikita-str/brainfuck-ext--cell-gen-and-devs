use bf_cell_gen::logistics;


// cmd '$' can be realised only by cd+-<>[] 
// it shows in include std_proof_macros.bf-ext
// and in other "sugarity" by using macro %cloneAtoB%

/// must run with "--nocapture"
/// 
/// show that '0' can be realised only by cd+-<>[]
#[test]
fn compiler_test_u8_sugar_mul() {
    let path = "examples/proof_of_sugarity_some_cmds/proof_mul.bf-ext";
    let x = logistics::LogisticParams::new_empty(path);
    logistics::main_logistics(&x); 
}

/// must run with "--nocapture"
/// 
/// show that '0' can be realised only by cd+-<>[]
#[test]
fn compiler_test_u8_sugar_0() {
    let path = "examples/proof_of_sugarity_some_cmds/proof_zeroed_cell.bf-ext";
    let x = logistics::LogisticParams::new_empty(path);
    logistics::main_logistics(&x); 
}

/// must run with "--nocapture"
/// 
/// show that 'z' can be realised only by cd+-<>[]
#[test]
fn compiler_test_u8_sugar_tz() {
    let path = "examples/proof_of_sugarity_some_cmds/proof_test_zero.bf-ext";
    let x = logistics::LogisticParams::new_empty(path);
    logistics::main_logistics(&x); 
}


/// must run with "--nocapture"
/// 
/// show that '/' '&' can be realised only by cd+-<>[]
#[test]
fn compiler_test_u8_sugar_div_and() {
    let path = "examples/proof_of_sugarity_some_cmds/proof_div.bf-ext";
    let x = logistics::LogisticParams::new_empty(path);
    logistics::main_logistics(&x); 
}