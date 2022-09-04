use std::{collections::HashMap, fs::File};

use bevy::{ecs::entity::Entity, math::Vec2};

use crate::{
    components::{Faction, Leader, Location, SpiceCard, TreacheryCard},
    data::{
        CameraNodeData, FactionData, LeaderData, LocationData, PredictionNodeData, SpiceCardData, TokenNodeData,
        TreacheryCardData, TreacheryDeckData,
    },
};

pub struct Data {
    pub leaders: HashMap<Leader, LeaderData>,
    pub locations: HashMap<Location, LocationData>,
    pub factions: HashMap<Faction, FactionData>,
    pub treachery_cards: HashMap<TreacheryCard, TreacheryCardData>,
    pub treachery_deck: Vec<TreacheryDeckData>,
    pub spice_cards: HashMap<SpiceCard, SpiceCardData>,
    pub camera_nodes: CameraNodeData,
    pub prediction_nodes: PredictionNodeData,
    pub traitor_nodes: Vec<Vec2>,
    pub token_nodes: TokenNodeData,
}

impl Default for Data {
    fn default() -> Self {
        use ron::de::from_reader;
        Data {
            locations: ron::de::from_reader(File::open("data/locations.ron").unwrap()).unwrap(),
            leaders: from_reader(File::open("data/leaders.ron").unwrap()).unwrap(),
            factions: from_reader(File::open("data/factions.ron").unwrap()).unwrap(),
            treachery_cards: from_reader(File::open("data/treachery_cards.ron").unwrap()).unwrap(),
            treachery_deck: from_reader(File::open("data/treachery_deck.ron").unwrap()).unwrap(),
            spice_cards: from_reader(File::open("data/spice_cards.ron").unwrap()).unwrap(),
            camera_nodes: from_reader(File::open("data/camera_nodes.ron").unwrap()).unwrap(),
            prediction_nodes: from_reader(File::open("data/prediction_nodes.ron").unwrap()).unwrap(),
            traitor_nodes: from_reader(File::open("data/traitor_nodes.ron").unwrap()).unwrap(),
            token_nodes: from_reader(File::open("data/token_nodes.ron").unwrap()).unwrap(),
        }
    }
}

pub struct Info {
    pub turn: i32,
    pub me: Option<Entity>,
    pub players: HashMap<Entity, String>,
    pub factions_in_play: Vec<Faction>,
    pub current_turn: usize,
    pub active_player: Option<Entity>,
    pub play_order: Vec<Entity>,
    pub default_clickables: Vec<Entity>,
}

impl Default for Info {
    fn default() -> Self {
        Info {
            turn: 0,
            me: None,
            players: HashMap::new(),
            factions_in_play: Vec::new(),
            current_turn: 0,
            active_player: None,
            play_order: Vec::new(),
            default_clickables: Vec::new(),
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
    }

    pub fn get_active_player(&self) -> Entity {
        self.active_player.unwrap_or(self.play_order[self.current_turn])
    }
}
