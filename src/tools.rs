use bevy::prelude::{
    in_state, App, Event, EventReader, Input, IntoSystemConfigs, KeyCode, Plugin, Res, ResMut, Resource, Update,
};

use crate::expedition::in_area_state;
use crate::AppState;

pub struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolUnlocks>()
            .init_resource::<ActiveTool>()
            .add_event::<ToolUnlockEvent>()
            .add_systems(Update, (switch_tool,).run_if(in_state(AppState::Expedition)))
            .add_systems(Update, (unlock_tool).run_if(in_area_state));
    }
}

#[derive(Event)]
pub struct ToolUnlockEvent(ToolType);

#[derive(Resource, Default)]
pub struct ToolUnlocks {
    tiny_hammer: Lock,
    pickaxe_horizontal: Lock,
    pickaxe_vertical: Lock,
    pickaxe_cross: Lock,
}

#[derive(Resource, Default)]
pub struct ActiveTool(pub ToolType);

#[derive(Default)]
pub enum ToolType {
    #[default]
    TinyHammer,
    Pickaxe {
        rotation: PickaxeRotation,
    },
}

#[derive(Copy, Clone)]
pub enum PickaxeRotation {
    Horizontal,
    Vertical,
    Cross,
}

#[derive(Default)]
enum Lock {
    Unlocked,
    #[default]
    Locked,
}

impl ToolUnlocks {
    pub fn get_total_unlocks(&self) -> u8 {
        let mut total = 0;

        if self.tiny_hammer.is_unlocked() {
            total += 1;
        }
        if self.pickaxe_vertical.is_unlocked() {
            total += 1;
        }
        if self.pickaxe_horizontal.is_unlocked() {
            total += 1;
        }
        if self.pickaxe_cross.is_unlocked() {
            total += 1;
        }

        total
    }
}

impl Lock {
    fn is_unlocked(&self) -> bool {
        matches!(self, Lock::Unlocked)
    }
}

fn switch_tool(mut tool: ResMut<ActiveTool>, keeb: Res<Input<KeyCode>>) {
    for just_pressed in keeb.get_just_pressed() {
        match just_pressed {
            KeyCode::Key1 => {
                tool.0 = ToolType::TinyHammer;
            }
            KeyCode::Key2 => {
                tool.0 = match tool.0 {
                    ToolType::Pickaxe { rotation } => {
                        let rotation = match rotation {
                            PickaxeRotation::Horizontal => PickaxeRotation::Vertical,
                            PickaxeRotation::Vertical => PickaxeRotation::Cross,
                            PickaxeRotation::Cross => PickaxeRotation::Horizontal,
                        };
                        ToolType::Pickaxe { rotation }
                    }
                    _ => ToolType::Pickaxe { rotation: PickaxeRotation::Vertical },
                };
            }
            _ => {}
        }
    }
}

fn unlock_tool(mut ev_tool_unlocks: EventReader<ToolUnlockEvent>, mut curr_tool_unlocks: ResMut<ToolUnlocks>) {
    for ev in ev_tool_unlocks.read() {
        match ev.0 {
            ToolType::TinyHammer => curr_tool_unlocks.tiny_hammer = Lock::Unlocked,
            ToolType::Pickaxe { rotation } => match rotation {
                PickaxeRotation::Horizontal => curr_tool_unlocks.pickaxe_horizontal = Lock::Unlocked,
                PickaxeRotation::Vertical => curr_tool_unlocks.pickaxe_vertical = Lock::Unlocked,
                PickaxeRotation::Cross => curr_tool_unlocks.pickaxe_cross = Lock::Unlocked,
            },
        }
    }
}
