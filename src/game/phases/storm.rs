use bevy::prelude::*;

pub struct StormPlugin;

impl Plugin for StormPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(State::<StormPhase>::get_driver());
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StormPhase {
    Reveal,
    WeatherControl,
    FamilyAtomics,
    MoveStorm,
}
