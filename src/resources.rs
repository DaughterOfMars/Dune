use std::fs::File;

use bevy::ecs::Entity;

use crate::{data::*, phase::Context};

pub struct Data {
    pub leaders: Vec<Leader>,
    pub locations: Vec<Location>,
    pub treachery_cards: Vec<TreacheryCard>,
    pub spice_cards: Vec<SpiceCard>,
    pub camera_nodes: CameraNodes,
    pub prediction_nodes: PredictionNodes,
    pub token_nodes: TokenNodes,
}

impl Data {
    pub fn init() -> Self {
        let locations = ron::de::from_reader(File::open("data/locations.ron").unwrap()).unwrap();
        let leaders = ron::de::from_reader(File::open("data/leaders.ron").unwrap()).unwrap();
        let treachery_cards =
            ron::de::from_reader(File::open("data/treachery.ron").unwrap()).unwrap();
        let spice_cards = ron::de::from_reader(File::open("data/spice.ron").unwrap()).unwrap();
        let camera_nodes =
            ron::de::from_reader(File::open("data/camera_nodes.ron").unwrap()).unwrap();
        let prediction_nodes =
            ron::de::from_reader(File::open("data/prediction_nodes.ron").unwrap()).unwrap();
        let token_nodes =
            ron::de::from_reader(File::open("data/token_nodes.ron").unwrap()).unwrap();
        Data {
            locations,
            leaders,
            treachery_cards,
            spice_cards,
            camera_nodes,
            prediction_nodes,
            token_nodes,
        }
    }
}

pub struct Info {
    pub turn: i32,
    pub factions_in_play: Vec<Faction>,
    pub current_turn: usize,
    pub active_player: Option<Entity>,
    pub play_order: Vec<Entity>,
    pub default_clickables: Vec<Entity>,
    pub context: Context,
}

impl Info {
    pub fn new() -> Self {
        Self {
            turn: 0,
            factions_in_play: Vec::new(),
            current_turn: 0,
            active_player: None,
            play_order: Vec::new(),
            default_clickables: Vec::new(),
            context: Context::None,
        }
    }
}

pub struct Collections {
    pub spice_bank: Vec<Entity>,
    pub treachery_deck: Vec<Entity>,
    pub treachery_discard: Vec<Entity>,
    pub traitor_deck: Vec<Entity>,
    pub traitor_discard: Vec<Entity>,
    pub spice_deck: Vec<Entity>,
    pub spice_discard: Vec<Entity>,
    pub storm_deck: Vec<Entity>,
}

impl Default for Collections {
    fn default() -> Self {
        Self {
            spice_bank: Vec::new(),
            treachery_deck: Vec::new(),
            treachery_discard: Vec::new(),
            traitor_deck: Vec::new(),
            traitor_discard: Vec::new(),
            spice_deck: Vec::new(),
            spice_discard: Vec::new(),
            storm_deck: Vec::new(),
        }
    }
}
