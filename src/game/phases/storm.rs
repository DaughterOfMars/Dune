use bevy::prelude::*;

pub struct StormPlugin;

impl Plugin for StormPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StormPhase {
    Reveal,
    WeatherControl,
    FamilyAtomics,
    MoveStorm,
}
