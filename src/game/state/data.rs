use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::{Deref, DerefMut};
use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

use super::{GameEvent, Object, ObjectId};
use crate::{
    components::{Bonus, Faction, Leader, Location, SpiceCard, StormCard, TraitorCard, TreacheryCard, Troop},
    game::phase::Phase,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameState {
    pub phase: Phase,
    pub game_turn: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_player: Option<PlayerId>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub players: HashMap<PlayerId, Player>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub play_order: Vec<PlayerId>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub factions: HashMap<Faction, PlayerId>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub prompts: HashMap<PlayerId, Prompt>,
    #[serde(skip_serializing_if = "Decks::is_empty")]
    pub decks: Decks,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub board: HashMap<Location, LocationState>,
    pub storm_sector: u8,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bidding_cards: BidStates,
    pub nexus: Option<Object<SpiceCard>>,
    pub bg_predictions: BeneGesseritPredictions,
    pub storm_card: Option<Object<StormCard>>,
    pub spice_card: Option<Object<SpiceCard>>,
    pub history: VecDeque<GameEvent>,
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize, Hash, From, Display)]
#[repr(transparent)]
pub struct PlayerId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub faction: Faction,
    pub treachery_cards: HashSet<Object<TreacheryCard>>,
    pub traitor_cards: HashSet<Object<TraitorCard>>,
    pub spice: u8,
    pub living_leaders: HashMap<Object<Leader>, bool>,
    pub offworld_forces: HashSet<Object<Troop>>,
    pub shipped: bool,
    pub tanks: TleilaxuTanks,
    pub bonuses: HashSet<Bonus>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bid {
    pub player_id: PlayerId,
    pub spice: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Deref, DerefMut)]
pub struct BidStates {
    pub bidding_cards: Vec<BidState>,
}

impl BidStates {
    pub fn current(&self) -> Option<&BidState> {
        self.bidding_cards.last()
    }

    pub fn win(&mut self) -> Option<BidState> {
        self.bidding_cards.pop()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BidState {
    pub card: Object<TreacheryCard>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_bid: Option<Bid>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Prompt {
    Faction { remaining: HashSet<Faction> },
    Traitor,
    FactionPrediction,
    TurnPrediction,
    GuildShip,
    Bid,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BeneGesseritPredictions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction: Option<Faction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deck<C> {
    pub cards: HashSet<Object<C>>,
    pub card_order: Vec<ObjectId>,
    pub discards: HashSet<Object<C>>,
    pub discard_order: Vec<ObjectId>,
}

impl<C> Deck<C> {
    pub fn add(&mut self, card: Object<C>) {
        self.card_order.push(card.id);
        self.cards.insert(card);
    }

    pub fn peek(&self) -> Option<&Object<C>> {
        self.card_order.last().and_then(|id| self.cards.get(id))
    }

    pub fn draw(&mut self) -> Option<Object<C>> {
        if let Some(id) = self.card_order.pop() {
            self.cards.take(&id)
        } else {
            None
        }
    }

    pub fn get(&self, id: ObjectId) -> Option<&Object<C>> {
        self.cards.get(&id).or(self.discards.get(&id))
    }

    pub fn last_discarded(&self) -> Option<&Object<C>> {
        self.discard_order.last().and_then(|id| self.discards.get(id))
    }

    pub fn discard(&mut self, card: Object<C>) {
        self.discard_order.push(card.id);
        self.discards.insert(card);
    }

    pub fn set_order(&mut self, order: Vec<ObjectId>) {
        self.card_order = order;
    }
}

impl<C> Default for Deck<C> {
    fn default() -> Self {
        Self {
            cards: Default::default(),
            card_order: Default::default(),
            discards: Default::default(),
            discard_order: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decks {
    pub traitor: Deck<TraitorCard>,
    pub treachery: Deck<TreacheryCard>,
    pub storm: Deck<StormCard>,
    pub spice: Deck<SpiceCard>,
}

impl Decks {
    fn is_empty(&self) -> bool {
        self.traitor.cards.is_empty()
            && self.traitor.discards.is_empty()
            && self.treachery.cards.is_empty()
            && self.treachery.discards.is_empty()
            && self.storm.cards.is_empty()
            && self.storm.discards.is_empty()
            && self.spice.cards.is_empty()
            && self.spice.discards.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeckType {
    Traitor,
    Treachery,
    Storm,
    Spice,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TleilaxuTanks {
    pub leaders: HashSet<Object<Leader>>,
    pub forces: HashSet<Object<Troop>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Forces {
    pub forces: HashSet<Object<Troop>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectorState {
    pub forces: HashMap<PlayerId, Forces>,
    pub spice: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationState {
    pub sectors: HashMap<u8, SectorState>,
    pub worm: Option<ObjectId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndGameReason {
    PlayerLeft { player_id: PlayerId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnType {
    Leader {
        player_id: PlayerId,
        leader: Object<Leader>,
    },
    Troop {
        player_id: PlayerId,
        unit: Object<Troop>,
    },
    TraitorCard(Object<TraitorCard>),
    TreacheryCard(Object<TreacheryCard>),
    SpiceCard(Object<SpiceCard>),
    StormCard(Object<StormCard>),
    Worm {
        location: Location,
        id: ObjectId,
    },
}
