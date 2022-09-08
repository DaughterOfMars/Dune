use std::{
    collections::{HashMap, HashSet},
    fs::File,
};

use bevy::{
    math::{Vec2, Vec3},
    prelude::Component,
};
use serde::{Deserialize, Serialize};

use crate::components::{CardEffect, Faction, Leader, Location, SpiceCard, Terrain, TreacheryCard, TreacheryCardKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Data {
    pub leaders: HashMap<Leader, LeaderData>,
    pub locations: HashMap<Location, LocationData>,
    pub factions: HashMap<Faction, FactionData>,
    pub treachery_cards: HashMap<TreacheryCardKind, TreacheryCardData>,
    pub treachery_deck: Vec<TreacheryCard>,
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
            treachery_deck: from_reader(File::open("data/treachery_deck.ron").unwrap()).unwrap(),
            spice_cards: from_reader(File::open("data/spice_cards.ron").unwrap()).unwrap(),
            camera_nodes: from_reader(File::open("data/camera_nodes.ron").unwrap()).unwrap(),
            prediction_nodes: from_reader(File::open("data/prediction_nodes.ron").unwrap()).unwrap(),
            traitor_nodes: from_reader(File::open("data/traitor_nodes.ron").unwrap()).unwrap(),
            token_nodes: from_reader(File::open("data/token_nodes.ron").unwrap()).unwrap(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct FactionStartingValues {
    pub units: u8,
    #[serde(default)]
    pub possible_locations: Option<HashSet<Location>>,
    pub spice: u8,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct FactionData {
    pub name: String,
    pub starting_values: FactionStartingValues,
    pub special_forces: u8,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct LeaderData {
    pub name: String,
    pub power: u8,
    pub faction: Faction,
    pub texture: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct LocationData {
    pub name: String,
    pub terrain: Terrain,
    pub spice: Option<Vec3>,
    pub sectors: HashMap<u8, LocationNodes>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct LocationNodes {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub fighters: Vec<Vec3>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CardEffectData {
    pub description: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct TreacheryCardData {
    pub name: String,
    pub effect: CardEffect,
    pub textures: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpiceCardData {
    pub name: String,
    #[serde(default)]
    pub location_data: Option<SpiceLocationData>,
    pub texture: String,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash)]
pub struct SpiceLocationData {
    pub location: Location,
    pub sector: u8,
    pub spice: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraNodeData {
    pub main: CameraNode,
    pub shield: CameraNode,
    pub board: CameraNode,
    pub treachery: CameraNode,
    pub traitor: CameraNode,
    pub spice: CameraNode,
    pub storm: CameraNode,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PredictionNodeData {
    pub src: Vec2,
    pub factions: Vec<Vec2>,
    pub turns: Vec<Vec2>,
    pub chosen_faction: Vec2,
    pub chosen_turn: Vec2,
}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug, Component)]
pub struct CameraNode {
    pub pos: Vec3,
    pub at: Vec3,
    pub up: Vec3,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TokenNodeData {
    pub leaders: Vec<Vec3>,
    pub spice: Vec<Vec3>,
    pub fighters: Vec<Vec3>,
    pub factions: Vec<Vec3>,
}
