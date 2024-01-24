use bevy::{prelude::*, utils::HashSet, window::PrimaryWindow};
use rand::Rng;

use crate::{
    assets::SpriteAssets,
    camera::MainCamera,
    consts::{
        A_BORDER_BOTTOM, A_BORDER_LEFT, A_BORDER_RIGHT, A_BORDER_TOP, A_CORNER_BL, A_CORNER_BR, A_CORNER_TL,
        A_CORNER_TR, A_DARK_GROUND,
    },
    expedition::{ExpeditionPersist, ExpeditionStatus, InitExpedition},
    point::{xy_to_idx, UPoint},
    stability::StabilityDamage,
    tools::{ActiveTool, PickaxeRotation, ToolType},
    treasures::CheckTreasure,
    AppState, SPRITE_PX_X, SPRITE_PX_Y,
};

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
struct TileHit {
    tile: Entity,
    damage: usize,
}

#[derive(Event)]
pub struct MineAction {
    tile_x: u32,
    tile_y: u32,
}

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
            let x = (x * SPRITE_PX_X as usize) as f32;
            let y = (y * SPRITE_PX_Y as usize) as f32;
            let hp: usize = rng.gen::<usize>() % 4;
            let tile = commands.spawn((
                MiningTile::new(hp + 1),
                SpriteSheetBundle {
                    texture_atlas: sprites.lvl1xped.clone(),
                    sprite: TextureAtlasSprite::new(hp),
                    transform: Transform::from_xyz(x, y, BREAKABLE_Z),
                    ..default()
                },
                ExpeditionPersist,
            ));
            grid.rock_tiles[tile_idx] = Some(tile.id());

            let _bg = commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: sprites.lvl1xped.clone(),
                    sprite: TextureAtlasSprite::new(5),
                    transform: Transform::from_xyz(x, y, BACKGROUND_Z),
                    ..default()
                },
                ExpeditionPersist,
            ));
        }
    }

    commands.spawn((grid, ExpeditionPersist));
    info!("created mining grid");

    for y in (-50)..50 {
        for x in (-50)..50 {
            if (0..new_grid.size_x as i32).contains(&x) && (0..new_grid.size_y as i32).contains(&y) {
                continue;
            }
            let atlas_idx = get_border_atlas_idx(x, y, (new_grid.size_x as i32, new_grid.size_y as i32));

            let x = (x * SPRITE_PX_X as i32) as f32;
            let y = (y * SPRITE_PX_Y as i32) as f32;
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: sprites.lvl1xped.clone(),
                    sprite: TextureAtlasSprite::new(atlas_idx),
                    transform: Transform::from_xyz(x, y, BACKGROUND_Z),
                    ..default()
                },
                ExpeditionPersist,
            ));
        }
    }
}

/// Mouse Input for player to touch the mining tiles
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
        // Click needs to be within grid of mineable rocks to even be considered a mining action
        if tile_x < 0 || tile_y < 0 || tile_x >= grid.width as i32 || tile_y >= grid.height as i32 {
            warn!("Click was outside the mining grid.");
            return;
        }

        ev_mine.send(MineAction { tile_x: tile_x as u32, tile_y: tile_y as u32 });
    }
}

