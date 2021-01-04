use std::{f32::consts::PI, ops::DerefMut};

use bevy::{prelude::*, render::camera::Camera};

use crate::{
    components::*,
    data::*,
    resources::*,
    util::{screen_to_world, set_view_to_active_player},
};

#[macro_export]
macro_rules! seq {
    ($($e:expr),+ $(,)?) => {
        Action::Sequence {
            actions: vec![$($e),+]
        }
    };
}

#[macro_export]
macro_rules! simul {
    ($($e:expr),+ $(,)?) => {
        Action::Simultaneous {
            actions: vec![$($e),+]
        }
    };
}

#[derive(Clone, Debug)]
pub enum Action {
    // Allows a player to choose between multiple actions
    Choice {
        player: Entity,
        options: Vec<Action>,
    },
    // Allows multiple actions to occur at once
    Simultaneous {
        actions: Vec<Action>,
    },
    // Execute a series of actions in order
    Sequence {
        actions: Vec<Action>,
    },
    // Wait for some action to occur before resolving this one
    Await {
        timer: Option<f32>,
        context: Option<Context>,
    },
    Delay {
        time: f32,
        action: Box<Action>,
    },
    SwitchToActivePlayer,
    Show {
        element: Entity,
    },
    Hide {
        element: Entity,
    },
    Enable {
        clickables: Vec<Entity>,
    },
    AnimateUIElement {
        element: Entity,
        src: Vec2,
        dest: Vec2,
        animation_time: f32,
        current_time: f32,
    },
    Animate3DElement {
        element: Entity,
        src: Option<Transform>,
        dest: Transform,
        animation_time: f32,
        current_time: f32,
    },
    AnimateUIElementTo3D {
        element: Entity,
        src: Vec2,
        dest: Transform,
        animation_time: f32,
        current_time: f32,
    },
    Animate3DElementToUI {
        element: Entity,
        src: Option<Transform>,
        dest: Vec2,
        animation_time: f32,
        current_time: f32,
    },
    MakePrediction {
        prediction: Prediction,
    },
    PlaceTroop {
        location: Entity,
        animation_time: f32,
    },
    CameraMotion {
        src: Option<Transform>,
        dest: CameraNode,
        remaining_time: f32,
        total_time: f32,
    },
    PassTurn,
    AdvancePhase,
    DebugPrint {
        text: String,
    },
}

impl Action {
    pub fn move_camera(dest: CameraNode, time: f32) -> Self {
        Action::CameraMotion {
            src: None,
            dest,
            remaining_time: time,
            total_time: time,
        }
    }

    pub fn place_troop(location: Entity, animation_time: f32) -> Self {
        Action::PlaceTroop {
            location,
            animation_time,
        }
    }

    pub fn animate_ui(element: Entity, src: Vec2, dest: Vec2, time: f32) -> Self {
        Action::AnimateUIElement {
            element,
            src,
            dest,
            animation_time: time,
            current_time: 0.0,
        }
    }

    pub fn animate_3d_to_ui(
        element: Entity,
        src: Option<Transform>,
        dest: Vec2,
        time: f32,
    ) -> Self {
        Action::Animate3DElementToUI {
            element,
            src,
            dest,
            animation_time: time,
            current_time: 0.0,
        }
    }

    pub fn animate_3d(element: Entity, src: Option<Transform>, dest: Transform, time: f32) -> Self {
        Action::Animate3DElement {
            element,
            src,
            dest,
            animation_time: time,
            current_time: 0.0,
        }
    }

    pub fn await_indefinite(context: Option<Context>) -> Self {
        Action::Await {
            timer: None,
            context,
        }
    }

    pub fn await_timed(time: f32, context: Option<Context>) -> Self {
        Action::Await {
            timer: Some(time),
            context,
        }
    }

