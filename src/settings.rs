pub struct Setting<'a> {
    pub key: &'a str,
    pub description: &'a str,
    pub default_value: bool,
}

pub fn get_settings<'a>() -> Vec<Setting<'a>> {

    vec![
        Setting{ key: "start_new_file", description: "Start only on empty files", default_value: true }
    ]

}