fn handle_mine_actions(
    mut ev_mine: EventReader<MineAction>,
    mut q_mining_tiles: Query<&mut MiningTile>,
    mut ev_stability: EventWriter<StabilityDamage>,
    mut ev_treasure_check: EventWriter<CheckTreasure>,
    q_mining_grid: Query<&MiningGrid>,
    tool: Res<ActiveTool>,
    expedition_status: Res<ExpeditionStatus>,
) {
    match *expedition_status {
        ExpeditionStatus::Cleared | ExpeditionStatus::Leaving => {
            return; // cant mine after the expedition is finished
        }
        ExpeditionStatus::Mining => {}
    }

    let grid = q_mining_grid.single();
    for ev in ev_mine.read() {
        let start = UPoint::new(ev.tile_x as usize, ev.tile_y as usize);
        let tiles_hit = get_tile_hits(&tool.0, &start, &grid);
        if tiles_hit.len() != 0 {
            ev_stability.send(StabilityDamage::new(get_hit_stability(&tool.0, &tiles_hit)));
        }
        for TileHit { tile, damage } in tiles_hit.iter() {
            match q_mining_tiles.get_mut(*tile) {
                Ok(mut hit) => {
                    hit.hp = hit.hp.saturating_sub(*damage);
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

/// Helper: to find which atlas index to use for creating the border around the mineable tiles
fn get_border_atlas_idx(x: i32, y: i32, grid: (i32, i32)) -> usize {
    if x < -1 || y < -1 || x > grid.0 || y > grid.1 {
        A_DARK_GROUND
    } else if x == -1 {
        if y == -1 {
            A_CORNER_BL
        } else if y == grid.1 {
            A_CORNER_TL
        } else {
            A_BORDER_LEFT
        }
    } else if x == grid.0 {
        if y == -1 {
            A_CORNER_BR
        } else if y == grid.1 {
            A_CORNER_TR
        } else {
            A_BORDER_RIGHT
        }
    } else if y == -1 {
        A_BORDER_BOTTOM
    } else if y == grid.1 {
        A_BORDER_TOP
    } else {
        A_DARK_GROUND
    }
}

fn get_hit_stability(tool: &ToolType, _hits: &[TileHit]) -> u32 {
    match tool {
        ToolType::TinyHammer => 75,
        ToolType::Pickaxe { rotation } => match rotation {
            PickaxeRotation::Horizontal => 25,
            PickaxeRotation::Vertical => 25,
            PickaxeRotation::Cross => 45,
        },
    }
}

// Helper: returns entity and how much damage dealt based on which tool is used
fn get_tile_hits(tool: &ToolType, start_pos: &UPoint, grid: &MiningGrid) -> Vec<TileHit> {
    match tool {
        ToolType::TinyHammer => {
            let start_idx = start_pos.as_idx(grid.width);
            if let Some(maybe_rock) = grid.rock_tiles.get(start_idx) {
                if let Some(rock) = maybe_rock {
                    return vec![(TileHit { tile: *rock, damage: 1 })];
                }
            }
            return vec![];
        }
        ToolType::Pickaxe { rotation } => {
            let mut hits = vec![];
            let hit_spots = match rotation {
                PickaxeRotation::Horizontal => {
                    vec![
                        *start_pos,
                        UPoint::new(start_pos.x.saturating_sub(1), start_pos.y),
                        UPoint::new(start_pos.x + 1, start_pos.y),
                    ]
                }
                PickaxeRotation::Vertical => {
                    vec![
                        *start_pos,
                        UPoint::new(start_pos.x, start_pos.y.saturating_sub(1)),
                        UPoint::new(start_pos.x, start_pos.y + 1),
                    ]
                }
                PickaxeRotation::Cross => {
                    vec![
                        *start_pos,
                        UPoint::new(start_pos.x, start_pos.y.saturating_sub(1)),
                        UPoint::new(start_pos.x, start_pos.y + 1),
                        UPoint::new(start_pos.x.saturating_sub(1), start_pos.y),
                        UPoint::new(start_pos.x + 1, start_pos.y),
                    ]
                }
            };
            let hit_spots = hit_spots
                .into_iter()
                .filter(|spot| spot.x < grid.width && spot.y < grid.height)
                .collect::<HashSet<_>>();

            for spot in hit_spots {
                let idx = spot.as_idx(grid.width);

                if let Some(maybe_rock) = grid.rock_tiles.get(idx) {
                    if let Some(rock) = maybe_rock {
                        hits.push(TileHit { tile: *rock, damage: 1 });
                    }
                }
            }
            return hits;
        }
    }
}
