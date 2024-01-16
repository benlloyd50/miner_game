use crate::expedition::ExpeditionClear;
use bevy::prelude::*;

use crate::assets::UiAssets;
use crate::expedition::ExpeditionPersist;
use crate::{AppState, SPRITE_PX_X, SPRITE_PX_Y};

// pub const TOOLBAR_TOOLS_Z: f32 = 71.0;
pub const TOOLBAR_Z: f32 = 70.0;

pub struct ExpeditionUIPlugin;

impl Plugin for ExpeditionUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Expedition), init_expedition_ui)
            .add_systems(Update, expedition_buttons.run_if(in_state(AppState::Expedition)));
    }
}

fn init_expedition_ui(mut commands: Commands, ui_assets: Res<UiAssets>) {
    info!("creating ui elements for expedition");
    // TODO: change to how ever many tools are currently unlocked
    // TODO2 change the transform to be centered based on how many are unlocked
    let y = -((3 * SPRITE_PX_Y) as f32);
    for i in 0..7 {
        let x = ((i * 3) * SPRITE_PX_X) as f32 - (2 * SPRITE_PX_X) as f32;
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(x, y, TOOLBAR_Z),
                texture: ui_assets.tool_shadow.clone(),
                ..default()
            },
            ExpeditionPersist,
        ));
    }

    commands.spawn((
        ImageBundle {
            style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), ..Default::default() },
            image: UiImage::new(ui_assets.level1fg.clone()),
            z_index: ZIndex::Global(2),
            ..Default::default()
        },
        ExpeditionPersist,
    ));

    let leave_style =
        TextStyle { font: ui_assets.text.clone(), font_size: 32.0, color: Color::rgb_u8(255, 241, 169), ..default() };
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(8.0),
                    bottom: Val::Percent(12.0),
                    width: Val::Px(125.0),
                    height: Val::Px(40.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                border_color: BorderColor(Color::BLACK),
                background_color: Color::RED.into(),
                image: UiImage::new(ui_assets.leave_button.clone()),
                z_index: ZIndex::Global(1),
                ..default()
            },
            ExpeditionPersist,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section("Leave", leave_style).with_alignment(TextAlignment::Center),
                ..default()
            });
        });

}

const DARK_MAROON: Color = Color::rgb(122.0 / 255.0, 40.0 / 255.0, 73.0 / 255.0);
const RED: Color = Color::rgb(183.0 / 255.0, 65.0 / 255.0, 50.0 / 255.0);
const LIGHT_RED: Color = Color::rgb(194.0 / 255.0, 55.0 / 255.0, 83.0 / 255.0);

fn expedition_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut ev: EventWriter<ExpeditionClear>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                ev.send(ExpeditionClear::EarlyLeave);
                bg_color.0 = DARK_MAROON;
            }
            Interaction::Hovered => {
                bg_color.0 = LIGHT_RED;
            }
            Interaction::None => {
                *bg_color = BackgroundColor::DEFAULT;
            }
        }
    }
}
