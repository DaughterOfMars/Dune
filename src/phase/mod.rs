pub(crate) mod actions;
mod stage;
mod systems;

use actions::*;
use stage::*;
use systems::*;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    f32::consts::PI,
    hash::Hash,
    mem::Discriminant,
};

use bevy::{
    ecs::Stage,
    prelude::*,
    render::camera::{Camera, OrthographicProjection},
};
use ncollide3d::{
    na::Vector3,
    shape::{ConvexHull, Cuboid, Cylinder, ShapeHandle},
    transformation::ToTriMesh,
};

use crate::{
    components::{
        Collider, ColliderBundle, Disorganized, LocationSector, Player, Prediction, Spice, Storm,
        Troop, Unique, UniqueBundle,
    },
    data::{
        Faction, FactionPredictionCard, Leader, StormCard, TraitorCard, TreacheryCard,
        TurnPredictionCard,
    },
    lerper::{Lerp, LerpType},
    resources::{Data, Info},
    util::{divide_spice, hand_positions, shuffle_deck},
    Screen, ScreenEntity, RESPONSE_STAGE, STATE_CHANGE_STAGE,
};

#[macro_export]
macro_rules! multi {
    ($($e:expr),+ $(,)?) => {
        ActionAggregation::Multiple(vec![$($e),+])
    };
}

#[macro_export]
macro_rules! single {
    ($e:expr) => {
        ActionAggregation::Single($e)
    };
}

const PHASE_STAGE: &str = "phase";

pub struct PhasePlugin;

impl Plugin for PhasePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ActionQueue>()
            .init_resource::<GamePhase>()
            .on_state_update(STATE_CHANGE_STAGE, Screen::HostingGame, action.system())
            .on_state_update(STATE_CHANGE_STAGE, Screen::HostingGame, phase_text.system())
            .on_state_update(
                STATE_CHANGE_STAGE,
                Screen::HostingGame,
                public_troop.system(),
            )
            .on_state_update(
                STATE_CHANGE_STAGE,
                Screen::HostingGame,
                stack_troops.system(),
            )
            .on_state_exit(RESPONSE_STAGE, Screen::HostingGame, reset.system());

        // Phase stages
        app.add_stage(PHASE_STAGE, PhaseStage::<Screen>::default())
            .stage(PHASE_STAGE, |stage: &mut PhaseStage<Screen>| {
                stage
                    .valid_states(vec![Screen::HostingGame, Screen::JoinedGame])
                    .on_phase_enter(
                        Phase::Setup(SetupSubPhase::ChooseFactions),
                        shuffle_decks.system(),
                    )
                    .on_phase_enter(
                        Phase::Setup(SetupSubPhase::ChooseFactions),
                        pick_factions.system(),
                    )
                    .on_phase_exit(
                        Phase::Setup(SetupSubPhase::ChooseFactions),
                        init_factions.system(),
                    )
                    .on_phase_enter(
                        Phase::Setup(SetupSubPhase::AtStart),
                        get_initial_spice.system(),
                    )
                    .on_phase_enter(Phase::Setup(SetupSubPhase::AtStart), place_troops.system())
                    .on_phase_enter(
                        Phase::Setup(SetupSubPhase::Prediction),
                        animate_prediction_cards.system(),
                    )
                    .on_phase_enter(
                        Phase::Setup(SetupSubPhase::DealTraitors),
                        deal_traitor_cards.system(),
                    )
            });
    }
}

pub struct PhaseText;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Phase {
    Setup(SetupSubPhase),
    Storm(StormSubPhase),
    SpiceBlow,
    Nexus,
    Bidding,
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
                SetupSubPhase::ChooseFactions => Phase::Setup(SetupSubPhase::Prediction),
                SetupSubPhase::Prediction => Phase::Setup(SetupSubPhase::AtStart),
                SetupSubPhase::AtStart => Phase::Setup(SetupSubPhase::DealTraitors),
                SetupSubPhase::DealTraitors => Phase::Setup(SetupSubPhase::PickTraitors),
                SetupSubPhase::PickTraitors => Phase::Setup(SetupSubPhase::DealTreachery),
                SetupSubPhase::DealTreachery => Phase::Storm(StormSubPhase::Reveal),
            },
            Phase::Storm(subphase) => match subphase {
                StormSubPhase::Reveal => Phase::Storm(StormSubPhase::WeatherControl),
                StormSubPhase::WeatherControl => Phase::Storm(StormSubPhase::FamilyAtomics),
                StormSubPhase::FamilyAtomics => Phase::Storm(StormSubPhase::MoveStorm),
                StormSubPhase::MoveStorm => Phase::SpiceBlow,
            },
            Phase::SpiceBlow => Phase::Nexus,
            Phase::Nexus => Phase::Bidding,
            Phase::Bidding => Phase::Revival,
            Phase::Revival => Phase::Movement,
            Phase::Movement => Phase::Battle,
            Phase::Battle => Phase::Collection,
            Phase::Collection => Phase::Control,
            Phase::Control => Phase::Storm(StormSubPhase::Reveal),
            Phase::EndGame => Phase::EndGame,
        }
    }
}

impl Default for Phase {
    fn default() -> Self {
        Phase::Setup(SetupSubPhase::ChooseFactions)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SetupSubPhase {
    ChooseFactions,
    Prediction,
    AtStart,
    DealTraitors,
    PickTraitors,
    DealTreachery,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StormSubPhase {
    Reveal,
    WeatherControl,
    FamilyAtomics,
    MoveStorm,
}

#[derive(Clone, Debug, Default)]
pub struct GamePhase {
    pub prev: Option<Phase>,
    pub curr: Phase,
    pub next: VecDeque<Phase>,
    pub handled: bool,
}

impl GamePhase {
    pub fn push_next(&mut self, phase: Phase) {
        self.next.push_back(phase);
    }

    pub fn advance(&mut self) {
        let next = if let Some(latest) = self.next.back() {
            latest.next()
        } else {
            self.curr.next()
        };
        self.push_next(next);
    }

    pub fn apply(&mut self) {
        if let Some(next) = self.next.pop_front() {
            let previous = std::mem::replace(&mut self.curr, next);
            if previous != self.curr {
                self.prev = Some(previous)
            }
        }
    }
}

fn reset(mut phase: ResMut<GamePhase>, mut queue: ResMut<ActionQueue>) {
    *phase = GamePhase::default();
    queue.clear();
}
