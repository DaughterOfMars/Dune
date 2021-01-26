use std::{
    collections::{HashMap, VecDeque},
    f32::consts::PI,
    ops::DerefMut,
};

use crate::{
    components::{Collider, Disorganized, Troop, UniqueBundle},
    data::{TraitorCard, TurnPredictionCard},
    lerper::{Lerp, LerpType, UITransform},
    util::{hand_positions, shuffle_deck},
    Screen, RESPONSE_STAGE, STATE_CHANGE_STAGE,
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

pub struct PhasePlugin;

impl Plugin for PhasePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(ActionQueue::default())
            .init_resource::<GamePhase>()
            .on_state_update(
                STATE_CHANGE_STAGE,
                crate::Screen::HostingGame,
                action_system.system(),
            )
            .on_state_update(
                STATE_CHANGE_STAGE,
                crate::Screen::HostingGame,
                phase_text_system.system(),
            )
            .on_state_update(
                STATE_CHANGE_STAGE,
                crate::Screen::HostingGame,
                public_troop_system.system(),
            )
            .on_state_update(
                STATE_CHANGE_STAGE,
                crate::Screen::HostingGame,
                active_player_system.system(),
            )
            .on_state_update(
                STATE_CHANGE_STAGE,
                crate::Screen::HostingGame,
                stack_troops_system.system(),
            )
            .on_state_update(
                STATE_CHANGE_STAGE,
                crate::Screen::HostingGame,
                setup_phase_system.system(),
            )
            .on_state_update(
                STATE_CHANGE_STAGE,
                crate::Screen::HostingGame,
                storm_phase_system.system(),
            )
            .on_state_exit(RESPONSE_STAGE, Screen::HostingGame, reset.system());
    }
}

pub struct PhaseText;

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
    pub fn action(&self, action: ActionChain) -> ContextAction {
        ContextAction {
            action: single!(action),
            context: *self,
        }
    }

    pub fn actions(&self, actions: Vec<ActionChain>) -> ContextAction {
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
    Assign { element: Entity, faction: Faction },
}

impl Action {
    pub fn add_lerp(element: Entity, lerp: Lerp) -> Self {
        Self::Lerp {
            element,
            lerp: Some(lerp),
        }
    }

    pub fn then(self, next: ActionChain) -> ActionChain {
        ActionChain {
            current: self,
            next: Some(Box::new(next)),
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
            Action::Assign { element, faction } => {
                write!(f, "Assign({:?} -> {:?})", element, faction)
            }
        }
    }
}

#[derive(Clone)]
pub struct ActionChain {
    current: Action,
    next: Option<Box<ActionChain>>,
}

impl ActionChain {
    pub fn append(&mut self, next: ActionChain) {
        let mut next_ref = &mut self.next;
        while next_ref.is_some() {
            next_ref = &mut next_ref.as_mut().unwrap().next;
        }
        next_ref.replace(Box::new(next));
    }
}

impl std::fmt::Display for ActionChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref next) = self.next {
            write!(f, "Chain({} -> {})", self.current, next)
        } else {
            write!(f, "{}", self.current)
        }
    }
}

impl From<Action> for ActionChain {
    fn from(action: Action) -> Self {
        ActionChain {
            current: action,
            next: None,
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
            action: single!(self.into()),
            context: Context::None,
        }
    }
}

impl Into<ContextAction> for ActionChain {
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
    Single(ActionChain),
    Multiple(Vec<ActionChain>),
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

    pub fn push_single(&mut self, action: ActionChain) {
        self.0.push_back(action.into())
    }

    pub fn push_single_for_context(&mut self, action: ActionChain, context: Context) {
        self.0.push_back(context.action(action))
    }

    pub fn push_multiple(&mut self, actions: Vec<ActionChain>) {
        self.0
            .push_back(ActionAggregation::Multiple(actions).into())
    }

    pub fn push_multiple_for_context(&mut self, actions: Vec<ActionChain>, context: Context) {
        self.0.push_back(context.actions(actions))
    }

