use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{level_change::Area, AppState};

pub struct AssetLoadPlugin;

impl Plugin for AssetLoadPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_loading_state(
            LoadingState::new(AppState::AssetLoading)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "full_dynamic_collection.assets.ron",
                )
                .load_collection::<SpriteAssets>()
                .continue_to_state(AppState::AreaViewer {
                    curr_area: Area::TheCaves,
                }),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct SpriteAssets {
    #[asset(key = "terrain_rocks")]
    pub mining_rocks: Handle<TextureAtlas>,
}
