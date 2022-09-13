pub mod bidding;
pub mod setup;
pub mod spice_blow;
pub mod storm;

use bevy::prelude::*;
use derive_more::Display;
use iyes_loopless::prelude::AppLooplessStateExt;
use serde::{Deserialize, Serialize};

use self::{
    bidding::{BiddingPhase, BiddingPlugin},
    setup::*,
    spice_blow::{SpiceBlowPhase, SpiceBlowPlugin},
    storm::*,
};
use super::{
    state::{GameEvent, GameState},
    GameEventStage,
};
use crate::{network::GameEvents, Screen};

pub struct PhasePlugin;

impl Plugin for PhasePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SetupPlugin)
            .add_plugin(StormPlugin)
            .add_plugin(SpiceBlowPlugin)
            .add_plugin(BiddingPlugin);

        app.add_enter_system(Screen::Game, init_phase_text)
            .add_system_to_stage(GameEventStage, phase_text);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Display, Serialize, Deserialize)]
pub enum Phase {
    Setup(SetupPhase),
    Storm(StormPhase),
    SpiceBlow(SpiceBlowPhase),
    Nexus,
    Bidding(BiddingPhase),
    Revival,
    Movement,
    Battle,
    Collection,
    Control,
    EndGame,
}

impl Phase {
    pub fn next(&self) -> Self {
        match self {
            Phase::Setup(subphase) => match subphase {
                SetupPhase::ChooseFactions => Phase::Setup(SetupPhase::Prediction),
                SetupPhase::Prediction => Phase::Setup(SetupPhase::AtStart),
                SetupPhase::AtStart => Phase::Setup(SetupPhase::DealTraitors),
                SetupPhase::DealTraitors => Phase::Setup(SetupPhase::PlaceForces),
                SetupPhase::PlaceForces => Phase::Setup(SetupPhase::DealTreachery),
                SetupPhase::DealTreachery => Phase::Storm(StormPhase::Reveal),
            },
            Phase::Storm(subphase) => match subphase {
                StormPhase::Reveal => Phase::Storm(StormPhase::WeatherControl),
                StormPhase::WeatherControl => Phase::Storm(StormPhase::FamilyAtomics),
                StormPhase::FamilyAtomics => Phase::Storm(StormPhase::MoveStorm),
                StormPhase::MoveStorm => Phase::SpiceBlow(SpiceBlowPhase::Reveal),
            },
            Phase::SpiceBlow(subphase) => match subphase {
                SpiceBlowPhase::Reveal => Phase::SpiceBlow(SpiceBlowPhase::ShaiHalud),
                SpiceBlowPhase::ShaiHalud => Phase::SpiceBlow(SpiceBlowPhase::PlaceSpice),
                SpiceBlowPhase::PlaceSpice => Phase::Nexus,
            },
            Phase::Nexus => Phase::Bidding(BiddingPhase::DealCards),
            Phase::Bidding(subphase) => match subphase {
                BiddingPhase::DealCards => Phase::Bidding(BiddingPhase::Bidding),
                BiddingPhase::Bidding => Phase::Revival,
            },
            Phase::Revival => Phase::Movement,
            Phase::Movement => Phase::Battle,
            Phase::Battle => Phase::Collection,
            Phase::Collection => Phase::Control,
            Phase::Control => Phase::Storm(StormPhase::Reveal),
            Phase::EndGame => Phase::Setup(SetupPhase::ChooseFactions),
        }
    }
}

impl Default for Phase {
    fn default() -> Self {
        Phase::EndGame
    }
}

#[derive(Component)]
struct PhaseText;

fn init_phase_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                },
                ..default()
            },
            text: Text::from_section(
                "Test",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            ..default()
        })
        .insert(PhaseText);
}

fn phase_text(game_events: Res<GameEvents>, game_state: Res<GameState>, mut text: Query<&mut Text, With<PhaseText>>) {
    if let Some(GameEvent::AdvancePhase) = game_events.peek() {
        let s = match game_state.phase {
            Phase::Setup(subphase) => match subphase {
                SetupPhase::ChooseFactions => "Choosing Factions...".to_string(),
                SetupPhase::Prediction => "Bene Gesserit are making a prediction...".to_string(),
                SetupPhase::AtStart => "Start of Game Setup...".to_string(),
                SetupPhase::DealTraitors => "Picking Traitor Cards...".to_string(),
                SetupPhase::PlaceForces => "Placing Forces...".to_string(),
                SetupPhase::DealTreachery => "Dealing Treachery Cards...".to_string(),
            },
            Phase::Storm(_) => "Storm Phase".to_string(),
            Phase::SpiceBlow(_) => "Spice Blow Phase".to_string(),
            Phase::Nexus => "Nexus Phase".to_string(),
            Phase::Bidding(_) => "Bidding Phase".to_string(),
            Phase::Revival => "Revival Phase".to_string(),
            Phase::Movement => "Movement Phase".to_string(),
            Phase::Battle => "Battle Phase".to_string(),
            Phase::Collection => "Collection Phase".to_string(),
            Phase::Control => "Control Phase".to_string(),
            Phase::EndGame => "".to_string(),
        };

        text.single_mut().sections[0].value = s;
    }
}
