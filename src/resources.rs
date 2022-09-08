use std::{collections::HashMap, fs::File};

use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

use crate::{
    components::{Faction, Leader, Location, SpiceCard, TreacheryCard},
    data::{
        CameraNodeData, FactionData, LeaderData, LocationData, PredictionNodeData, SpiceCardData, TokenNodeData,
        TreacheryCardData,
    },
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Data {
    pub leaders: HashMap<Leader, LeaderData>,
    pub locations: HashMap<Location, LocationData>,
    pub factions: HashMap<Faction, FactionData>,
    pub treachery_cards: HashMap<TreacheryCard, TreacheryCardData>,
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
            locations: from_reader(File::open("data/locations.ron").unwrap()).unwrap(),
            leaders: from_reader(File::open("data/leaders.ron").unwrap()).unwrap(),
            factions: from_reader(File::open("data/factions.ron").unwrap()).unwrap(),
            treachery_cards: from_reader(File::open("data/treachery_cards.ron").unwrap()).unwrap(),
            spice_cards: from_reader(File::open("data/spice_cards.ron").unwrap()).unwrap(),
            camera_nodes: from_reader(File::open("data/camera_nodes.ron").unwrap()).unwrap(),
            prediction_nodes: from_reader(File::open("data/prediction_nodes.ron").unwrap()).unwrap(),
            traitor_nodes: from_reader(File::open("data/traitor_nodes.ron").unwrap()).unwrap(),
            token_nodes: from_reader(File::open("data/token_nodes.ron").unwrap()).unwrap(),
        }
    }
}
