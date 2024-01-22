use bevy::prelude::*;
use bevy_mod_picking::{
    backend::prelude::Pickable,
    events::{Down, Pointer},
    prelude::On,
};

use crate::{
    assets::{SpriteAssets, UiAssets},
    expedition::{ExpeditionLeave, ExpeditionPersist, ExpeditionStatus},
    stability::{Stability, StabilityDamage},
    tools::{ui_tool_is_tool_type, ActiveTool, SwitchTool, ToolUnlocks},
    AppState, SystemOrder, SPRITE_PX_X, SPRITE_PX_Y,
};

pub const TOOLS_Z: f32 = 71.0;
pub const TOOLBAR_Z: f32 = 70.0;
// leave button colors
const DARK_MAROON: Color = Color::rgb(122.0 / 255.0, 40.0 / 255.0, 73.0 / 255.0);
const _RED: Color = Color::rgb(183.0 / 255.0, 65.0 / 255.0, 50.0 / 255.0);
const LIGHT_RED: Color = Color::rgb(194.0 / 255.0, 55.0 / 255.0, 83.0 / 255.0);

#[derive(Component)]
pub struct ExpeditionClearMenu;

#[derive(Component)]
pub struct LeaveButton;

#[derive(Component)]
pub struct StabilityText;

#[derive(Component, Copy, Clone)]
pub enum UITool {
    TinyHammer,
    Pickaxe,
}

pub struct ExpeditionUIPlugin;

impl Plugin for ExpeditionUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Expedition), init_expedition_ui)
            .add_systems(Update, (expedition_buttons).run_if(in_state(AppState::Expedition)))
            .add_systems(
                Update,
                (update_stability_text)
                    .run_if(on_event::<StabilityDamage>())
                    .in_set(SystemOrder::Render)
                    .after(SystemOrder::Logic),
            )
            .add_systems(
                Update,
                (update_active_tool_sprite)
                    .run_if(on_event::<SwitchTool>())
                    .in_set(SystemOrder::Render)
                    .after(SystemOrder::Logic),
            )
            .add_systems(Update, (reveal_clear_menu).run_if(in_state(AppState::Expedition)));
    }
}

