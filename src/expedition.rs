use std::fmt::Display;

use bevy::log::{error, warn};
use bevy::prelude::{
    in_state, Commands, Component, Entity, Event, EventReader, EventWriter, Input, IntoSystemConfigs, KeyCode,
    NextState, OnExit, Plugin, Query, Res, ResMut, State, Update, With,
};

use crate::camera::CameraUpdate;
use crate::data_read::LEVEL_DB;
use crate::stability::{LevelStability, Stability};
use crate::AppState;

pub struct ExpeditionPlugin;

impl Plugin for ExpeditionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<LevelChange>()
            .add_event::<InitExpedition>()
            .add_systems(Update, (setup_expedition, stop_expedition).run_if(in_area_state))
            .add_systems(Update, (stop_expedition).run_if(in_state(AppState::Expedition)))
            .add_systems(OnExit(AppState::Expedition), cleanup_expedition);
    }
}

pub fn in_area_state(app_state: Res<State<AppState>>) -> bool {
    matches!(app_state.to_owned(), AppState::AreaViewer { .. })
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
#[allow(unused)]
pub enum Area {
    TheCaves,
    TheCollapse,
}

impl Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            Area::TheCaves => "The Caves",
            Area::TheCollapse => "The Collapse",
        };
        write!(f, "{}", txt)
    }
}

#[derive(Event, Debug)]
pub struct InitExpedition {
    pub size_x: usize,
    pub size_y: usize,
}

/// Marks an entity as something that persists only for the lifetime of the current expedition
#[derive(Component)]
pub struct ExpeditionPersist;

// Flow of events
// In map view of levels, AppState::LevelViewer{ area }
// 1. Player selects level, creates Level Change event
// 2. LevelChange event is read and setups up components necessary for AppState::Expedition
// 3. State is changed to Expedition

#[derive(Event)]
pub struct LevelChange {
    pub area: Area,
    pub level_idx: usize,
}

fn setup_expedition(
    mut ev_level_change: EventReader<LevelChange>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ev_init_mining_grid: EventWriter<InitExpedition>,
    mut ev_cam_update: EventWriter<CameraUpdate>,
    mut stability: ResMut<Stability>,
) {
    // only process 1 level change event in case two+ are created
    let Some(ev) = ev_level_change.read().next() else {
        return;
    };
    // take area and level and get the level info from level DB
    let Some(area_info) = LEVEL_DB.get() else {
        error!("did not find the level db and it was not initialized");
        return;
    };

    // use level info to create generation events (mining grid, stability, treasures)
    let Some(info) = area_info.get(&ev.area.to_string()) else {
        warn!("No level found for {} in area {}", ev.level_idx, ev.area);
        return;
    };

    let level = &info.levels[ev.level_idx];
    ev_init_mining_grid.send(InitExpedition { size_x: level.size.0, size_y: level.size.1 });
    ev_cam_update.send(CameraUpdate { width: level.size.0 as f32, height: level.size.1 as f32, scale: 2.0 });
    *stability = match level.stability {
        LevelStability::Normal => Stability::new(10),
    };

    // switch state
    next_state.set(AppState::Expedition)
}

fn cleanup_expedition(mut commands: Commands, q_expedition_entities: Query<Entity, With<ExpeditionPersist>>) {
    for e in q_expedition_entities.iter() {
        commands.entity(e).despawn();
    }
}

fn stop_expedition(mut next_state: ResMut<NextState<AppState>>, keyboard: Res<Input<KeyCode>>) {
    if !keyboard.just_pressed(KeyCode::Back) {
        return;
    }
    next_state.set(AppState::AreaViewer { curr_area: crate::expedition::Area::TheCaves });
}
