use bevy::{
    log::{error, info},
    prelude::*,
    sprite::{SpriteSheetBundle, TextureAtlasSprite},
    utils::HashMap,
};
use rand::{seq::SliceRandom, Rng};

use crate::{
    assets::SpriteAssets,
    data_read::{TreasureInfo, TREASURE_DB},
    expedition::{ExpeditionPersist, ExpeditionStatus, InitExpedition},
    mining::{MiningGrid, MiningTile},
    point::{idx_to_xy, xy_to_idx, UPoint},
    AppState, SPRITE_PX_X, SPRITE_PX_Y,
};

pub struct TreasurePlugin;

impl Plugin for TreasurePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(OnEnter(AppState::Expedition), init_treasures)
            .add_systems(
                Update,
                (check_treasure_uncovered,).run_if(in_state(AppState::Expedition)).run_if(on_event::<CheckTreasure>()),
            )
            .add_event::<CheckTreasure>();
    }
}

#[derive(Component)]
pub struct Treasure {
    pub id: u32,           // id into the list of all the treasures
    pub parts: Vec<usize>, // a list of idx for where the item is
    pub is_discovered: bool,
}

#[derive(Component)]
pub struct TreasureTile;

// TODO: use these unused structs or get rid of them
#[derive(Resource)]
#[allow(unused)]
pub struct TreasureTrove {
    treasures: HashMap<u32, TreasureData>,
}

#[allow(unused)]
pub struct TreasureData {
    times_collected: u32,
}

#[derive(Event)]
pub struct CheckTreasure {}

pub struct TreasureGrid {
    pub treasures: Vec<Option<Entity>>,
    pub width: usize,
    pub height: usize,
}

impl TreasureGrid {
    fn new(width: usize, height: usize) -> Self {
        Self { treasures: vec![None; width * height], width, height }
    }
}

const TREASURE_Z: f32 = 20.0;

fn init_treasures(mut commands: Commands, mut ev_init: EventReader<InitExpedition>, sprites: Res<SpriteAssets>) {
    let Some(grid) = ev_init.read().next() else {
        info!("entering expedition state, no event to create treasure grid");
        return;
    };
    info!("init treasures");
    let mut grid = TreasureGrid::new(grid.size_x, grid.size_y);
    let mut rng = rand::thread_rng();

    // add treasures
    let Some(tdb) = TREASURE_DB.get() else {
        error!("could not find treasure database");
        return;
    };

    let mut total_treasures = 0;
    while total_treasures < 1 {
        // pick position
        let left_x = rng.gen_range(0..grid.width);
        let bottom_y = rng.gen_range(0..grid.height);

        // check it does not overlap any other treasure
        let Some(treasure_def) = &tdb.choose(&mut rng) else {
            return;
        };

        if !does_treasure_fit(&grid, treasure_def, UPoint { x: left_x, y: bottom_y }) {
            continue;
        }
        total_treasures += 1;

        // create treasure entity and sprite parts
        let mut treasure_parts = vec![];
        for (idx, tile) in treasure_def.shape.iter().enumerate() {
            if tile == &-1 {
                // ignore tiles that are -1 since that means the treasure does not occupy that spot
                continue;
            }

            let treasure_pos = idx_to_xy(idx, treasure_def.width);
            let new_pos = UPoint { x: treasure_pos.x + left_x, y: bottom_y - treasure_pos.y };
            let new_idx = xy_to_idx(new_pos.x, new_pos.y, grid.width);

            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: sprites.treasures.clone(),
                    // sprite: TextureAtlasSprite { color: Color::rgba(1.0, 0.0, 0.0, 1.0), index: *tile as usize, ..Default::default() },
                    sprite: TextureAtlasSprite::new(*tile as usize),
                    transform: Transform::from_xyz(
                        (new_pos.x * SPRITE_PX_X as usize) as f32,
                        (new_pos.y * SPRITE_PX_Y as usize) as f32,
                        TREASURE_Z,
                    ),
                    ..Default::default()
                },
                TreasureTile,
                ExpeditionPersist,
            ));

            treasure_parts.push(new_idx);
            info!("{:?} contains treasure", new_pos);
        }

        let parent = commands
            .spawn((Treasure { id: 0, parts: treasure_parts.clone(), is_discovered: false }, ExpeditionPersist));
        for treasure_idx in treasure_parts.iter() {
            grid.treasures[*treasure_idx] = Some(parent.id());
        }
    }
}

fn check_treasure_uncovered(
    mut expedition_status: ResMut<ExpeditionStatus>,
    mut q_treasures: Query<&mut Treasure>,
    q_mining_grid: Query<&MiningGrid>,
    q_mining_tiles: Query<&MiningTile>,
) {
    if matches!(*expedition_status, ExpeditionStatus::Cleared) {
        return;
    }
    let active_grid = q_mining_grid.single();

    let mut discovered_amt = vec![];
    for mut treasure in q_treasures.iter_mut() {
        if treasure.is_discovered {
            discovered_amt.push(true);
            continue;
        }

        let mut is_uncovered = true;
        for part in treasure.parts.iter() {
            let tile = q_mining_tiles.get(active_grid.rock_tiles[*part].unwrap()).unwrap();
            if tile.hp != 0 {
                is_uncovered = false;
            }
        }

        if is_uncovered {
            info!("Treasure was discovered.");
            discovered_amt.push(true);
        } else {
            discovered_amt.push(false);
        }
        treasure.is_discovered = is_uncovered;
    }

    if !discovered_amt.contains(&false) {
        info!("All treasures were discovered");
        *expedition_status = ExpeditionStatus::Cleared;
    }
}

fn does_treasure_fit(existing: &TreasureGrid, treasure: &TreasureInfo, start: UPoint) -> bool {
    for (idx, tile) in treasure.shape.iter().enumerate() {
        if tile == &-1 {
            // ignore tiles that are -1 since that means the treasure does not occupy that spot
            continue;
        }

        let treasure_grid_offset = idx_to_xy(idx, treasure.width);
        if treasure_grid_offset.y > start.y {
            debug!("Offset {:?} is too close to a border.", treasure_grid_offset);
            return false;
        }

        let new_pos = UPoint { x: treasure_grid_offset.x + start.x, y: start.y - treasure_grid_offset.y };
        if new_pos.x >= existing.width || new_pos.y >= existing.height {
            debug!("Offset {:?} is too close to a border.", treasure_grid_offset);
            return false;
        }

        let new_idx = xy_to_idx(new_pos.x, new_pos.y, existing.width);
        match existing.treasures.get(new_idx) {
            Some(maybe_treasure) if maybe_treasure.is_some() => {
                debug!("Position {:?} has treasure already in it", new_pos);
                return false;
            }
            Some(_empty) => {
                continue;
            }
            None => {
                debug!("Position {:?} is out of bounds of the treasure grid.", new_pos);
                return false;
            }
        }
    }

    true
}
