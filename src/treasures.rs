use bevy::log::{error, info};
use bevy::prelude::{
    debug, in_state, Commands, Component, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, OnEnter, Plugin,
    Query, Res, Transform, Update,
};
use bevy::sprite::{SpriteSheetBundle, TextureAtlasSprite};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::assets::SpriteAssets;
use crate::data_read::{TreasureInfo, TREASURE_DB};
use crate::expedition::{ExpeditionPersist, InitExpedition};
use crate::mining::{MiningGrid, MiningTile};
use crate::point::{idx_to_xy, xy_to_idx, Point};
use crate::{AppState, SPRITE_PX_X, SPRITE_PX_Y};

pub struct TreasurePlugin;

impl Plugin for TreasurePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(OnEnter(AppState::Expedition), init_treasures)
            .add_systems(Update, (check_treasure_uncovered,).run_if(in_state(AppState::Expedition)))
            .add_event::<CheckTreasure>()
            .add_event::<TreasureDiscover>();
    }
}

#[derive(Component)]
pub struct Treasure {
    pub _id: u32,          // id into the list of all the treasures
    pub parts: Vec<usize>, // a list of idx for where the item is
    pub is_discovered: bool,
}

#[derive(Component)]
pub struct TreasureTile;

#[derive(Event)]
pub struct CheckTreasure {}

#[derive(Event)]
struct TreasureDiscover {
    pub _what: Entity,
}

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
    while total_treasures < 5 {
        // pick position
        let left_x = rng.gen_range(0..grid.width);
        let bottom_y = rng.gen_range(0..grid.height);

        // check it does not overlap any other treasure
        let Some(treasure_def) = &tdb.choose(&mut rng) else {
            return;
        };

        if !does_treasure_fit(&grid, treasure_def, Point { x: left_x, y: bottom_y }) {
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
            let new_pos = Point { x: treasure_pos.x + left_x, y: bottom_y - treasure_pos.y };
            let new_idx = xy_to_idx(new_pos.x, new_pos.y, grid.width);

            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: sprites.treasures.clone(),
                    sprite: TextureAtlasSprite::new(*tile as usize),
                    transform: Transform::from_xyz(
                        (new_pos.x * SPRITE_PX_X) as f32,
                        (new_pos.y * SPRITE_PX_Y) as f32,
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
            .spawn((Treasure { _id: 0, parts: treasure_parts.clone(), is_discovered: false }, ExpeditionPersist));
        for treasure_idx in treasure_parts.iter() {
            grid.treasures[*treasure_idx] = Some(parent.id());
        }
    }
}

fn check_treasure_uncovered(
    mut ev_treasure_check: EventReader<CheckTreasure>,
    mut ev_treasure_discover: EventWriter<TreasureDiscover>,
    mut q_treasures: Query<(Entity, &mut Treasure)>,
    q_mining_grid: Query<&MiningGrid>,
    q_mining_tiles: Query<&MiningTile>,
) {
    let active_grid = q_mining_grid.single();
    for _ in ev_treasure_check.read() {
        for (treasure_entity, mut treasure) in q_treasures.iter_mut() {
            if treasure.is_discovered {
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
                ev_treasure_discover.send(TreasureDiscover { _what: treasure_entity });
                info!("Treasure was discovered.");
            }
            treasure.is_discovered = is_uncovered;
        }
    }
}

pub fn does_treasure_fit(existing: &TreasureGrid, treasure: &TreasureInfo, start: Point) -> bool {
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

        let new_pos = Point { x: treasure_grid_offset.x + start.x, y: start.y - treasure_grid_offset.y };
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
