use std::fmt::Display;

use bevy::{
    log::{error, warn},
    prelude::{
        in_state, App, Commands, Component, DespawnRecursiveExt, Entity, Event, EventReader, EventWriter, Input,
        IntoSystemConfigs, KeyCode, NextState, OnExit, Plugin, Query, Res, ResMut, State, Update, With,
    },
};

use crate::{
    camera::CameraUpdate,
    data_read::LEVEL_DB,
    stability::{LevelStability, Stability},
    AppState,
};

pub struct ExpeditionPlugin;

impl Plugin for ExpeditionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LevelChange>()
            .add_event::<InitExpedition>()
            .add_event::<ExpeditionClear>()
            .add_event::<ExpeditionFinish>()
            .add_systems(Update, (setup_expedition, stop_expedition).run_if(in_area_state))
            .add_systems(Update, (check_expedition_clear).run_if(in_state(AppState::Expedition)))
            .add_systems(Update, (check_expedition_finish).run_if(in_state(AppState::ExpeditionFinish)))
            .add_systems(OnExit(AppState::ExpeditionFinish), cleanup_expedition);

        debug_assert!(debug_leave_expedition(app));
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

#[derive(Event)]
pub enum ExpeditionClear {
    AllTreasure,
    AbandonSession,
    _CaveIn,
}

#[derive(Event)]
// TODO: remove this as it was made as a hacky fix to another problem with button presses
pub enum ExpeditionFinish {
    Finish,
    _Retry,
}

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
    // only process 1st level change
    let Some(ev) = ev_level_change.read().next() else {
        return;
    };

    // take area and level and get the level info from level DB
    let Some(level_db) = LEVEL_DB.get() else {
        error!("did not find the level db and it was not initialized");
        return;
    };

    // use level info to create generation events (mining grid, stability, treasures)
    let Some(info) = level_db.get(&ev.area.to_string()) else {
        warn!("No level found for {} in area {}", ev.level_idx, ev.area);
        return;
    };

    let level = &info.levels[ev.level_idx];
    ev_init_mining_grid.send(InitExpedition { size_x: level.size.0, size_y: level.size.1 });
    ev_cam_update.send(CameraUpdate { width: level.size.0 as f32, height: level.size.1 as f32, scale: 2.0 });
    *stability = match level.stability {
        LevelStability::Normal => Stability::new(1000),
    };

    // switch state
    next_state.set(AppState::Expedition)
}

fn check_expedition_clear(
    mut ev_expedition_clear: EventReader<ExpeditionClear>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(ec) = ev_expedition_clear.read().next() else {
        return;
    };
    match ec {
        ExpeditionClear::AllTreasure => {
            next_state.set(AppState::ExpeditionFinish);
        }
        ExpeditionClear::AbandonSession => {
            next_state.set(AppState::ExpeditionFinish);
        }
        ExpeditionClear::_CaveIn => {}
    }
}

fn check_expedition_finish(
    mut ev_expedition_finish: EventReader<ExpeditionFinish>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(ec) = ev_expedition_finish.read().next() else {
        return;
    };
    match ec {
        ExpeditionFinish::Finish => {
            // TODO: update to switch back to the area that the player is currently in
            next_state.set(AppState::AreaViewer { curr_area: crate::expedition::Area::TheCaves });
        }
        ExpeditionFinish::_Retry => {
            todo!("add retry feature");
        }
    }
}

fn cleanup_expedition(mut commands: Commands, q_expedition_entities: Query<Entity, With<ExpeditionPersist>>) {
    for e in q_expedition_entities.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn debug_leave_expedition(app: &mut App) -> bool {
    app.add_systems(Update, (stop_expedition).run_if(in_state(AppState::Expedition)));
    true
}

fn stop_expedition(mut next_state: ResMut<NextState<AppState>>, keeb: Res<Input<KeyCode>>) {
    if !keeb.just_pressed(KeyCode::Back) {
        return;
    }
    // TODO: update to switch back to the area that the player is currently in
    next_state.set(AppState::AreaViewer { curr_area: crate::expedition::Area::TheCaves });
}
