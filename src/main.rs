mod assets;
mod camera;
mod data_read;
mod expedition;
mod mining;
mod point;
mod stability;
mod treasures;
mod ui;

use assets::AssetLoadPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::{WindowMode, WindowResolution};
use camera::CameraPlugin;
use data_read::{load_area_info_into_db, load_treasures_into_db};
use expedition::{Area, ExpeditionPlugin};
use mining::MiningPlugin;
use stability::StabilityPlugin;
use treasures::TreasurePlugin;
use ui::UIPlugins;

fn main() {
    load_area_info_into_db();
    load_treasures_into_db();

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        mode: WindowMode::Windowed,
                        resolution: WindowResolution::new(800.0, 800.0),
                        title: "Underground Miner".to_string(),
                        resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(LogPlugin {
                    filter: "info,wgpu_core=warn,wgpu_hal=warn,mygame=debug".into(),
                    level: bevy::log::Level::DEBUG,
                }),
            CameraPlugin,
            MiningPlugin,
            TreasurePlugin,
            AssetLoadPlugin,
            StabilityPlugin,
            ExpeditionPlugin,
            UIPlugins,
        ))
        .add_state::<AppState>()
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

#[derive(States, PartialEq, Eq, Hash, Clone, Debug, Default)]
enum AppState {
    #[default]
    AssetLoading,
    AreaViewer {
        curr_area: Area,
    },
    Expedition,
}

const SPRITE_PX_X: usize = 32;
const SPRITE_PX_Y: usize = SPRITE_PX_X;
