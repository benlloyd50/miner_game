use std::{collections::HashMap, fs, sync::OnceLock};

use json5;
use serde::Deserialize;

use crate::stability::LevelStability;

pub static LEVEL_DB: OnceLock<HashMap<String, AreaInfo>> = OnceLock::new();
pub static TREASURE_DB: OnceLock<Vec<TreasureInfo>> = OnceLock::new();

#[derive(Deserialize)]
pub struct TreasureInfo {
    pub id: u32,
    pub name: String,
    // a 1d array representing the shape and atlas indices of the treasure
    // -1 represents an empty cell
    pub shape: Vec<i32>,
    pub width: usize,
    pub height: usize,
}

#[derive(Deserialize)]
pub struct AreaInfo {
    pub levels: Vec<LevelInfo>,
}

#[derive(Deserialize)]
pub struct LevelInfo {
    pub name: String,
    pub size: (usize, usize),
    pub stability: LevelStability,
}

const AREA_INFO_PATH: &str = "assets/levels.json5";
const TREASURE_PATH: &str = "assets/treasures.json5";

pub fn load_area_info_into_db() {
    let ai_str = fs::read_to_string(AREA_INFO_PATH).unwrap();
    let area_info = json5::from_str(&ai_str).expect(&format!("{AREA_INFO_PATH} had bad data, look into it"));
    let _ = LEVEL_DB.set(area_info);
}

pub fn load_treasures_into_db() {
    let treasure_str = fs::read_to_string(TREASURE_PATH).unwrap();
    let treasure_info = json5::from_str(&treasure_str).expect(&format!("{TREASURE_PATH} had bad data, look into it"));
    let _ = TREASURE_DB.set(treasure_info);
}
