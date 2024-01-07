use bevy::{prelude::*, window::PrimaryWindow};

use rand::Rng;

use crate::{
    assets::SpriteAssets, AppState, MainCamera, GRID_HEIGHT, GRID_WIDTH, SPRITE_PX_X, SPRITE_PX_Y,
};

pub struct MiningPlugin;

impl Plugin for MiningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Expedition), init_mining_grid)
            .add_systems(
                Update,
                (player_mouse_mine, handle_mine_actions, update_mining_tile)
                    .run_if(in_state(AppState::Expedition)),
            )
            .add_event::<MineAction>()
            .add_event::<InitMiningGrid>();
    }
}

#[derive(Component)]
struct MiningGrid {
    pub tiles: Vec<Entity>,
    pub width: usize,
    pub height: usize,
}

#[derive(Component)]
struct MiningTile {
    hp: usize,
}

impl MiningTile {
    fn new(hp: usize) -> Self {
        Self { hp }
    }
}

#[derive(Event)]
struct MineAction(pub Entity);

#[derive(Event, Debug)]
pub struct InitMiningGrid {
    pub size_x: usize,
    pub size_y: usize,
}

fn init_mining_grid(
    mut commands: Commands,
    mut ev_init: EventReader<InitMiningGrid>,
    sprites: Res<SpriteAssets>,
) {
    let Some(grid) = ev_init.read().next() else {
        warn!("no event to create mining grid with");
        return;
    };
    info!("running init mining grid {:?}", grid);

    let mut tiles = Vec::new();
    let mut rng = rand::thread_rng();

    for y in 0..grid.size_y {
        for x in 0..grid.size_x {
            let x = (x * SPRITE_PX_X) as f32;
            let y = (y * SPRITE_PX_Y) as f32;
            let hp: usize = rng.gen::<usize>() % 4;
            let tile = commands.spawn((
                MiningTile::new(hp + 1),
                SpriteSheetBundle {
                    texture_atlas: sprites.mining_rocks.clone(),
                    sprite: TextureAtlasSprite::new(hp),
                    transform: Transform::from_xyz(x, y, 0.0),
                    ..default()
                },
            ));
            tiles.push(tile.id());
        }
    }
    commands.spawn(MiningGrid {
        height: GRID_HEIGHT,
        width: GRID_WIDTH,
        tiles,
    });
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
        println!(
            "World coords: {}/{} Tile coords: {}/{}",
            world_pos.x, world_pos.y, tile_x, tile_y
        );

        let grid = q_mining_grid.single();
        if tile_x < 0 || tile_y < 0 || tile_x >= grid.width as i32 || tile_y >= grid.height as i32 {
            warn!("Click was outside the mining grid.");
            return;
        }

        let idx = xy_to_idx(tile_x as usize, tile_y as usize, grid.width);
        let tile_e = match grid.tiles.get(idx) {
            Some(e) => e,
            None => {
                warn!("Index was out of bounds.");
                return;
            }
        };
        ev_mine.send(MineAction(*tile_e));
    }
}

fn handle_mine_actions(
    mut ev_mine: EventReader<MineAction>,
    mut q_mining_tiles: Query<&mut MiningTile>,
) {
    for ev in ev_mine.read() {
        match q_mining_tiles.get_mut(ev.0) {
            Ok(mut hit) => {
                hit.hp = hit.hp.saturating_sub(1);
                debug!("Tile was hit");
            }
            Err(_) => {
                debug!("Entity did not have mining tile component.");
                return;
            }
        };
    }
}

fn update_mining_tile(
    mut q_mining_tiles: Query<(
        Entity,
        &mut Visibility,
        &mut TextureAtlasSprite,
        &MiningTile,
    )>,
) {
    for (_e, mut vis, mut atlas_idx, tile) in q_mining_tiles.iter_mut() {
        if tile.hp == 0 {
            *vis = Visibility::Hidden;
        } else {
            atlas_idx.index = tile.hp - 1;
        }
    }
}

fn xy_to_idx(x: usize, y: usize, width: usize) -> usize {
    x + y * width
}
