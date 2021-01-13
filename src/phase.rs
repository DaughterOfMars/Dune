use std::{
    collections::{HashMap, VecDeque},
    ops::DerefMut,
};

use crate::{
    components::{Collider, Disorganized, Troop},
    data::{TraitorCard, TurnPredictionCard},
    lerper::{Lerp, LerpType},
    util::shuffle_deck,
};
use bevy::{prelude::*, render::camera::Camera};
use rand::{prelude::SliceRandom, Rng};

use crate::{
    components::{LocationSector, Player, Storm, Unique},
    data::{Faction, FactionPredictionCard, Leader, StormCard, TreacheryCard},
    resources::{Data, Info},
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
        app.add_resource(ActionQueue::default())
            .add_stage(STAGE, SystemStage::parallel())
            .add_system_to_stage(STAGE, action_system.system())
            .add_system_to_stage(STAGE, public_troop_system.system())
            .add_system_to_stage(STAGE, active_player_system.system())
            .add_system_to_stage(STAGE, stack_troops_system.system())
            .add_system_to_stage(STAGE, setup_phase_system.system())
            .add_system_to_stage(STAGE, storm_phase_system.system());
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Context {
    None,
    Predicting,
    PlacingTroops,
    PickingTraitors,
    Prompting,
    StackResolving,
}

impl Context {
    pub fn action(&self, action: Action) -> ContextAction {
        ContextAction {
            action: single!(action),
            context: *self,
        }
    }

    pub fn actions(&self, actions: Vec<Action>) -> ContextAction {
        ContextAction {
            action: ActionAggregation::Multiple(actions),
            context: *self,
        }
    }
}

#[derive(Clone)]
pub enum Action {
    Enable { clickables: Vec<Entity> },
    SetActivePlayer { player: Entity },
    PassTurn,
    AdvancePhase,
    Lerp { element: Entity, lerp: Option<Lerp> },
    ContextChange(Context),
    Delay { time: f32 },
}

impl Action {
    pub fn add_lerp(element: Entity, lerp: Lerp) -> Self {
        Self::Lerp {
            element,
            lerp: Some(lerp),
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Enable { .. } => write!(f, "Enable"),
            Action::PassTurn => write!(f, "PassTurn"),
            Action::AdvancePhase => write!(f, "AdvancePhase"),
            Action::Lerp { .. } => write!(f, "Lerp"),
            Action::ContextChange(context) => write!(f, "ContextChange({:?})", context),
            Action::Delay { time } => write!(f, "Delay({})", time),
            Action::SetActivePlayer { player } => write!(f, "SetActivePlayer({:?})", player),
        }
    }
}

pub struct ContextAction {
    pub action: ActionAggregation,
    pub context: Context,
}

impl Into<ContextAction> for Action {
    fn into(self) -> ContextAction {
        ContextAction {
            action: single!(self),
            context: Context::None,
        }
    }
}

impl Into<ContextAction> for ActionAggregation {
    fn into(self) -> ContextAction {
        ContextAction {
            action: self,
            context: Context::None,
        }
    }
}

impl std::fmt::Display for ContextAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}-{}", self.context, self.action)
    }
}

pub enum ActionAggregation {
    Single(Action),
    Multiple(Vec<Action>),
}

impl std::fmt::Display for ActionAggregation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(action) => write!(f, "{}", action),
            Self::Multiple(actions) => {
                let mut s = String::from("[");
                for action in actions {
                    s.push_str(&format!("{},", action));
                }
                write!(f, "{}]", s)
            }
        }
    }
}

pub struct ActionQueue(VecDeque<ContextAction>);

impl Default for ActionQueue {
    fn default() -> Self {
        ActionQueue(VecDeque::new())
    }
}

