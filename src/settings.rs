pub struct Setting<'a> {
    pub key: &'a str,
    pub description: &'a str,
    pub default_value: bool,
}

pub fn get_settings<'a>() -> Vec<Setting<'a>> {

    vec![
        Setting{ key: "il_mode", description: "IL Mode", default_value: false},
        Setting{ key: "start", description: "Start (Global)", default_value: true },
        Setting{ key: "start_new_file", description: "Start only on empty files", default_value: true },
        Setting{ key: "reset", description: "Reset (Global)", default_value: true },
        Setting{ key: "split", description: "Split (Global)", default_value: true },
        Setting{ key: "split_simple_new_tp", description: "Split - Simple - New Time Piece", default_value: true },
        Setting{ key: "split_simple_any_tp", description: "Split - Simple - Any Time Piece", default_value: true },
        Setting{ key: "split_simple_act_entry", description: "Split - Simple - Act Entry", default_value: false },
        Setting{ key: "split_simple_dw_bth", description: "Split - Simple - Death Wish Back To Hub", default_value: false },
        Setting{ key: "splits_simple_yarn", description: "Split - Simple - Yarn", default_value: false },
        Setting{ key: "splits_simple_cp", description: "Split - Simple - Checkpoint", default_value: false },
    ]

}
