mod object;
mod phases;
pub mod state;
mod systems;

use std::hash::Hash;

use bevy::prelude::*;
use bevy_mod_picking::PickingEvent;
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet};
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};

use self::systems::*;
pub use self::{
    object::*,
    phases::{
        setup::{SetupPhase, SetupPlugin},
        storm::StormPhase,
    },
};
use crate::{
    components::{Deck, FactionChoiceCard, FactionPredictionCard, LocationSector, TraitorCard, TurnPredictionCard},
    lerper::Lerp,
    util::card_jitter,
    Screen,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObjectEntityMap>();

        app.add_plugin(SetupPlugin);

        app.add_event::<PickedEvent<FactionChoiceCard>>()
            .add_event::<PickedEvent<FactionPredictionCard>>()
            .add_event::<PickedEvent<TurnPredictionCard>>()
            .add_event::<PickedEvent<TraitorCard>>()
            .add_event::<PickedEvent<LocationSector>>();

        app.add_system_set(
            ConditionSet::new()
                .run_in_state(Screen::Game)
                .with_system(spawn_object)
                .with_system(hiararchy_picker::<FactionChoiceCard>)
                .with_system(hiararchy_picker::<FactionPredictionCard>)
                .with_system(hiararchy_picker::<TurnPredictionCard>)
                .with_system(hiararchy_picker::<TraitorCard>)
                .with_system(hiararchy_picker::<LocationSector>)
                .with_system(phase_text)
                .with_system(active_player_text)
                .with_system(shuffle)
                .with_system(shuffle_traitors)
                .into(),
        );

        app.add_exit_system(Screen::Game, reset);
    }
}

#[derive(Component)]
pub struct PhaseText;

#[derive(Component)]
pub struct ActivePlayerText;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Phase {
    Setup(SetupPhase),
    Storm(StormPhase),
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
                StormPhase::MoveStorm => Phase::SpiceBlow,
            },
            Phase::SpiceBlow => Phase::Nexus,
            Phase::Nexus => Phase::Bidding,
            Phase::Bidding => Phase::Revival,
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

fn reset() {
    todo!()
}

#[derive(Component)]
pub struct Shuffling(pub usize);

pub fn shuffle(
    mut commands: Commands,
    mut decks: Query<(Entity, &Children, &mut Shuffling), With<Deck>>,
    lerps: Query<&Lerp>,
) {
    let mut rng = rand::thread_rng();
    for (e, children, mut shuffling) in decks.iter_mut() {
        if children.iter().any(|c| lerps.get(*c).is_ok()) {
            shuffling.0 -= 1;
            if shuffling.0 == 0 {
                commands.entity(e).remove::<Shuffling>();
            }
            continue;
        }
        let mut cards = children.iter().enumerate().collect::<Vec<_>>();
        cards.shuffle(&mut rng);
        for (i, card) in cards {
            let transform = Transform::from_translation(Vec3::Y * 0.001 * (i as f32)) * card_jitter();
            commands.entity(*card).insert(Lerp::world_to(transform, 0.2, 0.0));
        }
    }
}

pub struct PickedEvent<T> {
    pub picked: Entity,
    pub inner: T,
}

// Converts PickingEvents to typed PickedEvents by looking up the hierarchy if needed
fn hiararchy_picker<T: Component + Clone>(
    pickables: Query<&T>,
    parents: Query<&Parent>,
    mut picking_events: EventReader<PickingEvent>,
    mut picked_events: EventWriter<PickedEvent<T>>,
) {
    if !pickables.is_empty() {
        for event in picking_events.iter() {
            if let PickingEvent::Clicked(clicked) = event {
                let mut clicked = *clicked;
                loop {
                    if let Ok(card) = pickables.get(clicked) {
                        picked_events.send(PickedEvent {
                            picked: clicked,
                            inner: card.clone(),
                        });
                        return;
                    } else {
                        if let Ok(parent) = parents.get(clicked).map(|p| p.get()) {
                            clicked = parent;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }
}