    pub fn push_front(&mut self, action: ContextAction) {
        self.0.push_front(action)
    }

    pub fn push_single_front(&mut self, action: ActionChain) {
        self.0.push_front(action.into())
    }

    pub fn push_single_front_for_context(&mut self, action: ActionChain, context: Context) {
        self.0.push_front(context.action(action))
    }

    pub fn push_multiple_front(&mut self, actions: Vec<ActionChain>) {
        self.0
            .push_front(ActionAggregation::Multiple(actions).into())
    }

    pub fn push_multiple_front_for_context(&mut self, actions: Vec<ActionChain>, context: Context) {
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

    pub fn clear(&mut self) {
        self.0.clear();
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
    Replace(ActionChain),
    Add(ActionChain),
}

pub fn action_system(
    commands: &mut Commands,
    time: Res<Time>,
    mut info: ResMut<Info>,
    mut phase: ResMut<GamePhase>,
    mut queue: ResMut<ActionQueue>,
    mut queries: QuerySet<(Query<&mut Lerp>, Query<&Player>, Query<&mut Collider>)>,
) {
    //println!("Context: {:?}, Queue: {}", info.context, queue.to_string());
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
                        &mut phase,
                        &mut queries,
                    ) {
                        ActionResult::None => (),
                        ActionResult::Remove => {
                            queue.pop();
                        }
                        ActionResult::Replace(new_action) => {
                            *action = new_action;
                        }
                        ActionResult::Add(new_action) => {
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
                            &mut phase,
                            &mut queries,
                        ) {
                            ActionResult::None => new_actions.push(action),
                            ActionResult::Remove => (),
                            ActionResult::Replace(new_action) => new_actions.push(new_action),
                            ActionResult::Add(new_action) => {
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
    action: &mut ActionChain,
    time: &Res<Time>,
    info: &mut ResMut<Info>,
    state: &mut ResMut<GamePhase>,
    queries: &mut QuerySet<(Query<&mut Lerp>, Query<&Player>, Query<&mut Collider>)>,
) -> ActionResult {
    match action.current {
        Action::Enable { ref clickables } => {
            for mut collider in queries.q2_mut().iter_mut() {
                collider.enabled = false;
            }
            for &entity in clickables.iter() {
                if let Ok(mut collider) = queries.q2_mut().get_mut(entity) {
                    collider.enabled = true;
                }
            }
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
                    action.append(Action::AdvancePhase.into());
                } else {
                    println!(
                        " to {:?}",
                        queries
                            .q1_mut()
                            .get(info.get_active_player())
                            .unwrap()
                            .faction
                    );
                }
            }
        }
        Action::AdvancePhase => {
            state.phase.advance();
        }
        Action::Lerp {
            element,
            lerp: ref mut new_lerp,
        } => {
            if let Ok(mut old_lerp) = queries.q0_mut().get_mut(element) {
                if old_lerp.time <= 0.0 {
                    if let Some(lerp) = new_lerp.take() {
                        *old_lerp = lerp;
                        return ActionResult::None;
                    }
                } else {
                    return ActionResult::None;
                }
            } else {
                if let Some(lerp) = new_lerp.take() {
                    commands.insert_one(element, lerp);
                    return ActionResult::None;
                }
            }
        }
        Action::ContextChange(context) => {
            info.context = context;
        }
        Action::Delay {
            time: ref mut remaining,
        } => {
            *remaining -= time.delta_seconds();
            if *remaining > 0.0 {
                return ActionResult::None;
            }
        }
        Action::SetActivePlayer { player } => {
            info.active_player = Some(player);
        }
        Action::Assign { element, faction } => {
            commands.insert(element, UniqueBundle::new(faction));
        }
    }
    if let Some(next) = action.next.take() {
        ActionResult::Replace(*next)
    } else {
        ActionResult::Remove
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
                                LerpType::world_to(
                                    Transform::from_translation(Vec3::new(node.x, node.z, -node.y))
                                        * Transform::from_translation(
                                            i as f32 * 0.0018 * Vec3::unit_y(),
                                        ),
                                ),
                                0.1,
                                0.0,
                            ),
                        )
                        .into()
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

fn phase_text_system(
    state: Res<GamePhase>,
    info: Res<Info>,
    players: Query<&Player>,
    mut text: Query<&mut Text, With<PhaseText>>,
) {
    let active_faction = players.get(info.get_active_player()).unwrap().faction;
    let s = match state.phase {
        Phase::Setup { subphase } => match subphase {
            SetupSubPhase::ChooseFactions => "Choosing Factions...".to_string(),
            SetupSubPhase::Prediction => "Bene Gesserit are making a prediction...".to_string(),
            SetupSubPhase::AtStart => format!("{:?} Initial Placement...", active_faction),
            SetupSubPhase::DealTraitors => "Dealing Traitor Cards...".to_string(),
            SetupSubPhase::PickTraitors => "Picking Traitors...".to_string(),
            SetupSubPhase::DealTreachery => "Dealing Treachery Cards...".to_string(),
        },
        Phase::Storm { subphase: _ } => "Storm Phase".to_string(),
        Phase::SpiceBlow => "Spice Blow Phase".to_string(),
        Phase::Nexus => "Nexus Phase".to_string(),
        Phase::Bidding => "Bidding Phase".to_string(),
        Phase::Revival => "Revival Phase".to_string(),
        Phase::Movement => "Movement Phase".to_string(),
        Phase::Battle => "Battle Phase".to_string(),
        Phase::Collection => "Collection Phase".to_string(),
        Phase::Control => "Control Phase".to_string(),
        Phase::EndGame => "".to_string(),
    };

    if let Some(mut text) = text.iter_mut().next() {
        text.value = s;
    }
}

fn setup_phase_system(
    mut queue: ResMut<ActionQueue>,
    mut state: ResMut<GamePhase>,
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
                                            LerpType::ui_from_to(
                                                (
                                                    data.prediction_nodes.src,
                                                    Quat::from_rotation_x(0.5 * PI),
                                                )
                                                    .into(),
                                                (
                                                    data.prediction_nodes.factions[i],
                                                    Quat::from_rotation_x(0.5 * PI),
                                                )
                                                    .into(),
                                            ),
                                            indiv_anim_time,
                                            delay * i as f32,
                                        ),
                                    )
                                    .into()
                                })
                                .collect::<Vec<_>>();
                            queue.push_multiple(actions);
                            let clickables = prediction_cards
                                .q0()
                                .iter()
                                .map(|(element, _)| element)
                                .collect();
                            queue.push_single(Action::Enable { clickables }.into());
                            queue.push_single(Action::ContextChange(Context::Predicting).into());

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
                                            LerpType::ui_from_to(
                                                (
                                                    data.prediction_nodes.src,
                                                    Quat::from_rotation_x(0.5 * PI),
                                                    0.6,
                                                )
                                                    .into(),
                                                (
                                                    data.prediction_nodes.turns[i],
                                                    Quat::from_rotation_x(0.5 * PI),
                                                    0.6,
                                                )
                                                    .into(),
                                            ),
                                            indiv_anim_time,
                                            delay * i as f32,
                                        ),
                                    )
                                    .into()
                                })
                                .collect::<Vec<_>>();
                            queue.push_multiple(actions);
                            let clickables = prediction_cards
                                .q1()
                                .iter()
                                .map(|(element, _)| element)
                                .collect();
                            queue.push_single(Action::Enable { clickables }.into());
                            queue.push_single(Action::ContextChange(Context::Predicting).into());
                            queue.push_single(Action::PassTurn.into());
                            queue.push_single(Action::AdvancePhase.into());
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
                                                            LerpType::world_to(
                                                                Transform::from_translation(
                                                                    Vec3::new(
                                                                        node.x, node.z, -node.y,
                                                                    ),
                                                                ) * Transform::from_translation(
                                                                    i as f32
                                                                        * 0.0018
                                                                        * Vec3::unit_y(),
                                                                ),
                                                            ),
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
                    queue.push_single(
                        Action::add_lerp(
                            cameras.iter().next().unwrap(),
                            Lerp::move_camera(data.camera_nodes.board, 1.0),
                        )
                        .into(),
                    );
                    queue.push_single(Action::Enable { clickables }.into());
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
                    queue.push_single(
                        Action::add_lerp(
                            cameras.iter().next().unwrap(),
                            Lerp::move_camera(data.camera_nodes.main, 1.0),
                        )
                        .into(),
                    );
                    queue.push_single(Action::AdvancePhase.into());
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
                    let mut actions = Vec::new();
                    let positions = hand_positions(4);
                    let turn_tile_pts = data
                        .ui_structure
                        .get_turn_tiles()
                        .iter()
                        .map(|tile| tile.center())
                        .collect::<Vec<_>>();

                    let mut delay = 0.0;
                    for i in 0..4 {
                        for (j, &entity) in info.play_order.iter().enumerate() {
                            if let Ok((_, mut player)) = players.get_mut(entity) {
                                let card = cards.pop().unwrap().0;
                                player.traitor_cards.push(card);
                                if entity == info.get_active_player() {
                                    actions.push(
                                        Action::add_lerp(
                                            card,
                                            Lerp::new(
                                                LerpType::card_to_ui(positions[i], 1.0),
                                                0.6,
                                                delay,
                                            ),
                                        )
                                        .then(
                                            Action::Assign {
                                                element: card,
                                                faction: player.faction,
                                            }
                                            .into(),
                                        ),
                                    );
                                } else {
                                    actions.push(
                                        Action::add_lerp(
                                            card,
                                            Lerp::new(
                                                LerpType::world_to_ui(
                                                    (
                                                        turn_tile_pts[j],
                                                        Quat::from_rotation_x(0.5 * PI)
                                                            * Quat::from_rotation_z(PI),
                                                        0.4,
                                                    )
                                                        .into(),
                                                ),
                                                0.6,
                                                delay,
                                            ),
                                        )
                                        .then(
                                            Action::Assign {
                                                element: card,
                                                faction: player.faction,
                                            }
                                            .into(),
                                        ),
                                    );
                                }
                            }
                            delay += 0.2;
                        }
                    }
                    queue.push_multiple(actions);

                    *subphase = SetupSubPhase::PickTraitors;
                }
                SetupSubPhase::PickTraitors => {
                    // TODO: Add traitor cards as clickables
                    queue.push_single(Action::Enable { clickables: vec![] }.into());
                    queue.push_single(Action::ContextChange(Context::PickingTraitors).into());
                    queue.push_single(Action::PassTurn.into());
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

fn storm_phase_system(
    mut queue: ResMut<ActionQueue>,
    mut state: ResMut<GamePhase>,
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
                        queue.push_single(Action::Enable { clickables: vec![] }.into());
                        queue.push_single(Action::ContextChange(Context::Prompting).into());
                        queue.push_single(Action::PassTurn.into());
                    }
                }
                StormSubPhase::FamilyAtomics => {
                    if let Some((entity, _, _)) = treachery_cards
                        .iter_mut()
                        .find(|(_, _, card)| card.name == "Family Atomics")
                    {
                        // TODO: Add family atomics as clickable
                        queue.push_single(Action::Enable { clickables: vec![] }.into());
                        queue.push_single(Action::ContextChange(Context::Prompting).into());
                        queue.push_single(Action::PassTurn.into());
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

pub struct GamePhase {
    pub phase: Phase,
}

impl Default for GamePhase {
    fn default() -> Self {
        GamePhase {
            phase: Phase::Setup {
                subphase: SetupSubPhase::ChooseFactions,
            },
        }
    }
}

fn reset(mut phase: ResMut<GamePhase>, mut queue: ResMut<ActionQueue>) {
    phase.phase = Phase::Setup {
        subphase: SetupSubPhase::ChooseFactions,
    };
    queue.clear();
}
