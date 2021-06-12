#[derive(Debug, Clone)]
pub struct Config {
    pub workspaces: Vec<String>,
    pub gap_px: u32,
    pub main_ratio_inc: f32,
    pub float_classes: Vec<String>,
    pub unfocused: u32,
    pub focused: u32,
    pub urgent: u32,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            workspaces: vec!["1".into(), "2".into(), "3".into()],
            gap_px: 0,
            main_ratio_inc: 0.05,
            float_classes: Vec::new(),
            unfocused: 0x555555,
            focused: 0xdddddd,
            urgent: 0xee00000,
        }
    }
}

//todo: add validation, builder, etc