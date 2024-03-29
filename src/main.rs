mod assets;
mod audio_events;
mod camera;
mod consts;
mod data_read;
mod expedition;
mod mining;
mod point;
mod stability;
mod tools;
mod treasures;
mod ui;

use assets::AssetLoadPlugin;
use audio_events::AudioEventsPlugin;
use bevy::{
    log::LogPlugin,
    prelude::*,
    window::{WindowMode, WindowResolution},
};
use bevy_kira_audio::AudioPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use camera::CameraPlugin;
use data_read::{load_area_info_into_db, load_treasures_into_db};
use expedition::{Area, ExpeditionPlugin};
use mining::MiningPlugin;
use stability::StabilityPlugin;
use tools::ToolPlugin;
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
                        resolution: WindowResolution::new(1280.0, 720.0),
                        // resolution: WindowResolution::new(1920.0, 1080.0),
                        title: "Underground Miner".to_string(),
                        resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(LogPlugin {
                    filter: "info,wgpu_core=warn,wgpu_hal=warn,mygame=debug".into(),
                    level: bevy::log::Level::DEBUG,
                })
                .set(ImagePlugin::default_nearest()),
            AudioPlugin,
            AudioEventsPlugin,
            DefaultPickingPlugins,
            CameraPlugin,
            MiningPlugin,
            ToolPlugin,
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
    // TODO: change area to resource type
    AreaViewer {
        curr_area: Area,
    },
    Expedition,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, SystemSet)]
enum SystemOrder {
    _Input, // anything to do with getting input from the player
    Logic,  // anything to do with changing state
    Render, // anything to do with drawing state
}

const SPRITE_PX_X: u32 = 16;
const SPRITE_PX_Y: u32 = SPRITE_PX_X;
