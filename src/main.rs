mod assets;
mod data_read;
mod expedition;
mod level_change;
mod mining;
mod stability;

use assets::AssetLoadPlugin;
use bevy::prelude::*;
use data_read::load_area_info_into_db;
use level_change::{Area, LevelPlugin};
use mining::MiningPlugin;
use stability::StabilityPlugin;

fn main() {
    load_area_info_into_db();

    App::new()
        .add_plugins((
            DefaultPlugins,
            MiningPlugin,
            AssetLoadPlugin,
            StabilityPlugin,
            LevelPlugin,
        ))
        .add_state::<AppState>()
        .add_systems(Startup, init_camera)
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

const GRID_WIDTH: usize = 10;
const GRID_HEIGHT: usize = 10;

#[derive(Component)]
struct MainCamera;

fn init_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(
                GRID_WIDTH as f32 * SPRITE_PX_X as f32 / 2.0,
                GRID_HEIGHT as f32 * SPRITE_PX_Y as f32 / 2.0,
                10.0,
            ),
            ..default()
        },
        MainCamera,
    ));
}
