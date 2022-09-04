use super::*;
use crate::{
    components::PlayerFaction,
    single,
};

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
        self.0.push_back(ActionAggregation::Multiple(actions).into())
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
        self.0.push_front(ActionAggregation::Multiple(actions).into())
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

enum ActionResult {
    None,
    Remove,
    Replace(ActionChain),
    Add(ActionChain),
}

pub fn action(
    mut commands: Commands,
    time: Res<Time>,
    mut info: ResMut<Info>,
    mut phase: ResMut<GamePhase>,
    mut queue: ResMut<ActionQueue>,
    mut queries: QuerySet<(Query<&mut Lerp>, Query<&PlayerFaction>, Query<&mut Collider>)>,
) {
    // println!("Context: {:?}, Queue: {}", info.context, queue.to_string());
    // println!(
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
                    match action_subsystem(commands, action, &time, &mut info, &mut phase, &mut queries) {
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
                        match action_subsystem(commands, &mut action, &time, &mut info, &mut phase, &mut queries) {
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
    } else {
        phase.advance();
    }
}

fn action_subsystem(
    mut commands: Commands,
    action: &mut ActionChain,
    time: &Res<Time>,
    info: &mut ResMut<Info>,
    phase: &mut ResMut<GamePhase>,
    queries: &mut QuerySet<(Query<&mut Lerp>, Query<&PlayerFaction>, Query<&mut Collider>)>,
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
                queries.q1_mut().get(info.get_active_player()).unwrap().0
            );
            if info.active_player.is_some() {
                info.active_player = None;
                println!(" to {:?}", queries.q1_mut().get(info.get_active_player()).unwrap().0);
            } else {
                info.current_turn += 1;
                if info.current_turn >= info.play_order.len() {
                    info.current_turn %= info.play_order.len();
                    println!(" to {:?}", queries.q1_mut().get(info.get_active_player()).unwrap().0);
                    action.append(Action::AdvancePhase.into());
                } else {
                    println!(" to {:?}", queries.q1_mut().get(info.get_active_player()).unwrap().0);
                }
            }
        }
        Action::AdvancePhase => {
            phase.advance();
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
