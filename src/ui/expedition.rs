use bevy::prelude::*;

use crate::assets::UiAssets;
use crate::expedition::{ExpeditionClear, ExpeditionPersist, ExpeditionFinish};
use crate::tools::ToolUnlocks;
use crate::{AppState, SPRITE_PX_X, SPRITE_PX_Y};

// pub const TOOLBAR_TOOLS_Z: f32 = 71.0;
pub const TOOLBAR_Z: f32 = 70.0;

#[derive(Component)]
pub struct ExpeditionClearMenu;

#[derive(Component)]
pub struct LeaveButton;

pub struct ExpeditionUIPlugin;

impl Plugin for ExpeditionUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Expedition), init_expedition_ui)
            .add_systems(Update, (expedition_buttons).run_if(in_either_expedition_state))
            .add_systems(Update, (reveal_clear_menu).run_if(in_state(AppState::Expedition)));
    }
}

fn in_either_expedition_state(app_state: Res<State<AppState>>) -> bool {
    match app_state.to_owned() {
        AppState::ExpeditionFinish | AppState::Expedition => true,
        _ => false,
    }
}

fn init_expedition_ui(mut commands: Commands, ui_assets: Res<UiAssets>, tool_unlocks: Res<ToolUnlocks>) {
    info!("creating ui elements for expedition");
    let tools_unlocked = tool_unlocks.get_total_unlocks();
    // TODO 2 change the transform to be centered based on how many are unlocked
    let y = -((3 * SPRITE_PX_Y) as f32);
    for i in 0..tools_unlocked as u32 {
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
            LeaveButton
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section("Leave", leave_style).with_alignment(TextAlignment::Center),
                ..default()
            });
        });

    let cleared_style =
        TextStyle { font: ui_assets.text.clone(), font_size: 32.0, color: Color::rgb_u8(255, 241, 169), ..default() };
    commands
        .spawn((
            ImageBundle {
                style: Style {
                    width: Val::Percent(50.0),
                    height: Val::Percent(80.0),
                    position_type: PositionType::Absolute,
                    top: Val::Px(85.0),
                    left: Val::Percent(25.0),
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                visibility: Visibility::Hidden,
                image: UiImage::new(ui_assets.clear_menu.clone()),
                ..default()
            },
            ExpeditionClearMenu,
            ExpeditionPersist,
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section("Expedition Cleared!", cleared_style)
                    .with_text_alignment(TextAlignment::Center),
            );
        });
}

const DARK_MAROON: Color = Color::rgb(122.0 / 255.0, 40.0 / 255.0, 73.0 / 255.0);
// const RED: Color = Color::rgb(183.0 / 255.0, 65.0 / 255.0, 50.0 / 255.0);
const LIGHT_RED: Color = Color::rgb(194.0 / 255.0, 55.0 / 255.0, 83.0 / 255.0);

fn expedition_buttons(
    mut q_interaction: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<LeaveButton>)>,
    mut ev_clear: EventWriter<ExpeditionClear>,
    mut ev_finish: EventWriter<ExpeditionFinish>,
    curr_state: Res<State<AppState>>,
) {
    for (interaction, mut bg_color) in &mut q_interaction {
        match *interaction {
            Interaction::Pressed => {
                match curr_state.to_owned() {
                    AppState::Expedition => {
                        ev_clear.send(ExpeditionClear::AbandonSession);
                        info!("sent an AbandonSession");
                    }
                    AppState::ExpeditionFinish => {
                        ev_finish.send(ExpeditionFinish::Finish);
                        info!("sent a Finish");
                    },
                    _ => { unreachable!("expedition_buttons should not be ran in this state");}
                }
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

fn reveal_clear_menu(
    mut ev_expedition_clear: EventReader<ExpeditionClear>,
    mut q_clear_menu: Query<&mut Visibility, With<ExpeditionClearMenu>>,
) {
        let Some(_ec) = ev_expedition_clear.read().next() else {
        return;
    };
    let mut clear_menu = q_clear_menu.single_mut();
    match *clear_menu {
        Visibility::Hidden => {
            info!("Reading an expedition clear, first time");
            *clear_menu = Visibility::Visible;
        }
        _ => {}
    }
}
