use bevy::prelude::*;

use self::{choose_factions::ChooseFactionsPlugin, prediction::PredictionPlugin, at_start::AtStartPlugin};

mod at_start;
mod choose_factions;
mod prediction;

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ChooseFactionsPlugin)
            .add_plugin(PredictionPlugin)
            .add_plugin(AtStartPlugin);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SetupPhase {
    ChooseFactions,
    Prediction,
    AtStart,
}
