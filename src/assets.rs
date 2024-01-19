use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{expedition::Area, AppState};

pub struct AssetLoadPlugin;

impl Plugin for AssetLoadPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_loading_state(
            LoadingState::new(AppState::AssetLoading)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("full_dynamic_collection.assets.ron")
                .load_collection::<SpriteAssets>()
                .load_collection::<UiAssets>()
                .continue_to_state(AppState::AreaViewer { curr_area: Area::TheCaves }),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct SpriteAssets {
    #[asset(key = "level_1_xped")]
    pub lvl1xped: Handle<TextureAtlas>,
    #[asset(key = "treasures")]
    pub treasures: Handle<TextureAtlas>,
    #[asset(key = "tools")]
    pub tools: Handle<TextureAtlas>,
}

#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(key = "tool_shadow")]
    pub tool_shadow: Handle<Image>,
    #[asset(key = "level_1fg")]
    pub level1fg: Handle<Image>,
    #[asset(key = "red_button")]
    pub leave_button: Handle<Image>,
    #[asset(key = "clear_menu")]
    pub clear_menu: Handle<Image>,

    #[asset(key = "kaph")]
    pub text: Handle<Font>,
}
