use bevy::prelude::*;

use crate::{AppState, SPRITE_PX_X, SPRITE_PX_Y};

pub struct CameraPlugin;

pub const CAMERA_Z: f32 = 100.0;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CameraUpdate>()
            .add_systems(Startup, init_camera)
            .add_systems(OnEnter(AppState::Expedition), update_camera_for_mine);
    }
}

#[derive(Component)]
pub struct MainCamera;

fn init_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle { transform: Transform::from_xyz(0.0, 0.0, CAMERA_Z), ..default() }, MainCamera));
}

#[derive(Event)]
pub struct CameraUpdate {
    pub width: f32,
    pub height: f32,
}

fn update_camera_for_mine(
    mut q_camera: Query<&mut Transform, With<MainCamera>>,
    mut ev_cam_move: EventReader<CameraUpdate>,
) {
    let mut cam = q_camera.single_mut();
    let Some(ev) = ev_cam_move.read().next() else {
        return;
    };
    *cam = Transform::from_xyz(
        ev.width as f32 * SPRITE_PX_X as f32 / 2.0,
        ev.height as f32 * SPRITE_PX_Y as f32 / 2.0,
        CAMERA_Z,
    );
}