impl ActionQueue {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, action: ContextAction) {
        self.0.push_back(action)
    }

    pub fn push_single(&mut self, action: Action) {
        self.0.push_back(action.into())
    }

    pub fn push_single_for_context(&mut self, action: Action, context: Context) {
        self.0.push_back(context.action(action))
    }

    pub fn push_multiple(&mut self, actions: Vec<Action>) {
        self.0
            .push_back(ActionAggregation::Multiple(actions).into())
    }

    pub fn push_multiple_for_context(&mut self, actions: Vec<Action>, context: Context) {
        self.0.push_back(context.actions(actions))
    }

    pub fn push_front(&mut self, action: ContextAction) {
        self.0.push_front(action)
    }

    pub fn push_single_front(&mut self, action: Action) {
        self.0.push_front(action.into())
    }

    pub fn push_single_front_for_context(&mut self, action: Action, context: Context) {
        self.0.push_front(context.action(action))
    }

    pub fn push_multiple_front(&mut self, actions: Vec<Action>) {
        self.0
            .push_front(ActionAggregation::Multiple(actions).into())
    }

    pub fn push_multiple_front_for_context(&mut self, actions: Vec<Action>, context: Context) {
        self.0.push_front(context.actions(actions))
    }

    pub fn peek(&self) -> Option<&ContextAction> {
        self.0.front()
    }

    pub fn peek_mut(&mut self) -> Option<&mut ContextAction> {
        self.0.front_mut()
    }

    pub fn pop(&mut self) -> Option<ContextAction> {
        self.0.pop_front()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn extend<T: IntoIterator<Item = ContextAction>>(&mut self, iter: T) {
        self.0.extend(iter);
    }

    pub fn push_seq_front<T: IntoIterator<Item = ContextAction>>(&mut self, iter: T)
    where
        <T as IntoIterator>::IntoIter: DoubleEndedIterator,
    {
        let mut iter = iter.into_iter().rev();
        while let Some(element) = iter.next() {
            if self.0.len() == self.0.capacity() {
                let (lower, _) = iter.size_hint();
                self.0.reserve(lower.saturating_add(1));
            }

            self.0.push_front(element);
        }
    }
}

impl std::fmt::Display for ActionQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for item in self.0.iter() {
            s.push_str(&format!("{}, ", item));
        }
        write!(f, "{}", s)
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
    time: Res<Time>,
    mut info: ResMut<Info>,
    mut state: ResMut<State>,
    mut queue: ResMut<ActionQueue>,
    mut queries: QuerySet<(Query<&mut Lerp>, Query<&Player>, Query<&mut Collider>)>,
) {
    println!("Context: {:?}, Queue: {}", info.context, queue.to_string());
    //println!(
    //    "Active player: {:?}",
    //    queries.q1().get(info.get_active_player()).unwrap().faction
    //);

    if let Some(ContextAction {
        action: aggregate,
        context,
    }) = queue.peek_mut()
    {
        if info.context == *context {
            match aggregate {
                ActionAggregation::Single(action) => {
                    match action_subsystem(
                        commands,
                        action,
                        &time,
                        &mut info,
                        &mut state,
                        &mut queries,
                    ) {
                        ActionResult::None => (),
                        ActionResult::Remove => {
                            queue.pop();
                        }
                        ActionResult::Replace { action: new_action } => {
                            *action = new_action;
                        }
                        ActionResult::Add { action: new_action } => {
                            queue.push_single(new_action);
                        }
                    }
                }
                ActionAggregation::Multiple(actions) => {
                    let mut new_actions = Vec::new();
                    for mut action in actions.drain(..) {
                        match action_subsystem(
                            commands,
                            &mut action,
                            &time,
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
                    if new_actions.is_empty() {
                        queue.pop();
                    } else {
                        *actions = new_actions;
                    }
                }
            }
        }
    }
}

fn action_subsystem(
    commands: &mut Commands,
    action: &mut Action,
    time: &Res<Time>,
    info: &mut ResMut<Info>,
    state: &mut ResMut<State>,
    queries: &mut QuerySet<(Query<&mut Lerp>, Query<&Player>, Query<&mut Collider>)>,
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
            print!(
                "Pass turn from {:?}",
                queries
                    .q1_mut()
                    .get(info.get_active_player())
                    .unwrap()
                    .faction
            );
            if info.active_player.is_some() {
                info.active_player = None;
                println!(
                    " to {:?}",
                    queries
                        .q1_mut()
                        .get(info.get_active_player())
                        .unwrap()
                        .faction
                );
                ActionResult::Remove
            } else {
                info.current_turn += 1;
                if info.current_turn >= info.play_order.len() {
                    info.current_turn %= info.play_order.len();
                    println!(
                        " to {:?}",
                        queries
                            .q1_mut()
                            .get(info.get_active_player())
                            .unwrap()
                            .faction
                    );
                    ActionResult::Replace {
                        action: Action::AdvancePhase,
                    }
                } else {
                    println!(
                        " to {:?}",
                        queries
                            .q1_mut()
                            .get(info.get_active_player())
                            .unwrap()
                            .faction
                    );
                    ActionResult::Remove
                }
            }
        }
        Action::AdvancePhase => {
            state.phase.advance();
            ActionResult::Remove
        }
        Action::Lerp {
            element,
            lerp: new_lerp,
        } => {
            if let Ok(mut old_lerp) = queries.q0_mut().get_mut(*element) {
                if old_lerp.time <= 0.0 {
                    if let Some(lerp) = new_lerp.take() {
                        *old_lerp = lerp;
                        ActionResult::None
                    } else {
                        ActionResult::Remove
                    }
                } else {
                    ActionResult::None
                }
            } else {
                if let Some(lerp) = new_lerp.take() {
                    commands.insert_one(*element, lerp);
                    ActionResult::None
                } else {
                    ActionResult::Remove
                }
            }
        }
        Action::ContextChange(context) => {
            info.context = *context;
            ActionResult::Remove
        }
        Action::Delay { time: remaining } => {
            *remaining -= time.delta_seconds();
            if *remaining <= 0.0 {
                ActionResult::Remove
            } else {
                ActionResult::None
            }
        }
        Action::SetActivePlayer { player } => {
            info.active_player = Some(*player);
            ActionResult::Remove
        }
    }
}

