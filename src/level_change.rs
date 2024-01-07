use bevy::log::{error, warn};

use std::fmt::Display;

use bevy::prelude::{
    Event, EventReader, EventWriter, IntoSystemConfigs, NextState, OnEnter, Plugin, Res, ResMut,
    State, Update,
};

use crate::{data_read::LEVEL_DB, mining::InitMiningGrid, AppState};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<LevelChange>()
            .add_systems(
                OnEnter(AppState::AreaViewer {
                    curr_area: Area::TheCaves,
                }),
                debug_level_switch,
            )
            .add_systems(Update, setup_level.run_if(in_area_state));
    }
}

fn in_area_state(app_state: Res<State<AppState>>) -> bool {
    matches!(app_state.to_owned(), AppState::AreaViewer { .. })
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Area {
    TheCaves,
}

impl Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            Area::TheCaves => "The Caves",
        };
        write!(f, "{}", txt)
    }
}

// Flow of events
// In map view of levels, AppState::LevelViewer{ area }
// 1. Player selects level, creates Level Change event
// 2. LevelChange event is read and setups up components necessary for AppState::Expedition
// 3. State is changed to Expedition

#[derive(Event)]
pub struct LevelChange {
    area: Area,
    level: usize,
}

fn debug_level_switch(mut ev: EventWriter<LevelChange>) {
    ev.send(LevelChange {
        area: Area::TheCaves,
        level: 0,
    });
}

fn setup_level(
    mut ev_level_change: EventReader<LevelChange>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ev_init_mining_grid: EventWriter<InitMiningGrid>,
) {
    // only process 1 level change event in case two+ are created
    let Some(ev) = ev_level_change.read().next() else {
        return;
    };
    // take area and level and get the level info from level DB
    let area_info = match LEVEL_DB.get() {
        Some(a) => a,
        None => {
            error!("did not find the level db and it was not initialized");
            return;
        }
    };

    // use level info to create generation events (mining grid, stability)
    match area_info.get(&ev.area.to_string()) {
        Some(info) => {
            let level = &info.levels[ev.level];
            ev_init_mining_grid.send(InitMiningGrid {
                size_x: level.size.0,
                size_y: level.size.1,
            });
        }
        None => {
            warn!("No level found for {} in area {}", ev.level, ev.area);
            ev_init_mining_grid.send(InitMiningGrid {
                size_x: 10,
                size_y: 10,
            });
        }
    }

    // switch state
    next_state.set(AppState::Expedition)
}
