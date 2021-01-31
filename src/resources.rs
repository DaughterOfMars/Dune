use std::fs::File;

use bevy::{ecs::Entity, math::Vec2};

use crate::{data::*, phase::actions::Context};

pub(crate) struct Data {
    pub leaders: Vec<Leader>,
    pub locations: Vec<Location>,
    pub treachery_cards: Vec<TreacheryCard>,
    pub spice_cards: Vec<SpiceCard>,
    pub camera_nodes: CameraNodes,
    pub prediction_nodes: PredictionNodes,
    pub traitor_nodes: Vec<Vec2>,
    pub token_nodes: TokenNodes,
    pub ui_structure: UiStructure,
}

impl Default for Data {
    fn default() -> Self {
        let locations = ron::de::from_reader(File::open("data/locations.ron").unwrap()).unwrap();
        let leaders = ron::de::from_reader(File::open("data/leaders.ron").unwrap()).unwrap();
        let treachery_cards =
            ron::de::from_reader(File::open("data/treachery.ron").unwrap()).unwrap();
        let spice_cards = ron::de::from_reader(File::open("data/spice.ron").unwrap()).unwrap();
        let camera_nodes =
            ron::de::from_reader(File::open("data/camera_nodes.ron").unwrap()).unwrap();
        let prediction_nodes =
            ron::de::from_reader(File::open("data/prediction_nodes.ron").unwrap()).unwrap();
        let traitor_nodes =
            ron::de::from_reader(File::open("data/traitor_nodes.ron").unwrap()).unwrap();
        let token_nodes =
            ron::de::from_reader(File::open("data/token_nodes.ron").unwrap()).unwrap();
        let ui_structure = ron::de::from_reader(File::open("data/ui.ron").unwrap()).unwrap();
        Data {
            locations,
            leaders,
            treachery_cards,
            spice_cards,
            camera_nodes,
            prediction_nodes,
            traitor_nodes,
            token_nodes,
            ui_structure,
        }
    }
}

pub(crate) struct Info {
    pub turn: i32,
    pub me: Option<Entity>,
    pub players: Vec<String>,
    pub factions_in_play: Vec<Faction>,
    pub current_turn: usize,
    pub active_player: Option<Entity>,
    pub play_order: Vec<Entity>,
    pub default_clickables: Vec<Entity>,
    pub context: Context,
}

impl Default for Info {
    fn default() -> Self {
        Info {
            turn: 0,
            me: None,
            players: Vec::new(),
            factions_in_play: Vec::new(),
            current_turn: 0,
            active_player: None,
            play_order: Vec::new(),
            default_clickables: Vec::new(),
            context: Context::None,
        }
    }
}

impl Info {
    pub fn reset(&mut self) {
        self.turn = 0;
        self.me = None;
        self.factions_in_play = Vec::new();
        self.current_turn = 0;
        self.active_player = None;
        self.play_order = Vec::new();
        self.default_clickables = Vec::new();
        self.context = Context::None;
    }

    pub fn get_active_player(&self) -> Entity {
        self.active_player
            .unwrap_or(self.play_order[self.current_turn])
    }
}
