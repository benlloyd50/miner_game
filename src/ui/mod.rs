mod expedition;

use bevy::prelude::*;

use crate::assets::FontAssets;
use crate::data_read::LEVEL_DB;
use crate::expedition::{in_area_state, Area, LevelChange};
use crate::AppState;

pub struct UIPlugins;
impl Plugin for UIPlugins {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::AreaViewer { curr_area: Area::TheCaves }), setup)
            .add_systems(Update, button_system.run_if(in_area_state))
            .add_systems(OnExit(AppState::AreaViewer { curr_area: Area::TheCaves }), cleanup);
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct ButtonLevelData {
    level_idx: usize,
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor, &ButtonLevelData),
        (Changed<Interaction>, With<Button>),
    >,
    mut ev: EventWriter<LevelChange>,
) {
    for (interaction, mut color, mut border_color, bld) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;
                ev.send(LevelChange { area: Area::TheCaves, level_idx: bld.level_idx });
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

#[derive(Component)]
pub struct StateUIMaster;

fn setup(mut commands: Commands, fonts: Res<FontAssets>) {
    debug!("setting up ui for area viewer");
    let Some(info) = LEVEL_DB.get() else {
        return;
    };
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
            debug!("does info work");
            let Some(area) = info.get(&Area::TheCaves.to_string()) else {
                return;
            };
            for (idx, level) in area.levels.iter().enumerate() {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(200.0),
                                height: Val::Auto,
                                border: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            border_color: BorderColor(Color::BLACK),
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        ButtonLevelData { level_idx: idx },
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            format!("{}", level.name),
                            TextStyle { font_size: 40.0, color: Color::rgb(0.9, 0.9, 0.9), font: fonts.text.clone() },
                        ));
                    });
            }
        });
}

/// Removes the ui that is only alive for the lifetime of the scene
pub(crate) fn cleanup(mut commands: Commands, ui_q: Query<Entity, With<StateUIMaster>>) {
    for e in ui_q.iter() {
        commands.entity(e).despawn_recursive();
    }
}
