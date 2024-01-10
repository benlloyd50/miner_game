use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use crate::assets::SpriteAssets;
use crate::camera::MainCamera;
use crate::expedition::{ExpeditionPersist, InitExpedition};
use crate::point::xy_to_idx;
use crate::treasures::CheckTreasure;
use crate::{AppState, SPRITE_PX_X, SPRITE_PX_Y};

const BREAKABLE_Z: f32 = 30.0;
const BACKGROUND_Z: f32 = 1.0;

pub struct MiningPlugin;

impl Plugin for MiningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Expedition), init_mining_grid)
            .add_systems(
                Update,
                (
                    player_mouse_mine.before(handle_mine_actions),
                    handle_mine_actions,
                    update_mining_tile.after(handle_mine_actions),
                )
                    .run_if(in_state(AppState::Expedition)),
            )
            .add_event::<MineAction>();
    }
}

#[derive(Component)]
pub struct MiningGrid {
    pub rock_tiles: Vec<Option<Entity>>,
    pub width: usize,
    pub height: usize,
}

impl MiningGrid {
    fn new(width: usize, height: usize) -> Self {
        Self { rock_tiles: vec![None; width * height], width, height }
    }
}

#[derive(Component)]
pub struct MiningTile {
    pub hp: usize,
}

impl MiningTile {
    fn new(hp: usize) -> Self {
        Self { hp }
    }
}

#[derive(Event)]
struct MineAction(pub Entity);

fn init_mining_grid(mut commands: Commands, mut ev_init: EventReader<InitExpedition>, sprites: Res<SpriteAssets>) {
    let Some(new_grid) = ev_init.read().next() else {
        info!("entering expedition state, no event to create mining grid");
        return;
    };
    info!("running init mining grid {:?}", new_grid);

    let mut grid = MiningGrid::new(new_grid.size_x, new_grid.size_y);
    let mut rng = rand::thread_rng();

    for y in 0..grid.height {
        for x in 0..grid.width {
            let tile_idx = xy_to_idx(x, y, grid.width) as usize;
            let x = (x * SPRITE_PX_X) as f32;
            let y = (y * SPRITE_PX_Y) as f32;
            let hp: usize = rng.gen::<usize>() % 4;
            let tile = commands.spawn((
                MiningTile::new(hp + 1),
                SpriteSheetBundle {
                    texture_atlas: sprites.mining_rocks.clone(),
                    sprite: TextureAtlasSprite::new(hp),
                    transform: Transform::from_xyz(x, y, BREAKABLE_Z),
                    ..default()
                },
                ExpeditionPersist,
            ));
            grid.rock_tiles[tile_idx] = Some(tile.id());

            let _bg = commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: sprites.mining_rocks.clone(),
                    sprite: TextureAtlasSprite::new(5),
                    transform: Transform::from_xyz(x, y, BACKGROUND_Z),
                    ..default()
                },
                ExpeditionPersist,
            ));
        }
    }
    commands.spawn(grid);
    info!("created mining grid");
}

fn player_mouse_mine(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    q_mining_grid: Query<&MiningGrid>,
    mut ev_mine: EventWriter<MineAction>,
    mouse: Res<Input<MouseButton>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let (cam, cam_trans) = q_camera.single();
    let window = q_windows.single();

    if let Some(world_pos) = window
        .cursor_position()
        .and_then(|cursor| cam.viewport_to_world(cam_trans, cursor))
        .map(|ray| ray.origin.truncate())
    {
        let tile_x = (world_pos.x / SPRITE_PX_X as f32).round() as i32;
        let tile_y = (world_pos.y / SPRITE_PX_Y as f32).round() as i32;
        println!("World coords: {}/{} Tile coords: {}/{}", world_pos.x, world_pos.y, tile_x, tile_y);

        let grid = q_mining_grid.single();
        if tile_x < 0 || tile_y < 0 || tile_x >= grid.width as i32 || tile_y >= grid.height as i32 {
            warn!("Click was outside the mining grid.");
            return;
        }

        let idx = xy_to_idx(tile_x as usize, tile_y as usize, grid.width);
        match grid.rock_tiles.get(idx) {
            Some(rock_tile) => {
                if let Some(e) = rock_tile {
                    ev_mine.send(MineAction(*e));
                }
            }
            None => {
                warn!("Index was out of bounds.");
                return;
            }
        }
    }
}

fn handle_mine_actions(
    mut ev_mine: EventReader<MineAction>,
    mut q_mining_tiles: Query<&mut MiningTile>,
    mut ev_treasure_check: EventWriter<CheckTreasure>,
) {
    for ev in ev_mine.read() {
        match q_mining_tiles.get_mut(ev.0) {
            Ok(mut hit) => {
                hit.hp = hit.hp.saturating_sub(1);
                ev_treasure_check.send(CheckTreasure {});
                debug!("Tile was hit");
            }
            Err(_) => {
                debug!("Entity did not have mining tile component.");
                return;
            }
        };
    }
}

fn update_mining_tile(mut q_mining_tiles: Query<(&mut Visibility, &mut TextureAtlasSprite, &MiningTile)>) {
    for (mut vis, mut atlas_idx, tile) in q_mining_tiles.iter_mut() {
        if tile.hp == 0 {
            *vis = Visibility::Hidden;
        } else {
            atlas_idx.index = tile.hp - 1;
        }
    }
}
