use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl};
use rand::{seq::SliceRandom, thread_rng};

use crate::{assets::SoundAssets, mining::MineAction, AppState};

pub struct AudioEventsPlugin;

impl Plugin for AudioEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, play_mining_sound.run_if(in_state(AppState::Expedition)));
    }
}

fn play_mining_sound(mut mine_actions: EventReader<MineAction>, sounds: Res<SoundAssets>, audio: Res<Audio>) {
    // TODO: in the future we should play the sound based on the type of tile hit
    let rock_sounds = vec![
        sounds.mine_rock1.clone(),
        sounds.mine_rock2.clone(),
        sounds.mine_rock3.clone(),
        sounds.mine_rock4.clone(),
        sounds.mine_rock5.clone(),
    ];
    let mut rng = thread_rng();
    for _ev in mine_actions.read() {
        let rock_sound = rock_sounds.choose(&mut rng).unwrap();
        audio.play(rock_sound.clone()).with_volume(1.0);
        info!("played mining sound")
    }
}
