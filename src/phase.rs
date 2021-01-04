use std::collections::HashMap;

use crate::{
    seq, simul,
    stack::{Action, ActionStack, Context},
};
use bevy::prelude::*;
use rand::{prelude::SliceRandom, Rng};

use crate::{
    components::{LocationSector, Player, Storm, Unique},
    data::{Faction, FactionPredictionCard, Leader, StormCard, TreacheryCard},
    resources::{Data, Info},
    util::set_view_to_active_player,
};

pub fn handle_phase(
    commands: &mut Commands,
    mut stack: ResMut<ActionStack>,
    mut state: ResMut<State>,
    mut info: ResMut<Info>,
    data: Res<Data>,
    mut resources: ResMut<crate::resources::Resources>,
    mut player_query: Query<(Entity, &mut Player)>,
    mut treachery_cards: Query<(Entity, &mut Transform, &TreacheryCard)>,
    mut traitor_cards: Query<(Entity, &mut Transform, &Leader)>,
    mut storm_query: Query<&mut Storm>,
    storm_cards: Query<&StormCard>,
    mut unique_query: Query<(&mut Visible, &Unique)>,
    prediction_cards: Query<(Entity, &FactionPredictionCard)>,
    clickable_locations: Query<(Entity, &LocationSector)>,
) {
    // We need to resolve any pending actions first
    if stack.is_empty() {
        match state.phase {
            Phase::Setup { ref mut subphase } => {
                match subphase {
                    SetupSubPhase::ChooseFactions => {
                        // skip for now
                        set_view_to_active_player(&info, &mut player_query, &mut unique_query);
                        state.phase.advance();
                    }
                    SetupSubPhase::Prediction => {
                        for (_, player) in player_query.iter_mut() {
                            if player.faction == Faction::BeneGesserit {
                                for (mut visible, unique) in unique_query.iter_mut() {
                                    visible.is_visible = unique.faction == Faction::BeneGesserit;
                                }
                                // Animate in faction prediction cards
                                let num_factions = info.factions_in_play.len();
                                let animation_time = 1.5;
                                let delay = animation_time / (2.0 * num_factions as f32);
                                let indiv_anim_time =
                                    animation_time - (delay * (num_factions - 1) as f32);
                                let in_actions: Vec<Action> = prediction_cards
                                    .iter()
                                    .enumerate()
                                    .map(|(i, (element, _))| {
                                        simul![
                                            Action::animate_3d_to_ui(
                                                element,
                                                None,
                                                data.prediction_nodes.src,
                                                0.0,
                                            ),
                                            Action::delay(
                                                Action::animate_ui(
                                                    element,
                                                    data.prediction_nodes.src,
                                                    data.prediction_nodes.factions[i],
                                                    indiv_anim_time,
                                                ),
                                                delay * i as f32,
                                            ),
                                        ]
                                    })
                                    .collect();
                                let in_action = Action::Simultaneous {
                                    actions: in_actions,
                                };
                                let clickables = prediction_cards
                                    .iter()
                                    .map(|(element, _)| element)
                                    .collect();
                                let sequence = seq![
                                    in_action,
                                    Action::Enable { clickables },
                                    Action::await_indefinite(None),
                                ];
                                stack.push(sequence);
                            }
                        }
                    }
                    SetupSubPhase::AtStart => {
                        let players = player_query.iter_mut().collect::<HashMap<_, _>>();
                        let clickables = clickable_locations
                            .iter()
                            .map(|(entity, _)| entity)
                            .collect::<Vec<_>>();

                        let mut actions_map = players
                            .iter()
                            .map(|(&entity, player)| {
                                let (troops, locations, _) = player.faction.initial_values();
                                (
                                    entity,
                                    seq![
                                        Action::SwitchToActivePlayer,
                                        Action::await_indefinite(Some(Context::PlaceTroops)),
                                        Action::PassTurn
                                    ],
                                )
                            })
                            .collect::<HashMap<_, _>>();

                        let mut faction_order = info
                            .play_order
                            .iter()
                            .map(|entity| (entity, players[entity].faction))
                            .enumerate();

                        let (bg_pos, fr_pos) = (
                            faction_order
                                .find(|(_, (_, faction))| *faction == Faction::BeneGesserit)
                                .unwrap()
                                .0,
                            faction_order
                                .find(|(_, (_, faction))| *faction == Faction::Fremen)
                                .unwrap()
                                .0,
                        );

                        let mut actions = vec![Action::Enable { clickables }];
                        actions.extend(if bg_pos < fr_pos {
                            let order = faction_order.collect::<Vec<_>>();
                            order[..bg_pos]
                                .iter()
                                .chain(std::iter::once(&order[fr_pos]))
                                .chain(std::iter::once(&order[bg_pos]))
                                .chain(order[bg_pos + 1..fr_pos].iter())
                                .chain(order[fr_pos + 1..].iter())
                                .map(|(_, (entity, _))| actions_map.remove(entity).unwrap())
                                .collect::<Vec<_>>()
                        } else {
                            faction_order
                                .map(|(_, (entity, _))| actions_map.remove(entity).unwrap())
                                .collect::<Vec<_>>()
                        });

                        stack.push(Action::Sequence { actions })
                    }
                    SetupSubPhase::DealTraitors => {
                        for _ in 0..4 {
                            for &entity in info.play_order.iter() {
                                if let Ok((_, mut player)) = player_query.get_mut(entity) {
                                    player
                                        .traitor_cards
                                        .push(resources.traitor_deck.pop().unwrap());
                                }
                            }
                        }

                        *subphase = SetupSubPhase::PickTraitors;
                    }
                    SetupSubPhase::PickTraitors => {
                        // TODO: Add traitor cards as clickables
                        stack.push(seq![
                            Action::Enable { clickables: vec![] },
                            Action::await_indefinite(Some(Context::PickTraitors)),
                        ])
                    }
                    SetupSubPhase::DealTreachery => {
                        for &entity in info.play_order.iter() {
                            if let Ok((_, mut player)) = player_query.get_mut(entity) {
                                player
                                    .treachery_cards
                                    .push(resources.treachery_deck.pop().unwrap());
                                if player.faction == Faction::Harkonnen {
                                    player
                                        .treachery_cards
                                        .push(resources.treachery_deck.pop().unwrap());
                                }
                            }
                        }
                        state.phase = Phase::Storm {
                            subphase: StormSubPhase::Reveal,
                        };
                    }
                }
            }
            Phase::Storm { ref mut subphase } => {
                match subphase {
                    StormSubPhase::Reveal => {
                        // Make card visible to everyone
                        if info.turn == 0 {
                            *subphase = StormSubPhase::MoveStorm;
                        } else {
                            *subphase = StormSubPhase::WeatherControl;
                        }
                    }
                    StormSubPhase::WeatherControl => {
                        if let Some((entity, _, _)) = treachery_cards
                            .iter_mut()
                            .find(|(_, _, card)| card.name == "Weather Control")
                        {
                            // TODO: Add weather control card as clickable
                            stack.push(seq![
                                Action::Enable { clickables: vec![] },
                                Action::await_indefinite(Some(Context::PlayTreacheryPrompt)),
                            ]);
                        }

                        info.active_player += 1;
                        info.active_player %= info.play_order.len();
                    }
                    StormSubPhase::FamilyAtomics => {
                        if let Some((entity, _, _)) = treachery_cards
                            .iter_mut()
                            .find(|(_, _, card)| card.name == "Family Atomics")
                        {
                            // TODO: Add family atomics as clickable
                            stack.push(seq![
                                Action::Enable { clickables: vec![] },
                                Action::await_indefinite(Some(Context::PlayTreacheryPrompt)),
                            ]);
                        }

                        info.active_player += 1;
                        info.active_player %= info.play_order.len();
                    }
                    StormSubPhase::MoveStorm => {
                        let mut rng = rand::thread_rng();
                        if info.turn == 0 {
                            for mut storm in storm_query.iter_mut() {
                                storm.sector = rng.gen_range(0..18);
                            }
                        } else {
                            let &storm_card = resources.storm_deck.last().unwrap();
                            let delta = storm_cards.get(storm_card).unwrap().val;
                            for mut storm in storm_query.iter_mut() {
                                storm.sector += delta;
                                storm.sector %= 18;
                            }
                            // TODO: Kill everything it passed over and wipe spice
                            resources.storm_deck.shuffle(&mut rng)
                            // TODO: Choose a first player
                            // TODO: Assign bonuses
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Phase {
    Setup { subphase: SetupSubPhase },
    Storm { subphase: StormSubPhase },
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
            Phase::Setup { subphase } => match subphase {
                SetupSubPhase::ChooseFactions => Phase::Setup {
                    subphase: SetupSubPhase::Prediction,
                },
                SetupSubPhase::Prediction => Phase::Setup {
                    subphase: SetupSubPhase::AtStart,
                },
                SetupSubPhase::AtStart => Phase::Setup {
                    subphase: SetupSubPhase::DealTraitors,
                },
                SetupSubPhase::DealTraitors => Phase::Setup {
                    subphase: SetupSubPhase::PickTraitors,
                },
                SetupSubPhase::PickTraitors => Phase::Setup {
                    subphase: SetupSubPhase::DealTreachery,
                },
                SetupSubPhase::DealTreachery => Phase::Storm {
                    subphase: StormSubPhase::Reveal,
                },
            },
            Phase::Storm { subphase } => match subphase {
                StormSubPhase::Reveal => Phase::Storm {
                    subphase: StormSubPhase::WeatherControl,
                },
                StormSubPhase::WeatherControl => Phase::Storm {
                    subphase: StormSubPhase::FamilyAtomics,
                },
                StormSubPhase::FamilyAtomics => Phase::Storm {
                    subphase: StormSubPhase::MoveStorm,
                },
                StormSubPhase::MoveStorm => Phase::SpiceBlow,
            },
            Phase::SpiceBlow => Phase::Nexus,
            Phase::Nexus => Phase::Bidding,
            Phase::Bidding => Phase::Revival,
            Phase::Revival => Phase::Movement,
            Phase::Movement => Phase::Battle,
            Phase::Battle => Phase::Collection,
            Phase::Collection => Phase::Control,
            Phase::Control => Phase::Storm {
                subphase: StormSubPhase::Reveal,
            },
            Phase::EndGame => Phase::EndGame,
        }
    }

    pub fn advance(&mut self) {
        *self = self.next();
    }
}

#[derive(Copy, Clone)]
pub enum SetupSubPhase {
    ChooseFactions,
    Prediction,
    AtStart,
    DealTraitors,
    PickTraitors,
    DealTreachery,
}

#[derive(Copy, Clone)]
pub enum StormSubPhase {
    Reveal,
    WeatherControl,
    FamilyAtomics,
    MoveStorm,
}

pub struct State {
    pub phase: Phase,
}

impl Default for State {
    fn default() -> Self {
        State {
            phase: Phase::Setup {
                subphase: SetupSubPhase::ChooseFactions,
            },
        }
    }
}
