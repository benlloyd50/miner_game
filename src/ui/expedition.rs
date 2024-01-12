use bevy::prelude::*;

use crate::AppState;

use super::{StateUIMaster, cleanup};


pub struct ExpeditionUIPlugin;

impl Plugin for ExpeditionUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Expedition), init_expedition_ui)
            .add_systems(OnExit(AppState::Expedition), cleanup);
    }
}


fn init_expedition_ui(mut commands: Commands) {
    commands.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    ..default()
                },
                ..default()
            },
            StateUIMaster,
        ));
}
