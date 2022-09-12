use bevy::prelude::*;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::network::GameEvents;

pub struct StormPlugin;

impl Plugin for StormPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Display, Serialize, Deserialize)]
pub enum StormPhase {
    Reveal,
    WeatherControl,
    FamilyAtomics,
    MoveStorm,
}

fn reveal(game_events: Res<GameEvents>) {}
