use bevy::prelude::*;

use super::{cleanup, StateUIMaster};
use crate::assets::UiAssets;
use crate::AppState;

pub struct ExpeditionUIPlugin;

impl Plugin for ExpeditionUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Expedition), init_expedition_ui)
            .add_systems(OnExit(AppState::Expedition), cleanup);
    }
}

fn init_expedition_ui(mut commands: Commands, ui_assets: Res<UiAssets>) {
    commands
        .spawn((
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
        ))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    // style: Style {
                    //     ..default()
                    // },
                    background_color: Color::WHITE.into(),
                    ..default()
                },
                UiImage::new(ui_assets.tool_shadow.clone()),
            ));
        });
}
