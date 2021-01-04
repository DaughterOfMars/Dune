use std::collections::HashMap;

use bevy::{
    ecs::{Bundle, Entity},
    math::Vec3,
    prelude::{GlobalTransform, Transform, Visible},
};
use ncollide3d::{
    na::Vector3,
    shape::{Cuboid, ShapeHandle},
};

use crate::{
    data::{Faction, Leader},
    stack::{Action, ActionGenerator, Context},
};

#[derive(Copy, Clone)]
pub struct Spice {
    pub value: i32,
}

#[derive(Copy, Clone)]
pub struct Troop {
    pub value: i32,
    pub location: Option<Entity>,
}

#[derive(Default)]
pub struct Storm {
    pub sector: i32,
}

pub struct LocationSector;

pub struct Sector {
    pub sector: i32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct SpiceNode {
    pub pos: Vec3,
    pub val: i32,
}

impl SpiceNode {
    pub fn new(pos: Vec3) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Unique {
    pub faction: Faction,
    pub public: bool,
}

#[derive(Clone)]
pub struct Collider {
    pub shape: ShapeHandle<f32>,
}

impl Default for Collider {
    fn default() -> Self {
        Collider {
            shape: ShapeHandle::new(Cuboid::new(Vector3::new(0.5, 0.5, 0.5))),
        }
    }
}

#[derive(Bundle, Default)]
pub struct ColliderBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    collider: Collider,
    click_action: ClickAction,
    hover_action: HoverAction,
}

impl ColliderBundle {
    pub fn new(shape: ShapeHandle<f32>) -> Self {
        Self {
            collider: Collider { shape },
            ..Default::default()
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn with_click_action(mut self, action: Action) -> Self {
        self.click_action = self.click_action.with_base_action(action);
        self
    }

    pub fn with_hover_action(mut self, action: Action) -> Self {
        self.hover_action = self.hover_action.with_base_action(action);
        self
    }

    pub fn with_click_context(
        mut self,
        context: Context,
        generator: &'static ActionGenerator,
    ) -> Self {
        self.click_action = self.click_action.with_context(context, generator);
        self
    }

    pub fn with_hover_context(mut self, context: Context, action: Action) -> Self {
        self.hover_action = self.hover_action.with_context(context, action);
        self
    }
}

#[derive(Bundle)]
pub struct UniqueBundle {
    unique: Unique,
    visible: Visible,
}

impl UniqueBundle {
    pub fn new(faction: Faction) -> Self {
        Self {
            unique: Unique {
                faction,
                public: false,
            },
            visible: Visible {
                is_visible: false,
                ..Default::default()
            },
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Prediction {
    pub faction: Option<Faction>,
    pub turn: Option<i32>,
}

#[derive(Clone, Default)]
pub struct ClickAction {
    pub base_action: Option<Action>,
    pub contextual_actions: HashMap<Context, Box<&'static ActionGenerator>>,
    pub enabled: bool,
}

impl ClickAction {
    fn new(base_action: Action) -> Self {
        ClickAction {
            base_action: Some(base_action),
            contextual_actions: HashMap::new(),
            enabled: false,
        }
    }

    fn with_base_action(mut self, action: Action) -> Self {
        self.base_action = Some(action);
        self
    }

    fn with_context(mut self, context: Context, generator: &'static ActionGenerator) -> Self {
        self.contextual_actions.insert(context, Box::new(generator));
        self
    }
}

#[derive(Clone, Default)]
pub struct HoverAction {
    pub base_action: Option<Action>,
    pub contextual_actions: HashMap<Context, Action>,
    pub enabled: bool,
}

impl HoverAction {
    fn new(base_action: Action) -> Self {
        HoverAction {
            base_action: Some(base_action),
            contextual_actions: HashMap::new(),
            enabled: false,
        }
    }

    fn with_base_action(mut self, action: Action) -> Self {
        self.base_action = Some(action);
        self
    }

    fn with_context(mut self, context: Context, action: Action) -> Self {
        self.contextual_actions.insert(context, action);
        self
    }
}

pub struct Player {
    pub faction: Faction,
    pub traitor_cards: Vec<Entity>,
    pub treachery_cards: Vec<Entity>,
}

impl Player {
    pub fn new(faction: Faction, all_leaders: &Vec<Leader>) -> Self {
        Player {
            faction,
            traitor_cards: Vec::new(),
            treachery_cards: Vec::new(),
        }
    }
}
