use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;

use json5;
use serde::Deserialize;

pub static LEVEL_DB: OnceLock<HashMap<String, AreaInfo>> = OnceLock::new();

#[derive(Deserialize)]
pub struct AreaInfo {
    pub levels: Vec<LevelInfo>,
}

#[derive(Deserialize)]
pub struct LevelInfo {
    pub size: (usize, usize),
}

const AREA_INFO_PATH: &str = "assets/levels.json5";

pub fn load_area_info_into_db() {
    let ai_str = fs::read_to_string(AREA_INFO_PATH).unwrap();
    let area_info =
        json5::from_str(&ai_str).expect(&format!("{AREA_INFO_PATH} had bad data, look into it"));
    let _ = LEVEL_DB.set(area_info);
}