    pub fn delay(action: Action, time: f32) -> Self {
        Action::Delay {
            action: Box::new(action),
            time,
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Choice { player, options } => {
                let s = options
                    .iter()
                    .map(|option| option.to_string())
                    .collect::<Vec<String>>()
                    .join("/");
                write!(f, "Choice({})", s)
            }
            Action::Simultaneous { actions } => {
                let s = actions
                    .iter()
                    .map(|action| action.to_string())
                    .collect::<Vec<String>>()
                    .join(" + ");
                write!(f, "Simul({})", s)
            }
            Action::Sequence { actions } => {
                let s = actions
                    .iter()
                    .map(|action| action.to_string())
                    .collect::<Vec<String>>()
                    .join(" -> ");
                write!(f, "Seq({})", s)
            }
            Action::Await { timer, .. } => {
                if let Some(timer) = timer {
                    write!(f, "Await(remaining={})", timer)
                } else {
                    write!(f, "Await(Forever)")
                }
            }
            Action::Delay { time, action } => write!(f, "Delay({}, remaining={})", action, time),
            Action::Show { element } => write!(f, "Show({:?})", *element),
            Action::Hide { element } => write!(f, "Hide({:?})", *element),
            Action::AnimateUIElement {
                element,
                src,
                dest,
                animation_time,
                current_time,
            } => write!(f, "AnimateUIElement"),
            Action::Animate3DElement {
                element,
                src,
                dest,
                animation_time,
                current_time,
            } => write!(f, "Animate3DElement"),
            Action::AnimateUIElementTo3D {
                element,
                src,
                dest,
                animation_time,
                current_time,
            } => write!(f, "AnimateUIElementTo3D"),
            Action::Animate3DElementToUI {
                element,
                src,
                dest,
                animation_time,
                current_time,
            } => write!(f, "Animate3DElementToUI"),
            Action::MakePrediction { prediction } => {
                if let Some(faction) = prediction.faction {
                    write!(f, "MakePrediction({:?})", faction)
                } else {
                    if let Some(turn) = prediction.turn {
                        write!(f, "MakePrediction({})", turn)
                    } else {
                        write!(f, "MakePrediction")
                    }
                }
            }
            Action::AdvancePhase => write!(f, "AdvancePhase"),
            Action::CameraMotion {
                src,
                dest,
                remaining_time,
                total_time,
            } => write!(f, "CameraMotion"),
            Action::Enable { clickables } => write!(f, "Enable"),
            Action::DebugPrint { text } => write!(f, "DebugPrint({})", text),
            Action::SwitchToActivePlayer => write!(f, "SwitchToActivePlayer"),
            Action::PassTurn => write!(f, "PassTurn"),
            Action::PlaceTroop {
                location,
                animation_time: _,
            } => write!(f, "PlaceTroop"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Context {
    PlaceTroops,
    PickTraitors,
    PlayTreacheryPrompt,
    PlayTraitorPrompt,
}

pub type ActionGenerator = dyn Fn(Entity) -> Action + Send + Sync;

pub struct ActionStack(pub Vec<Action>);

impl Default for ActionStack {
    fn default() -> Self {
        ActionStack(Vec::new())
    }
}

impl ActionStack {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, action: Action) {
        self.0.push(action);
    }

    pub fn peek(&self) -> Option<&Action> {
        self.0.last()
    }

    pub fn peek_mut(&mut self) -> Option<&mut Action> {
        self.0.last_mut()
    }

    pub fn pop(&mut self) -> Option<Action> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn extend<T: IntoIterator<Item = Action>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}
enum ActionResult {
    None,
    Remove,
    Replace { action: Action },
    Add { action: Action },
}

pub fn handle_actions(
    mut stack: ResMut<ActionStack>,
    mut state: ResMut<crate::phase::State>,
    mut info: ResMut<Info>,
    data: Res<Data>,
    mut resources: ResMut<crate::resources::Resources>,
    mut player_query: Query<(Entity, &mut Player)>,
    storm_cards: Query<&StormCard>,
    mut cameras: Query<(&mut Transform, &Camera)>,
    mut uniques: Query<(&mut Visible, &Unique)>,
    mut transforms: Query<&mut Transform, Without<Camera>>,
    time: Res<Time>,
    mut predictions: Query<&mut Prediction>,
    prediction_cards: QuerySet<(
        Query<(Entity, &FactionPredictionCard)>,
        Query<(Entity, &TurnPredictionCard)>,
    )>,
    mut colliders: Query<(&Collider, &mut Option<ClickAction>)>,
    mut troops: Query<(&mut Troop, &mut Transform)>,
    //mut locations: &Query<(Entity, &Location)>,
) {
    /*
    print!("Stack: ");
    for item in stack.0.iter().rev() {
        print!("{}, ", item);
    }
    println!();
    */
    if let Some(mut action) = stack.pop() {
        match handle_action_recursive(
            &mut action,
            &mut stack,
            &mut state,
            &mut info,
            &data,
            &mut resources,
            &mut player_query,
            &storm_cards,
            &mut cameras,
            &mut uniques,
            &time,
            &mut transforms,
            &mut predictions,
            &prediction_cards,
            &mut colliders,
            &mut troops,
            //&mut locations,
        ) {
            ActionResult::None => {
                stack.push(action);
            }
            ActionResult::Remove => (),
            ActionResult::Replace {
                action: replace_action,
            } => {
                stack.push(replace_action);
            }
            ActionResult::Add { action: add_action } => {
                stack.push(action);
                stack.push(add_action);
            }
        };
    }
}

fn handle_action_recursive(
    action: &mut Action,
    stack: &mut ResMut<ActionStack>,
    state: &mut ResMut<crate::phase::State>,
    info: &mut ResMut<Info>,
    data: &Res<Data>,
    resources: &mut ResMut<crate::resources::Resources>,
    player_query: &mut Query<(Entity, &mut Player)>,
    storm_cards: &Query<&StormCard>,
    cameras: &mut Query<(&mut Transform, &Camera)>,
    uniques: &mut Query<(&mut Visible, &Unique)>,
    time: &Res<Time>,
    transforms: &mut Query<&mut Transform, Without<Camera>>,
    predictions: &mut Query<&mut Prediction>,
    prediction_cards: &QuerySet<(
        Query<(Entity, &FactionPredictionCard)>,
        Query<(Entity, &TurnPredictionCard)>,
    )>,
    colliders: &mut Query<(&Collider, &mut Option<ClickAction>)>,
    troops: &mut Query<(&mut Troop, &mut Transform)>,
    //locations: &mut Query<(Entity, &Location)>,
) -> ActionResult {
    //println!("{}", action);
    match action {
        Action::Simultaneous { actions } => {
            let mut new_actions = Vec::new();
            for mut action in actions.drain(..) {
                match handle_action_recursive(
                    &mut action,
                    stack,
                    state,
                    info,
                    data,
                    resources,
                    player_query,
                    storm_cards,
                    cameras,
                    uniques,
                    time,
                    transforms,
                    predictions,
                    prediction_cards,
                    colliders,
                    troops,
                    //locations,
                ) {
                    ActionResult::None => {
                        new_actions.push(action);
                    }
                    ActionResult::Remove => (),
                    ActionResult::Replace {
                        action: replace_action,
                    } => {
                        new_actions.push(replace_action);
                    }
                    ActionResult::Add { action: add_action } => {
                        new_actions.push(action);
                        new_actions.push(add_action);
                    }
                };
            }
            if !new_actions.is_empty() {
                ActionResult::Replace {
                    action: Action::Simultaneous {
                        actions: new_actions,
                    },
                }
            } else {
                ActionResult::Remove
            }
        }
        Action::Sequence { actions } => {
            actions.reverse();
            stack.extend(actions.drain(..));
            if let Some(mut action) = stack.pop() {
                handle_action_recursive(
                    &mut action,
                    stack,
                    state,
                    info,
                    data,
                    resources,
                    player_query,
                    storm_cards,
                    cameras,
                    uniques,
                    time,
                    transforms,
                    predictions,
                    prediction_cards,
                    colliders,
                    troops,
                    //locations,
                )
            } else {
                ActionResult::Remove
            }
        }
        Action::CameraMotion {
            src,
            dest,
            remaining_time,
            total_time,
        } => {
            if let Some((mut cam_transform, _)) = cameras.iter_mut().next() {
                if *remaining_time <= 0.0 {
                    *cam_transform =
                        Transform::from_translation(dest.pos).looking_at(dest.at, dest.up);
                    return ActionResult::Remove;
                } else {
                    if cam_transform.translation != dest.pos {
                        if let Some(src_transform) = src {
                            let mut lerp_amount =
                                PI * (*total_time - *remaining_time) / *total_time;
                            lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                            let dest_transform =
                                Transform::from_translation(dest.pos).looking_at(dest.at, dest.up);
                            *cam_transform = Transform::from_translation(
                                src_transform
                                    .translation
                                    .lerp(dest_transform.translation, lerp_amount),
                            ) * Transform::from_rotation(
                                src_transform
                                    .rotation
                                    .lerp(dest_transform.rotation, lerp_amount),
                            );
                        } else {
                            *src = Some(cam_transform.clone())
                        }
                        *remaining_time -= time.delta_seconds();
                        return ActionResult::None;
                    } else {
                        return ActionResult::Remove;
                    }
                }
            } else {
                return ActionResult::Remove;
            }
        }
        Action::AnimateUIElement {
            element,
            src,
            dest,
            animation_time,
            current_time,
        } => {
            if let Ok(mut element_transform) = transforms.get_mut(*element) {
                if let Some((cam_transform, camera)) = cameras.iter_mut().next() {
                    if *current_time >= *animation_time {
                        element_transform.translation = screen_to_world(
                            dest.extend(0.1),
                            *cam_transform,
                            camera.projection_matrix,
                        );

                        return ActionResult::Remove;
                    } else {
                        let mut lerp_amount =
                            PI * (*current_time).min(*animation_time) / *animation_time;
                        lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                        element_transform.translation = screen_to_world(
                            src.extend(0.1),
                            *cam_transform,
                            camera.projection_matrix,
                        )
                        .lerp(
                            screen_to_world(
                                dest.extend(0.1),
                                *cam_transform,
                                camera.projection_matrix,
                            ),
                            lerp_amount,
                        );

                        *current_time += time.delta_seconds();
                    }
                }
            }
            ActionResult::None
        }
        Action::Animate3DElementToUI {
            element,
            src,
            dest,
            animation_time,
            current_time,
        } => {
            if let Ok(mut element_transform) = transforms.get_mut(*element) {
                if let Some((cam_transform, camera)) = cameras.iter_mut().next() {
                    if src.is_none() {
                        src.replace(element_transform.clone());
                    }
                    if *current_time >= *animation_time {
                        element_transform.translation = screen_to_world(
                            dest.extend(0.1),
                            *cam_transform,
                            camera.projection_matrix,
                        );
                        element_transform.rotation = cam_transform.rotation * src.unwrap().rotation;

                        return ActionResult::Remove;
                    } else {
                        let mut lerp_amount =
                            PI * (*current_time).min(*animation_time) / *animation_time;
                        lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                        element_transform.translation = src.unwrap().translation.lerp(
                            screen_to_world(
                                dest.extend(0.1),
                                *cam_transform,
                                camera.projection_matrix,
                            ),
                            lerp_amount,
                        );
                        element_transform.rotation = src
                            .unwrap()
                            .rotation
                            .lerp(cam_transform.rotation * src.unwrap().rotation, lerp_amount);

                        *current_time += time.delta_seconds();
                    }
                }
            }
            ActionResult::None
        }
        Action::MakePrediction { prediction } => {
            for mut player_prediction in predictions.iter_mut() {
                player_prediction.faction = prediction.faction.or(player_prediction.faction);
                player_prediction.turn = prediction.turn.or(player_prediction.turn);
            }
            if prediction.faction.is_some() {
                let num_factions = info.factions_in_play.len();
                let animation_time = 1.5;
                let mut delay = animation_time / (2.0 * num_factions as f32);
                let mut indiv_anim_time = animation_time - (delay * (num_factions - 1) as f32);
                // Animate selected card
                let chosen_action = prediction_cards
                    .q0()
                    .iter()
                    .enumerate()
                    .find(|(_, (_, card))| card.faction == prediction.faction.unwrap())
                    .map(|(i, (element, _))| {
                        Action::animate_ui(
                            element,
                            data.prediction_nodes.factions[i],
                            data.prediction_nodes.chosen_faction,
                            1.0,
                        )
                    })
                    .unwrap();
                // Animate out faction cards
                let mut out_actions: Vec<Action> = prediction_cards
                    .q0()
                    .iter()
                    .filter(|(_, card)| card.faction != prediction.faction.unwrap())
                    .enumerate()
                    .map(|(i, (element, _))| {
                        Action::delay(
                            Action::animate_ui(
                                element,
                                data.prediction_nodes.factions[i],
                                data.prediction_nodes.src,
                                indiv_anim_time,
                            ),
                            1.0 + (delay * i as f32),
                        )
                    })
                    .collect();
                out_actions.push(chosen_action);
                let out_action = Action::Simultaneous {
                    actions: out_actions,
                };
                // Animate in turn cards
                delay = animation_time / 30.0;
                indiv_anim_time = animation_time - (delay * 14.0);
                let in_actions: Vec<Action> = prediction_cards
                    .q1()
                    .iter()
                    .enumerate()
                    .map(|(i, (element, _))| {
                        simul![
                            Action::animate_3d_to_ui(element, None, data.prediction_nodes.src, 0.0),
                            Action::delay(
                                Action::animate_ui(
                                    element,
                                    data.prediction_nodes.src,
                                    data.prediction_nodes.turns[i],
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
                    .q1()
                    .iter()
                    .map(|(element, _)| element)
                    .collect();
                return ActionResult::Replace {
                    action: seq![
                        out_action,
                        in_action,
                        Action::Enable { clickables },
                        Action::await_indefinite(None),
                    ],
                };
            } else if prediction.turn.is_some() {
                let animation_time = 1.5;
                let delay = animation_time / 30.0;
                let indiv_anim_time = animation_time - (delay * 14.0);
                // Animate selected card
                let chosen_action = prediction_cards
                    .q1()
                    .iter()
                    .enumerate()
                    .find(|(_, (_, card))| card.turn == prediction.turn.unwrap())
                    .map(|(i, (element, _))| {
                        Action::animate_ui(
                            element,
                            data.prediction_nodes.turns[i],
                            data.prediction_nodes.chosen_turn,
                            1.0,
                        )
                    })
                    .unwrap();
                // Animate out turn cards
                let mut out_actions: Vec<Action> = prediction_cards
                    .q1()
                    .iter()
                    .filter(|(_, card)| card.turn != prediction.turn.unwrap())
                    .enumerate()
                    .map(|(i, (element, _))| {
                        Action::delay(
                            Action::animate_ui(
                                element,
                                data.prediction_nodes.turns[i],
                                data.prediction_nodes.src,
                                indiv_anim_time,
                            ),
                            1.0 + (delay * i as f32),
                        )
                    })
                    .collect();
                out_actions.push(chosen_action);
                let out_action = Action::Simultaneous {
                    actions: out_actions,
                };
                return ActionResult::Replace {
                    action: seq![out_action, Action::delay(Action::AdvancePhase, 1.5)],
                };
            }
            ActionResult::Remove
        }
        Action::PlaceTroop {
            location,
            animation_time,
        } => {
            if let Ok((_, player)) = player_query.get_mut(info.play_order[info.active_player]) {
                todo!();
            }
            todo!();
        }
        Action::Await { timer, .. } => {
            if let Some(timer) = timer {
                *timer -= time.delta_seconds();
                if *timer <= 0.0 {
                    ActionResult::Remove
                } else {
                    ActionResult::None
                }
            } else {
                ActionResult::None
            }
        }
        Action::Delay {
            action: delayed,
            time: timer,
        } => {
            *timer -= time.delta_seconds();
            if *timer <= 0.0 {
                ActionResult::Replace {
                    action: delayed.deref_mut().clone(),
                }
            } else {
                ActionResult::None
            }
        }
        Action::AdvancePhase => {
            state.phase.advance();
            ActionResult::Remove
        }
        Action::Enable { clickables } => {
            for (_, mut action) in colliders.iter_mut() {
                if let Some(action) = action.deref_mut() {
                    action.enabled = false;
                }
            }
            for &entity in clickables.iter() {
                if let Ok((_, mut action)) = colliders.get_mut(entity) {
                    if let Some(action) = action.deref_mut() {
                        action.enabled = true;
                    }
                }
            }
            ActionResult::Remove
        }
        Action::SwitchToActivePlayer => {
            set_view_to_active_player(&info, player_query, uniques);
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
        Action::DebugPrint { text } => {
            println!("Debug: {}", text);
            ActionResult::Remove
        }
        _ => {
            return ActionResult::Remove;
        }
    }
}