fn stack_troops_system(
    commands: &mut Commands,
    mut queue: ResMut<ActionQueue>,
    troops: Query<(Entity, &Unique, &Troop)>,
    locations: Query<(Entity, &LocationSector), With<Disorganized>>,
) {
    for (loc_entity, loc_sec) in locations.iter() {
        let mut map = HashMap::new();
        for (entity, faction) in troops.iter().filter_map(|(entity, unique, troop)| {
            troop.location.and_then(|location| {
                if location == loc_entity {
                    Some((entity, unique.faction))
                } else {
                    None
                }
            })
        }) {
            map.entry(faction).or_insert(Vec::new()).push(entity);
        }
        for (node_ind, troops) in map.values().enumerate() {
            let node = loc_sec.location.sectors[&loc_sec.sector].fighters[node_ind];
            queue.push_multiple_front(
                troops
                    .iter()
                    .enumerate()
                    .map(|(i, entity)| {
                        Action::add_lerp(
                            *entity,
                            Lerp::new(
                                LerpType::World {
                                    src: None,
                                    dest: Transform::from_translation(Vec3::new(
                                        node.x, node.z, -node.y,
                                    )) * Transform::from_translation(
                                        i as f32 * 0.0018 * Vec3::unit_y(),
                                    ),
                                },
                                0.1,
                                0.0,
                            ),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
        }
        commands.remove_one::<Disorganized>(loc_entity);
    }
}

fn public_troop_system(mut troops: Query<(&Troop, &mut Unique)>) {
    for (troop, mut unique) in troops.iter_mut() {
        unique.public = troop.location.is_some();
    }
}

fn active_player_system(
    info: Res<Info>,
    players: Query<&Player>,
    mut uniques: Query<(&mut Visible, &Unique)>,
) {
    let entity = info
        .active_player
        .unwrap_or(info.play_order[info.current_turn]);
    let active_player_faction = players.get(entity).unwrap().faction;
    for (mut visible, unique) in uniques.iter_mut() {
        if visible.is_visible != (unique.public || unique.faction == active_player_faction) {
            visible.is_visible = unique.public || unique.faction == active_player_faction;
        }
    }
}

pub fn setup_phase_system(
    mut queue: ResMut<ActionQueue>,
    mut state: ResMut<State>,
    mut info: ResMut<Info>,
    data: Res<Data>,
    mut players: Query<(Entity, &mut Player)>,
    mut treachery_cards: Query<(Entity, &mut Transform, &TreacheryCard)>,
    mut traitor_cards: Query<(Entity, &mut Transform, &TraitorCard)>,
    prediction_cards: QuerySet<(
        Query<(Entity, &FactionPredictionCard)>,
        Query<(Entity, &TurnPredictionCard)>,
    )>,
    mut uniques: Query<(&mut Visible, &Unique)>,
    clickable_locations: Query<(Entity, &LocationSector)>,
    cameras: Query<Entity, With<Camera>>,
    mut troops: Query<(Entity, &mut Troop, &Unique, &Transform)>,
) {
    // We need to resolve any pending actions first
    if queue.is_empty() {
        if let Phase::Setup { ref mut subphase } = state.phase {
            match subphase {
                SetupSubPhase::ChooseFactions => {
                    let mut rng = rand::thread_rng();
                    shuffle_deck(
                        &mut rng,
                        0.001,
                        &mut treachery_cards
                            .iter_mut()
                            .map(|(entity, transform, _)| (entity, transform))
                            .collect(),
                    );
                    shuffle_deck(
                        &mut rng,
                        0.001,
                        &mut traitor_cards
                            .iter_mut()
                            .map(|(entity, transform, _)| (entity, transform))
                            .collect(),
                    );
                    // skip for now
                    state.phase.advance();
                }
                SetupSubPhase::Prediction => {
                    for (entity, player) in players.iter_mut() {
                        if player.faction == Faction::BeneGesserit {
                            info.active_player = Some(entity);
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
                                        Lerp::new(
                                            LerpType::UI {
                                                src: Some(data.prediction_nodes.src),
                                                dest: data.prediction_nodes.factions[i],
                                            },
                                            indiv_anim_time,
                                            delay * i as f32,
                                        ),
                                    )
                                })
                                .collect::<Vec<_>>();
                            queue.push_multiple(actions);
                            let clickables = prediction_cards
                                .q0()
                                .iter()
                                .map(|(element, _)| element)
                                .collect();
                            queue.push_single(Action::Enable { clickables });
                            queue.push_single(Action::ContextChange(Context::Predicting));

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
                                        Lerp::new(
                                            LerpType::UI {
                                                src: Some(data.prediction_nodes.src),
                                                dest: data.prediction_nodes.turns[i],
                                            },
                                            indiv_anim_time,
                                            delay * i as f32,
                                        ),
                                    )
                                })
                                .collect::<Vec<_>>();
                            queue.push_multiple(actions);
                            let clickables = prediction_cards
                                .q1()
                                .iter()
                                .map(|(element, _)| element)
                                .collect();
                            queue.push_single(Action::Enable { clickables });
                            queue.push_single(Action::ContextChange(Context::Predicting));
                            queue.push_single(Action::PassTurn);
                            queue.push_single(Action::AdvancePhase);
                            break;
                        }
                    }
                }
                SetupSubPhase::AtStart => {
                    let clickables = clickable_locations
                        .iter()
                        .map(|(entity, _)| entity)
                        .collect::<Vec<_>>();

                    let mut actions_map = players
                        .iter_mut()
                        .map(|(entity, player)| {
                            let (num_troops, locations, _) = player.faction.initial_values();
                            (
                                entity,
                                // Check if we even have free troops to place
                                if num_troops > 0 {
                                    if let Some(locations) = locations {
                                        let mut res =
                                            vec![Action::SetActivePlayer { player: entity }];
                                        if locations.len() == 0 {
                                            // Do nothing
                                        } else if locations.len() == 1 {
                                            let (location, loc_sec) = clickable_locations
                                                .iter()
                                                .find(|(_, loc_sec)| {
                                                    loc_sec.location.name == locations[0]
                                                })
                                                .unwrap();
                                            let mut troop_stack = troops
                                                .iter_mut()
                                                .filter(|(_, troop, unique, _)| {
                                                    unique.faction == player.faction
                                                        && troop.location.is_none()
                                                })
                                                .collect::<Vec<_>>();
                                            troop_stack.sort_by(
                                                |(_, _, _, transform1), (_, _, _, transform2)| {
                                                    transform1
                                                        .translation
                                                        .y
                                                        .partial_cmp(&transform2.translation.y)
                                                        .unwrap()
                                                },
                                            );
                                            res.extend((0..num_troops).map(|i| {
                                                if let Some((entity, troop, _, _)) =
                                                    troop_stack.get_mut(i as usize)
                                                {
                                                    troop.location = Some(location);
                                                    let node = loc_sec.location.sectors
                                                        [&loc_sec.sector]
                                                        .fighters[0];
                                                    Action::add_lerp(
                                                        *entity,
                                                        Lerp::new(
                                                            LerpType::World {
                                                                src: None,
                                                                dest: Transform::from_translation(
                                                                    Vec3::new(
                                                                        node.x, node.z, -node.y,
                                                                    ),
                                                                )
                                                                    * Transform::from_translation(
                                                                        i as f32
                                                                            * 0.0018
                                                                            * Vec3::unit_y(),
                                                                    ),
                                                            },
                                                            0.1,
                                                            0.0,
                                                        ),
                                                    )
                                                } else {
                                                    panic!();
                                                }
                                            }));
                                        } else {
                                            res.push(Action::ContextChange(Context::PlacingTroops));
                                        };
                                        res
                                    } else {
                                        vec![
                                            Action::SetActivePlayer { player: entity },
                                            Action::ContextChange(Context::PlacingTroops),
                                        ]
                                    }
                                } else {
                                    vec![]
                                },
                            )
                        })
                        .collect::<HashMap<_, _>>();

                    let faction_order = info
                        .play_order
                        .iter()
                        .map(|&entity| (entity, players.get_mut(entity).unwrap().1.faction))
                        .enumerate()
                        .collect::<Vec<_>>();

                    let (bg_pos, fr_pos) = (
                        faction_order
                            .iter()
                            .find(|(_, (_, faction))| *faction == Faction::BeneGesserit)
                            .unwrap()
                            .0,
                        faction_order
                            .iter()
                            .find(|(_, (_, faction))| *faction == Faction::Fremen)
                            .unwrap()
                            .0,
                    );

                    // Move the camera so we can see the board good
                    queue.push_single(Action::add_lerp(
                        cameras.iter().next().unwrap(),
                        Lerp::move_camera(data.camera_nodes.board, 1.0),
                    ));
                    queue.push_single(Action::Enable { clickables });
                    queue.extend(if bg_pos < fr_pos {
                        faction_order[..bg_pos]
                            .iter()
                            .chain(std::iter::once(&faction_order[fr_pos]))
                            .chain(std::iter::once(&faction_order[bg_pos]))
                            .chain(faction_order[bg_pos + 1..fr_pos].iter())
                            .chain(faction_order[fr_pos + 1..].iter())
                            .map(|(_, (entity, _))| actions_map.remove(entity).unwrap())
                            .flatten()
                            .map(|action| action.into())
                            .collect::<Vec<_>>()
                    } else {
                        faction_order
                            .iter()
                            .map(|(_, (entity, _))| actions_map.remove(&entity).unwrap())
                            .flatten()
                            .map(|action| action.into())
                            .collect::<Vec<_>>()
                    });
                    queue.push_single(Action::add_lerp(
                        cameras.iter().next().unwrap(),
                        Lerp::move_camera(data.camera_nodes.main, 1.0),
                    ));
                    queue.push_single(Action::AdvancePhase);
                }
                SetupSubPhase::DealTraitors => {
                    let mut cards = traitor_cards
                        .iter_mut()
                        .map(|(entity, transform, _)| (entity, transform))
                        .collect::<Vec<_>>();
                    cards.sort_by(|(_, transform1), (_, transform2)| {
                        transform1
                            .translation
                            .y
                            .partial_cmp(&transform2.translation.y)
                            .unwrap()
                    });
                    for _ in 0..4 {
                        for &entity in info.play_order.iter() {
                            if let Ok((_, mut player)) = players.get_mut(entity) {
                                let card = cards.pop().unwrap().0;
                                player.traitor_cards.push(card);
                            }
                        }
                    }

                    *subphase = SetupSubPhase::PickTraitors;
                }
                SetupSubPhase::PickTraitors => {
                    // TODO: Add traitor cards as clickables
                    todo!();
                    queue.push_single(Action::Enable { clickables: vec![] });
                    queue.push_single(Action::ContextChange(Context::PickingTraitors));
                    queue.push_single(Action::PassTurn);
                }
                SetupSubPhase::DealTreachery => {
                    /*
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
                    */
                }
            }
        }
    }
}

pub fn storm_phase_system(
    mut queue: ResMut<ActionQueue>,
    mut state: ResMut<State>,
    mut info: ResMut<Info>,
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
                        queue.push_single(Action::Enable { clickables: vec![] });
                        queue.push_single(Action::ContextChange(Context::Prompting));
                        queue.push_single(Action::PassTurn);
                    }
                }
                StormSubPhase::FamilyAtomics => {
                    if let Some((entity, _, _)) = treachery_cards
                        .iter_mut()
                        .find(|(_, _, card)| card.name == "Family Atomics")
                    {
                        // TODO: Add family atomics as clickable
                        queue.push_single(Action::Enable { clickables: vec![] });
                        queue.push_single(Action::ContextChange(Context::Prompting));
                        queue.push_single(Action::PassTurn);
                    }
                }
                StormSubPhase::MoveStorm => {
                    /*
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
                    */
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