// TODO: maybe change the starting x position to be centered based on how many tools are unlocked
fn init_expedition_ui(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    sprites: Res<SpriteAssets>,
    tool_unlocks: Res<ToolUnlocks>,
    active_tool: Res<ActiveTool>,
) {
    info!("SETUP: creating ui elements for expedition");
    let tools_unlocked = tool_unlocks.get_tools_for_ui();
    let bar_y = -((3 * SPRITE_PX_Y) as f32);
    let tool_y = -((3 * SPRITE_PX_Y - SPRITE_PX_Y / 2) as f32);
    for (i, tool) in tools_unlocked.iter().enumerate() {
        let x = ((i as u32 * 3) * SPRITE_PX_X) as f32 - (2 * SPRITE_PX_X) as f32;
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(x, bar_y, TOOLBAR_Z),
                texture: ui_assets.tool_shadow.clone(),
                ..default()
            },
            ExpeditionPersist,
        ));

        let tool_atlas_idx = if ui_tool_is_tool_type(tool, &active_tool.0) { i + 8 } else { i };
        commands.spawn((
            SpriteSheetBundle {
                transform: Transform::from_xyz(x, tool_y, TOOLS_Z),
                sprite: TextureAtlasSprite::new(tool_atlas_idx),
                texture_atlas: sprites.tools.clone(),
                ..default()
            },
            On::<Pointer<Down>>::send_event::<SwitchTool>(),
            *tool,
            ExpeditionPersist,
        ));
    }

    // Level Foreground fg
    commands.spawn((
        ImageBundle {
            style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), ..Default::default() },
            image: UiImage::new(ui_assets.level1fg.clone()),
            z_index: ZIndex::Global(2),
            ..Default::default()
        },
        Pickable::IGNORE,
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
            LeaveButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section("Leave", leave_style).with_alignment(TextAlignment::Center),
                ..default()
            });
        });

    // DEBUG - stability text
    commands.spawn((
        TextBundle {
            text: Text::from_sections([
                TextSection::new(
                    "Stability: ",
                    TextStyle { font: ui_assets.text.clone(), font_size: 22.0, color: Color::rgb_u8(240, 10, 240) },
                ),
                TextSection::new(
                    "",
                    TextStyle { font: ui_assets.text.clone(), font_size: 22.0, color: Color::rgb_u8(255, 255, 255) },
                ),
            ]),
            z_index: ZIndex::Global(3),
            ..default()
        },
        StabilityText,
        ExpeditionPersist,
    ));

    let cleared_style =
        TextStyle { font: ui_assets.text.clone(), font_size: 28.0, color: Color::rgb_u8(255, 241, 169) };
    commands
        .spawn((
            ImageBundle {
                style: Style {
                    width: Val::Percent(50.0),
                    height: Val::Percent(80.0),
                    position_type: PositionType::Absolute,
                    top: Val::Px(85.0),
                    left: Val::Percent(25.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
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
            parent.spawn(TextBundle {
                style: Style {
                    margin: UiRect::new(Val::Percent(10.0), Val::ZERO, Val::Percent(10.0), Val::ZERO),
                    ..default()
                },
                text: Text::from_section("Expedition Cleared!", cleared_style.clone()),
                ..default()
            });

            parent.spawn(TextBundle {
                style: Style {
                    margin: UiRect::new(Val::Percent(10.0), Val::ZERO, Val::Percent(10.0), Val::ZERO),
                    ..default()
                },
                text: Text::from_section("Total Treasures Found ##", cleared_style.clone()),
                ..default()
            });
        });
}

fn update_active_tool_sprite(mut q_ui_tools: Query<(&mut TextureAtlasSprite, &UITool)>, active_tool: Res<ActiveTool>) {
    // NOTE: this 8 relates to the columns in the tools tilesheets, by addding
    // the width we get the idx of the highlighted variant of the tool sprite
    for (mut ui_tool_sprite, ui_tool) in q_ui_tools.iter_mut() {
        let old_idx = ui_tool_sprite.index;
        if ui_tool_is_tool_type(ui_tool, &active_tool.0) {
            if old_idx >= 8 {
                // the sprite was already active
                continue;
            }
            ui_tool_sprite.index = old_idx + 8;
        } else {
            // set back to original index if it was active prior
            if old_idx >= 8 {
                ui_tool_sprite.index -= 8;
            }
        }
    }
}

fn expedition_buttons(
    mut q_leave_button: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<LeaveButton>)>,
    mut ev_leave: EventWriter<ExpeditionLeave>,
) {
    for (interaction, mut bg_color) in &mut q_leave_button {
        match *interaction {
            Interaction::Pressed => {
                ev_leave.send_default();
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

fn update_stability_text(mut q_stability_text: Query<&mut Text, With<StabilityText>>, stability: Res<Stability>) {
    let mut stability_text = match q_stability_text.get_single_mut() {
        Ok(t) => t,
        Err(e) => {
            warn!("No stability text found so cannot update it., {}", e);
            return;
        }
    };

    stability_text.sections[1].value = stability.remaining.to_string();
}

fn reveal_clear_menu(
    mut q_clear_menu: Query<&mut Visibility, With<ExpeditionClearMenu>>,
    expedition_status: Res<ExpeditionStatus>,
) {
    if !(expedition_status.is_changed() && matches!(*expedition_status, ExpeditionStatus::Cleared)) {
        return;
    }

    let mut clear_menu = q_clear_menu.single_mut();
    match *clear_menu {
        Visibility::Hidden => {
            info!("Reading an expedition clear, first time");
            *clear_menu = Visibility::Visible;
        }
        _ => {}
    }
}
