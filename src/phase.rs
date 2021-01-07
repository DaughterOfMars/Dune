use std::collections::{HashMap, VecDeque};

use crate::{
    components::Collider,
    lerper::{Lerp, LerpType},
    resources::Collections,
};
use bevy::prelude::*;
use rand::{prelude::SliceRandom, Rng};

use crate::{
    components::{LocationSector, Player, Storm, Unique},
    data::{Faction, FactionPredictionCard, Leader, StormCard, TreacheryCard},
    resources::{Data, Info},
    util::set_view_to_active_player,
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

const STAGE: &str = "phase";

pub struct PhasePlugin;

impl Plugin for PhasePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_stage(STAGE, SystemStage::parallel())
            .add_system_to_stage(STAGE, action_system.system())
            .add_system_to_stage(STAGE, active_player_system.system())
            .add_system_to_stage(STAGE, setup_phase_system.system())
            .add_system_to_stage(STAGE, storm_phase_system.system());
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Context {
    None,
    Predicting,
    PlacingTroops,
    PickingTraitors,
    Prompting,
    StackResolving,
}

pub enum Action {
    Enable { clickables: Vec<Entity> },
    PassTurn,
    AdvancePhase,
    Lerp { element: Entity, lerp: Option<Lerp> },
    ContextChange { context: Context },
}

impl Action {
    pub fn add_lerp(element: Entity, lerp: Lerp) -> Self {
        Self::Lerp {
            element,
            lerp: Some(lerp),
        }
    }
}

pub enum ActionAggregation {
    Single(Action),
    Multiple(Vec<Action>),
}

pub struct ActionQueue(VecDeque<ActionAggregation>);

impl Default for ActionQueue {
    fn default() -> Self {
        ActionQueue(VecDeque::new())
    }
}

impl ActionQueue {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, action: ActionAggregation) {
        self.0.push_back(action)
    }

    pub fn push_front(&mut self, action: ActionAggregation) {
        self.0.push_front(action)
    }

    pub fn peek(&self) -> Option<&ActionAggregation> {
        self.0.front()
    }

    pub fn peek_mut(&mut self) -> Option<&mut ActionAggregation> {
        self.0.front_mut()
    }

    pub fn pop(&mut self) -> Option<ActionAggregation> {
        self.0.pop_front()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn extend<T: IntoIterator<Item = ActionAggregation>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

enum ActionResult {
    None,
    Remove,
    Replace { action: Action },
    Add { action: Action },
}

pub fn action_system(
    commands: &mut Commands,
    mut info: ResMut<Info>,
    mut state: ResMut<State>,
    mut queue: ResMut<ActionQueue>,
    mut queries: QuerySet<(Query<&Lerp>, Query<&Player>, Query<&mut Collider>)>,
) {
    if info.context == Context::None {
        if let Some(aggregate) = queue.peek_mut() {
            match aggregate {
                ActionAggregation::Single(action) => {
                    match action_subsystem(commands, action, &mut info, &mut state, &mut queries) {
                        ActionResult::None => (),
                        ActionResult::Remove => {
                            queue.pop();
                        }
                        ActionResult::Replace { action: new_action } => {
                            *action = new_action;
                        }
                        ActionResult::Add { action: new_action } => {
                            queue.push(ActionAggregation::Single(new_action));
                        }
                    }
                }
                ActionAggregation::Multiple(actions) => {
                    let mut new_actions = Vec::new();
                    for mut action in actions.drain(..) {
                        match action_subsystem(
                            commands,
                            &mut action,
                            &mut info,
                            &mut state,
                            &mut queries,
                        ) {
                            ActionResult::None => new_actions.push(action),
                            ActionResult::Remove => (),
                            ActionResult::Replace { action: new_action } => {
                                new_actions.push(new_action)
                            }
                            ActionResult::Add { action: new_action } => {
                                new_actions.push(action);
                                new_actions.push(new_action)
                            }
                        }
                    }
                }
            }
        }
    }
}

fn action_subsystem(
    commands: &mut Commands,
    action: &mut Action,
    info: &mut ResMut<Info>,
    state: &mut ResMut<State>,
    queries: &mut QuerySet<(Query<&Lerp>, Query<&Player>, Query<&mut Collider>)>,
) -> ActionResult {
    match action {
        Action::Enable { clickables } => {
            for mut collider in queries.q2_mut().iter_mut() {
                collider.enabled = false;
            }
            for &entity in clickables.iter() {
                if let Ok(mut collider) = queries.q2_mut().get_mut(entity) {
                    collider.enabled = true;
                }
            }
            ActionResult::Remove
        }
        Action::PassTurn => {
            info.active_player += 1;
            if info.active_player >= info.play_order.len() {
                info.active_player %= info.play_order.len();
                ActionResult::Replace {
                    action: Action::AdvancePhase,
                }
            } else {
                ActionResult::Remove
            }
        }
        Action::AdvancePhase => {
            state.phase.advance();
            ActionResult::Remove
        }
        Action::Lerp { element, lerp } => {
            if let Some(lerp) = lerp.take() {
                commands.insert_one(*element, lerp);
                return ActionResult::None;
            }
            if let Ok(lerp) = queries.q0().get(*element) {
                if lerp.time <= 0.0 {
                    return ActionResult::Remove;
                }
            }
            ActionResult::None
        }
        Action::ContextChange { context } => {
            info.context = *context;
            ActionResult::Remove
        }
    }
}

fn active_player_system(
    info: Res<Info>,
    players: Query<&Player>,
    mut uniques: Query<(&mut Visible, &Unique)>,
) {
    let entity = info.play_order[info.active_player];
    let active_player_faction = players.get(entity).unwrap().faction;
    for (mut visible, unique) in uniques.iter_mut() {
        visible.is_visible = unique.faction == active_player_faction;
    }
}

pub fn setup_phase_system(
    mut queue: ResMut<ActionQueue>,
    mut state: ResMut<State>,
    mut info: ResMut<Info>,
    data: Res<Data>,
    mut collections: ResMut<Collections>,
    mut players: Query<(Entity, &mut Player)>,
    mut treachery_cards: Query<(Entity, &mut Transform, &TreacheryCard)>,
    mut traitor_cards: Query<(Entity, &mut Transform, &Leader)>,
    prediction_cards: QuerySet<(
        Query<(Entity, &FactionPredictionCard)>,
        Query<(Entity, &FactionPredictionCard)>,
    )>,
    mut uniques: Query<(&mut Visible, &Unique)>,
    clickable_locations: Query<(Entity, &LocationSector)>,
) {
    // We need to resolve any pending actions first
    if queue.is_empty() {
        if let Phase::Setup { ref mut subphase } = state.phase {
            match subphase {
                SetupSubPhase::ChooseFactions => {
                    // skip for now
                    set_view_to_active_player(&info, &mut players, &mut uniques);
                    state.phase.advance();
                }
                SetupSubPhase::Prediction => {
                    for (_, player) in players.iter_mut() {
                        if player.faction == Faction::BeneGesserit {
                            for (mut visible, unique) in uniques.iter_mut() {
                                visible.is_visible = unique.faction == Faction::BeneGesserit;
                            }
                            // Lerp in faction cards
                            let num_factions = info.factions_in_play.len();
                            let animation_time = 1.5;
                            let delay = animation_time / (2.0 * num_factions as f32);
                            let indiv_anim_time =
                                animation_time - (delay * (num_factions - 1) as f32);

                            let actions = prediction_cards
                                .q0()
                                .iter()
                                .enumerate()
                                .map(|(i, (element, _))| {
                                    Action::add_lerp(
                                        element,
                                        Lerp {
                                            lerp_type: LerpType::UI {
                                                src: Some(data.prediction_nodes.src),
                                                dest: data.prediction_nodes.factions[i],
                                            },
                                            time: indiv_anim_time,
                                            delay: delay * i as f32,
                                        },
                                    )
                                })
                                .collect::<Vec<_>>();
                            queue.push(ActionAggregation::Multiple(actions));
                            let clickables = prediction_cards
                                .q0()
                                .iter()
                                .map(|(element, _)| element)
                                .collect();
                            queue.push(single!(Action::Enable { clickables }));
                            queue.push(single!(Action::ContextChange {
                                context: Context::Predicting,
                            }));

                            // Lerp in Turn Cards
                            let animation_time = 1.5;
                            let delay = animation_time / 30.0;
                            let indiv_anim_time = animation_time - (delay * 14.0);

                            let actions = prediction_cards
                                .q1()
                                .iter()
                                .enumerate()
                                .map(|(i, (element, _))| {
                                    Action::add_lerp(
                                        element,
                                        Lerp {
                                            lerp_type: LerpType::UI {
                                                src: Some(data.prediction_nodes.src),
                                                dest: data.prediction_nodes.turns[i],
                                            },
                                            time: indiv_anim_time,
                                            delay: delay * i as f32,
                                        },
                                    )
                                })
                                .collect::<Vec<_>>();
                            queue.push(ActionAggregation::Multiple(actions));
                            let clickables = prediction_cards
                                .q1()
                                .iter()
                                .map(|(element, _)| element)
                                .collect();
                            queue.push(ActionAggregation::Single(Action::Enable { clickables }));
                            queue.push(ActionAggregation::Single(Action::ContextChange {
                                context: Context::Predicting,
                            }));
                            break;
                        }
                    }
                }
                SetupSubPhase::AtStart => {
                    //let players = players.iter_mut().collect::<HashMap<_, _>>();
                    let clickables = clickable_locations
                        .iter()
                        .map(|(entity, _)| entity)
                        .collect::<Vec<_>>();

                    let mut actions_map = players
                        .iter_mut()
                        .map(|(entity, player)| {
                            let (troops, locations, _) = player.faction.initial_values();
                            (
                                entity,
                                vec![
                                    Action::ContextChange {
                                        context: Context::PlacingTroops,
                                    },
                                    Action::PassTurn,
                                ],
                            )
                        })
                        .collect::<HashMap<_, _>>();

                    let mut faction_order = info
                        .play_order
                        .iter()
                        .map(|&entity| (entity, players.get_mut(entity).unwrap().1.faction))
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

                    queue.push(single!(Action::Enable { clickables }));
                    queue.extend(if bg_pos < fr_pos {
                        let order = faction_order.collect::<Vec<_>>();
                        order[..bg_pos]
                            .iter()
                            .chain(std::iter::once(&order[fr_pos]))
                            .chain(std::iter::once(&order[bg_pos]))
                            .chain(order[bg_pos + 1..fr_pos].iter())
                            .chain(order[fr_pos + 1..].iter())
                            .map(|(_, (entity, _))| actions_map.remove(entity).unwrap())
                            .flatten()
                            .map(|action| single!(action))
                            .collect::<Vec<_>>()
                    } else {
                        faction_order
                            .map(|(_, (entity, _))| actions_map.remove(&entity).unwrap())
                            .flatten()
                            .map(|action| single!(action))
                            .collect::<Vec<_>>()
                    });
                }
                SetupSubPhase::DealTraitors => {
                    for _ in 0..4 {
                        for &entity in info.play_order.iter() {
                            if let Ok((_, mut player)) = players.get_mut(entity) {
                                player
                                    .traitor_cards
                                    .push(collections.traitor_deck.pop().unwrap());
                            }
                        }
                    }

                    *subphase = SetupSubPhase::PickTraitors;
                }
                SetupSubPhase::PickTraitors => {
                    // TODO: Add traitor cards as clickables
                    todo!();
                    queue.push(single!(Action::Enable { clickables: vec![] }));
                    queue.push(single!(Action::ContextChange {
                        context: Context::PickingTraitors,
                    }));
                    queue.push(single!(Action::PassTurn));
                }
                SetupSubPhase::DealTreachery => {
                    for &entity in info.play_order.iter() {
                        if let Ok((_, mut player)) = players.get_mut(entity) {
                            player
                                .treachery_cards
                                .push(collections.treachery_deck.pop().unwrap());
                            if player.faction == Faction::Harkonnen {
                                player
                                    .treachery_cards
                                    .push(collections.treachery_deck.pop().unwrap());
                            }
                        }
                    }
                    state.phase = Phase::Storm {
                        subphase: StormSubPhase::Reveal,
                    };
                }
            }
        }
    }
}

pub fn storm_phase_system(
    mut queue: ResMut<ActionQueue>,
    mut state: ResMut<State>,
    mut info: ResMut<Info>,
    mut collections: ResMut<Collections>,
    mut treachery_cards: Query<(Entity, &mut Transform, &TreacheryCard)>,
    mut storm_query: Query<&mut Storm>,
    storm_cards: Query<&StormCard>,
) {
    if queue.is_empty() {
        if let Phase::Storm { ref mut subphase } = state.phase {
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
                        todo!();
                        queue.push(single!(Action::Enable { clickables: vec![] }));
                        queue.push(single!(Action::ContextChange {
                            context: Context::Prompting,
                        }));
                        queue.push(single!(Action::PassTurn));
                    }
                }
                StormSubPhase::FamilyAtomics => {
                    if let Some((entity, _, _)) = treachery_cards
                        .iter_mut()
                        .find(|(_, _, card)| card.name == "Family Atomics")
                    {
                        // TODO: Add family atomics as clickable
                        queue.push(single!(Action::Enable { clickables: vec![] }));
                        queue.push(single!(Action::ContextChange {
                            context: Context::Prompting,
                        }));
                        queue.push(single!(Action::PassTurn));
                    }
                }
                StormSubPhase::MoveStorm => {
                    let mut rng = rand::thread_rng();
                    if info.turn == 0 {
                        for mut storm in storm_query.iter_mut() {
                            storm.sector = rng.gen_range(0..18);
                        }
                    } else {
                        let &storm_card = collections.storm_deck.last().unwrap();
                        let delta = storm_cards.get(storm_card).unwrap().val;
                        for mut storm in storm_query.iter_mut() {
                            storm.sector += delta;
                            storm.sector %= 18;
                        }
                        // TODO: Kill everything it passed over and wipe spice
                        collections.storm_deck.shuffle(&mut rng)
                        // TODO: Choose a first player
                        // TODO: Assign bonuses
                    }
                }
            }
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
