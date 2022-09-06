use std::collections::{HashMap, HashSet};

use bevy::{
    math::{Vec2, Vec3},
    prelude::Component,
};
use serde::{Deserialize, Serialize};

use crate::components::{CardEffect, Faction, Location, Terrain, TreacheryCard};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FactionStartingValues {
    pub units: u8,
    #[serde(default)]
    pub possible_locations: Option<HashSet<Location>>,
    pub spice: u8,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FactionData {
    pub name: String,
    pub starting_values: FactionStartingValues,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LeaderData {
    pub name: String,
    pub power: u8,
    pub faction: Faction,
    pub texture: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LocationData {
    pub name: String,
    pub terrain: Terrain,
    pub spice: Option<Vec3>,
    pub sectors: HashMap<i32, LocationNodes>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LocationNodes {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub fighters: Vec<Vec3>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CardEffectData {
    pub description: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TreacheryCardData {
    pub name: String,
    pub effect: CardEffect,
}

#[derive(Clone, Serialize, Deserialize, Debug, Component)]
pub struct TreacheryDeckData {
    pub card: TreacheryCard,
    pub texture: String,
}

#[derive(Clone, Serialize, Deserialize, Component)]
pub struct SpiceCardData {
    pub name: String,
    #[serde(default)]
    pub location: Option<Location>,
    pub texture: String,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct CameraNodeData {
    pub main: CameraNode,
    pub shield: CameraNode,
    pub board: CameraNode,
    pub treachery: CameraNode,
    pub traitor: CameraNode,
    pub spice: CameraNode,
    pub storm: CameraNode,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PredictionNodeData {
    pub src: Vec2,
    pub factions: Vec<Vec2>,
    pub turns: Vec<Vec2>,
    pub chosen_faction: Vec2,
    pub chosen_turn: Vec2,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, Component)]
pub struct CameraNode {
    pub pos: Vec3,
    pub at: Vec3,
    pub up: Vec3,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TokenNodeData {
    pub leaders: Vec<Vec3>,
    pub spice: Vec<Vec3>,
    pub fighters: Vec<Vec3>,
    pub factions: Vec<Vec3>,
}
