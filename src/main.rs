use bevy::{prelude::*, window::PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, init_mining_grid)
        .add_systems(Startup, init_camera)
        .add_systems(Update, player_mouse_mine)
        .add_systems(Update, update_mining_tile)
        .run();
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

const SPRITE_PX_X: usize = 64;
const SPRITE_PX_Y: usize = SPRITE_PX_X;

const GRID_WIDTH: usize = 10;
const GRID_HEIGHT: usize = 10;

#[derive(Component)]
struct MainCamera;

fn init_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(
                GRID_WIDTH as f32 * SPRITE_PX_X as f32 / 2.0,
                GRID_HEIGHT as f32 * SPRITE_PX_Y as f32 / 2.0,
                10.0,
            ),
            ..default()
        },
        MainCamera,
    ));
}

fn init_mining_grid(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut tiles = Vec::new();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let x = (x * SPRITE_PX_X) as f32;
            let y = (y * SPRITE_PX_Y) as f32;
            let tile = commands.spawn((
                MiningTile::new(1),
                SpriteBundle {
                    texture: asset_server.load("test_rock.png"),
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
    mut q_mining_tiles: Query<&mut MiningTile>,
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
        match q_mining_tiles.get_mut(*tile_e) {
            Ok(mut hit) => {
                hit.hp = hit.hp.saturating_sub(1);
                debug!("Tile was hit");
            }
            Err(_) => {
                debug!("Resulted in No hit");
                return;
            }
        };
    }
}

fn update_mining_tile(mut q_mining_tiles: Query<(Entity, &mut Visibility, &MiningTile)>) {
    for (_e, mut vis, tile) in q_mining_tiles.iter_mut() {
        if tile.hp == 0 {
            *vis = Visibility::Hidden;
        }
    }
}

fn xy_to_idx(x: usize, y: usize, width: usize) -> usize {
    x + y * width
}
