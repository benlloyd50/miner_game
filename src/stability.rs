use bevy::prelude::{in_state, Event, EventReader, IntoSystemConfigs, Plugin, ResMut, Resource, Update};
use serde::Deserialize;

use crate::AppState;

pub struct StabilityPlugin;

impl Plugin for StabilityPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Stability>()
            .add_event::<StabilityDamage>()
            .add_systems(Update, (handle_stability_damage).run_if(in_state(AppState::Expedition)));
    }
}

/// Describes the "stamina" meter of the expedition
/// Reach 0 and enter chaos cave mode
#[derive(Resource)]
pub struct Stability {
    pub remaining: i32,
}

impl Stability {
    pub fn new(value: i32) -> Self {
        Self { remaining: value }
    }
}

impl Default for Stability {
    fn default() -> Self {
        Self { remaining: 0 }
    }
}

#[derive(Deserialize)]
pub enum LevelStability {
    Normal,
}

#[derive(Event)]
pub struct StabilityDamage {
    amt: u32,
}

impl StabilityDamage {
    pub fn new(value: u32) -> Self {
        Self { amt: value }
    }
}

fn handle_stability_damage(mut stability: ResMut<Stability>, mut ev_damage: EventReader<StabilityDamage>) {
    for ev in ev_damage.read() {
        stability.remaining -= ev.amt as i32;
    }
}
