use bevy::prelude::*;
use bevy_mod_picking::PickingEvent;
use iyes_loopless::prelude::IntoConditionalSystem;

use self::{choose_factions::ChooseFactionsPlugin, prediction::PredictionPlugin};
use crate::{
    components::{Faction, FactionPredictionCard, TurnPredictionCard},
    lerper::Lerp,
    Active, Screen,
};

mod choose_factions;
mod prediction;

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FactionPickedEvent>()
            .add_event::<TurnPickedEvent>()
            .add_plugin(ChooseFactionsPlugin)
            .add_plugin(PredictionPlugin)
            .add_system(faction_card_picker.run_in_state(Screen::Game))
            .add_system(turn_card_picker.run_in_state(Screen::Game));
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SetupPhase {
    ChooseFactions,
    Prediction,
    AtStart,
    DealTraitors,
    PickTraitors,
    DealTreachery,
}

pub struct FactionPickedEvent {
    entity: Entity,
    faction: Faction,
}

fn faction_card_picker(
    active: Res<Active>,
    faction_cards: Query<(&FactionPredictionCard, Option<&Lerp>)>,
    parents: Query<&Parent>,
    mut picking_events: EventReader<PickingEvent>,
    mut picked_events: EventWriter<FactionPickedEvent>,
) {
    if !faction_cards.is_empty() {
        for event in picking_events.iter() {
            if let PickingEvent::Clicked(clicked) = event {
                let mut clicked = *clicked;
                loop {
                    if let Ok((faction_card, lerp)) = faction_cards.get(clicked) {
                        if lerp.is_none() {
                            picked_events.send(FactionPickedEvent {
                                entity: active.entity,
                                faction: faction_card.faction,
                            });
                            return;
                        } else {
                            break;
                        }
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

pub struct TurnPickedEvent {
    entity: Entity,
    turn: u8,
}

fn turn_card_picker(
    active: Res<Active>,
    turn_cards: Query<(&TurnPredictionCard, Option<&Lerp>)>,
    parents: Query<&Parent>,
    mut picking_events: EventReader<PickingEvent>,
    mut picked_events: EventWriter<TurnPickedEvent>,
) {
    if !turn_cards.is_empty() {
        for event in picking_events.iter() {
            if let PickingEvent::Clicked(clicked) = event {
                let mut clicked = *clicked;
                loop {
                    if let Ok((turn_card, lerp)) = turn_cards.get(clicked) {
                        if lerp.is_none() {
                            picked_events.send(TurnPickedEvent {
                                entity: active.entity,
                                turn: turn_card.turn,
                            });
                            return;
                        } else {
                            break;
                        }
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
