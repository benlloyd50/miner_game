use bevy::{
    log::info,
    prelude::{
        on_event, App, Entity, Event, EventReader, IntoSystemConfigs, Plugin, Query, Res, ResMut, Resource, Update,
    },
};
use bevy_mod_picking::{
    events::Down,
    prelude::{ListenerInput, Pointer},
};

use crate::{expedition::in_area_state, ui::prelude::UITool, SystemOrder};

pub struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolUnlocks>()
            .init_resource::<ActiveTool>()
            .add_event::<ToolUnlockEvent>()
            .add_event::<SwitchTool>()
            .add_systems(Update, (switch_tool_from_ui,).run_if(on_event::<SwitchTool>()).in_set(SystemOrder::Logic))
            .add_systems(Update, (unlock_tool).run_if(in_area_state));
    }
}

#[derive(Event)]
pub struct ToolUnlockEvent(ToolType);

#[derive(Event)]
pub struct SwitchTool(Entity);

#[derive(Resource)]
pub struct ToolUnlocks {
    tiny_hammer: Lock,
    pickaxe_horizontal: Lock,
    pickaxe_vertical: Lock,
    pickaxe_cross: Lock,
}

#[derive(Resource, Default)]
pub struct ActiveTool(pub ToolType);

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ToolType {
    #[default]
    TinyHammer,
    Pickaxe {
        rotation: PickaxeRotation,
    },
}

impl ToolType {
    /// Checks if the tool type is Pickaxe with no concern for the rotation
    fn is_pickaxe(&self) -> bool {
        matches!(self, ToolType::Pickaxe { .. })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    pub fn get_tools_for_ui(&self) -> Vec<UITool> {
        let mut unlocked = vec![];
        if self.tiny_hammer.is_unlocked() {
            unlocked.push(UITool::TinyHammer);
        }
        if self.pickaxe_vertical.is_unlocked()
            || self.pickaxe_horizontal.is_unlocked()
            || self.pickaxe_cross.is_unlocked()
        {
            unlocked.push(UITool::Pickaxe);
        }

        unlocked
    }

    #[allow(unused)]
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

impl Default for ToolUnlocks {
    fn default() -> Self {
        Self {
            tiny_hammer: Lock::Unlocked,
            pickaxe_horizontal: Lock::Unlocked,
            pickaxe_vertical: Lock::Locked,
            pickaxe_cross: Lock::Locked,
        }
    }
}

impl Lock {
    fn is_unlocked(&self) -> bool {
        matches!(self, Lock::Unlocked)
    }
}

impl From<ListenerInput<Pointer<Down>>> for SwitchTool {
    fn from(value: ListenerInput<Pointer<Down>>) -> Self {
        Self(value.target)
    }
}

// TODO: check unlocked_tools before switching and fall back to switching to something else
// log a warning for funsies
fn switch_tool_from_ui(
    mut ev_switch: EventReader<SwitchTool>,
    mut active_tool: ResMut<ActiveTool>,
    _unlocked_tools: Res<ToolUnlocks>,
    q_ui_tools: Query<&UITool>,
) {
    for ev in ev_switch.read() {
        let Ok(new_switch_tool) = q_ui_tools.get(ev.0) else {
            info!("Could not switch tool since there is no UITool def");
            continue;
        };

        let actual_switch = match new_switch_tool {
            UITool::TinyHammer => ToolType::TinyHammer,
            UITool::Pickaxe => {
                if active_tool.0.is_pickaxe() {
                    match active_tool.0 {
                        ToolType::Pickaxe { rotation } => {
                            let rotation = match rotation {
                                PickaxeRotation::Horizontal => PickaxeRotation::Vertical,
                                PickaxeRotation::Vertical => PickaxeRotation::Cross,
                                PickaxeRotation::Cross => PickaxeRotation::Horizontal,
                            };
                            ToolType::Pickaxe { rotation }
                        }
                        _ => unreachable!("Unreachable because we know active tool is a pickaxe"),
                    }
                } else {
                    ToolType::Pickaxe { rotation: PickaxeRotation::Horizontal }
                }
            }
        };

        // DOWN HERE: check unlocked tools

        active_tool.0 = actual_switch;
        info!("switched to {:?}", actual_switch);
    }
}

// fn switch_tool(mut tool: ResMut<ActiveTool>, keeb: Res<Input<KeyCode>>) {
//     for just_pressed in keeb.get_just_pressed() {
//         match just_pressed {
//             KeyCode::Key1 => {
//                 tool.0 = ToolType::TinyHammer;
//             }
//             KeyCode::Key2 => {
//                 tool.0 = match tool.0 {
//                     ToolType::Pickaxe { rotation } => {
//                         let rotation = match rotation {
//                             PickaxeRotation::Horizontal => PickaxeRotation::Vertical,
//                             PickaxeRotation::Vertical => PickaxeRotation::Cross,
//                             PickaxeRotation::Cross => PickaxeRotation::Horizontal,
//                         };
//                         ToolType::Pickaxe { rotation }
//                     }
//                     _ => ToolType::Pickaxe { rotation: PickaxeRotation::Vertical },
//                 };
//             }
//             _ => {}
//         }
//     }
// }

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

pub fn ui_tool_is_tool_type(ui_tool: &UITool, tool_type: &ToolType) -> bool {
    match ui_tool {
        UITool::TinyHammer => matches!(tool_type, ToolType::TinyHammer),
        UITool::Pickaxe => matches!(tool_type, ToolType::Pickaxe { .. }),
    }
}
