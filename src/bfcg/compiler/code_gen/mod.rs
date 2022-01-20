mod code_gen;

pub use code_gen::cgen_set_cell_value;
pub use code_gen::add_cgen_set_cell_value;

pub use code_gen::cgen_shift_cell_ptr;

pub use code_gen::add_cgen_cell_create;



pub use code_gen::cgen_set_cell_max_cmds;

pub use code_gen::cgen_zero_while_not_zero;
pub use code_gen::add_cgen_zero_while_not_zero;

pub use code_gen::cgen_move_to_next_after_left_zero;
pub use code_gen::add_cgen_move_to_next_after_left_zero;

pub use code_gen::cgen_move_to_prev_before_right_zero;
pub use code_gen::add_cgen_move_to_prev_before_right_zero;

pub use code_gen::add_cgen_init_se_cem;