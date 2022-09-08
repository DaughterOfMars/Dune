use std::collections::{HashMap, HashSet, VecDeque};

use derive_more::{Display, From};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use super::{Object, ObjectId};
use crate::{
    components::{
        Bonus, Faction, Leader, Location, LocationSector, SpiceCard, StormCard, TraitorCard, TreacheryCard, Troop,
    },
    data::SpiceLocationData,
    game::{Phase, SetupPhase},
    resources::Data,
};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Prompt>,
    pub bonuses: HashSet<Bonus>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnpickedPlayer {
    pub prompt: Option<Prompt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deck<C> {
    pub cards: Vec<Object<C>>,
    pub discard: Vec<Object<C>>,
}

impl<C> Default for Deck<C> {
    fn default() -> Self {
        Self {
            cards: Default::default(),
            discard: Default::default(),
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent {
    StartGame,
    EndGame {
        reason: EndGameReason,
    },
    PlayerJoined {
        player_id: PlayerId,
    },
    PlayerDisconnected {
        player_id: PlayerId,
    },
    SetActive {
        player_id: PlayerId,
    },
    Pass,
    AdvancePhase,
    SpawnObject {
        spawn_type: SpawnType,
    },
    ShowPrompt {
        player_id: PlayerId,
        prompt: Prompt,
    },
    SetPlayOrder {
        play_order: Vec<PlayerId>,
    },
    DealCards {
        player_id: PlayerId,
        from: DeckType,
        count: usize,
    },
    ShuffleDeck {
        deck_type: DeckType,
    },
    ChooseFaction {
        faction: Faction,
    },
    ChooseTraitor {
        player_id: PlayerId,
        card_id: ObjectId,
    },
    MakeFactionPrediction {
        faction: Faction,
    },
    MakeTurnPrediction {
        turn: u8,
    },
    CollectSpice {
        player_id: PlayerId,
        spice: u8,
        from: Option<LocationSector>,
    },
    Bribe {
        player_id: PlayerId,
        other_player_id: PlayerId,
        spice: u8,
    },
    ShipForces {
        to: LocationSector,
        forces: HashSet<ObjectId>,
    },
    MoveForces {
        path: Vec<LocationSector>,
        forces: HashSet<ObjectId>,
    },
    AdvanceStorm {
        sectors: u8,
    },
    SpiceBlow,
    StartBidding,
    MakeBid {
        spice: Option<u8>,
    },
    Revive {
        forces: HashSet<ObjectId>,
        leader: Option<ObjectId>,
    },
    SetBattlePlan {
        player: PlayerId,
        forces: u8,
        leader: Option<ObjectId>,
        treachery_cards: Vec<ObjectId>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bid {
    pub player_id: PlayerId,
    pub spice: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BidState {
    pub card: Object<TreacheryCard>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_bid: Option<Bid>,
    pub passed: HashSet<PlayerId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Prompt {
    Faction,
    Traitor,
    FactionPrediction,
    TurnPrediction,
    GuildShip,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BeneGesseritPredictions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction: Option<Faction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn: Option<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameState {
    pub phase: Phase,
    pub game_turn: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_player: Option<PlayerId>,
    pub unpicked_players: HashMap<PlayerId, UnpickedPlayer>,
    pub players: HashMap<PlayerId, Player>,
    pub play_order: Vec<PlayerId>,
    pub decks: Decks,
    pub board: HashMap<Location, LocationState>,
    pub storm_sector: u8,
    pub bidding_cards: Vec<BidState>,
    pub nexus: bool,
    pub bg_predictions: BeneGesseritPredictions,
    pub history: VecDeque<GameEvent>,
    #[serde(skip)]
    pub data: Data,
}

impl EventReduce for GameState {
    type Event = GameEvent;

    fn validate(&self, event: &Self::Event) -> bool {
        use GameEvent::*;
        match event {
            StartGame => self.unpicked_players.len() > 1 && self.players.is_empty(),
            Pass => self.play_order.len() == self.players.len(),
            ChooseFaction { .. } => {
                if matches!(self.phase, Phase::Setup(SetupPhase::ChooseFactions)) {
                    if let Some(ref player_id) = self.active_player {
                        return self.unpicked_players.contains_key(player_id);
                    }
                }
                false
            }
            ChooseTraitor { player_id, card_id } => {
                if matches!(self.phase, Phase::Setup(SetupPhase::DealTraitors)) {
                    if let Some(player) = self.players.get(player_id) {
                        if player.traitor_cards.contains(card_id) {
                            return !matches!(player.faction, Faction::Harkonnen);
                        }
                    }
                }
                false
            }
            MakeFactionPrediction { faction } => {
                matches!(self.phase, Phase::Setup(SetupPhase::Prediction))
                    && self.players.values().find(|p| p.faction == *faction).is_some()
            }
            MakeTurnPrediction { turn } => matches!(self.phase, Phase::Setup(SetupPhase::Prediction)) && *turn < 15,
            Bribe {
                player_id,
                other_player_id,
                spice,
            } => todo!(),
            ShipForces { to, forces } => todo!(),
            MoveForces { path, forces } => todo!(),
            MakeBid { spice } => todo!(),
            Revive { forces, leader } => todo!(),
            SetBattlePlan {
                player,
                forces,
                leader,
                treachery_cards,
            } => todo!(),

            // These events should only be created by the server, and are always invalid if coming from a client
            ShowPrompt { .. }
            | DealCards { .. }
            | SetActive { .. }
            | ShuffleDeck { .. }
            | EndGame { .. }
            | PlayerJoined { .. }
            | PlayerDisconnected { .. }
            | SetPlayOrder { .. }
            | AdvancePhase
            | StartBidding
            | AdvanceStorm { .. }
            | SpiceBlow
            | CollectSpice { .. }
            | SpawnObject { .. } => false,
        }
    }

    fn consume(&mut self, event: Self::Event) {
        use GameEvent::*;
        match &event {
            PlayerDisconnected { .. } => (),
            _ => {
                self.history.push_back(event.clone());
                if self.history.len() > 10 {
                    self.history.pop_front();
                }
            }
        }
        match event {
            StartGame => {}
            EndGame { .. } => {
                self.phase = Phase::EndGame;
            }
            PlayerJoined { player_id } => {
                self.unpicked_players.insert(player_id, Default::default());
            }
            PlayerDisconnected { player_id } => {
                self.players.remove(&player_id);
            }
            ShowPrompt { prompt, player_id } => {
                self.players.get_mut(&player_id).unwrap().prompt.replace(prompt);
            }
            AdvancePhase => {
                self.phase = self.phase.next();
            }
            SpawnObject { spawn_type } => match spawn_type {
                SpawnType::Leader { player_id, leader } => {
                    self.players
                        .get_mut(&player_id)
                        .unwrap()
                        .living_leaders
                        .insert(leader, false);
                }
                SpawnType::Troop { player_id, unit } => {
                    self.players.get_mut(&player_id).unwrap().offworld_forces.insert(unit);
                }
                SpawnType::TraitorCard(card) => {
                    self.decks.traitor.cards.push(card);
                }
                SpawnType::TreacheryCard(card) => {
                    self.decks.treachery.cards.push(card);
                }
                SpawnType::SpiceCard(card) => {
                    self.decks.spice.cards.push(card);
                }
                SpawnType::StormCard(card) => {
                    self.decks.storm.cards.push(card);
                }
                SpawnType::Worm { location, id } => {
                    self.board.get_mut(&location).unwrap().worm.replace(id);
                }
            },
            SetPlayOrder { play_order } => {
                self.play_order = play_order;
            }
            ShuffleDeck { deck_type } => {
                let mut rng = rand::thread_rng();
                match deck_type {
                    DeckType::Traitor => self.decks.traitor.cards.shuffle(&mut rng),
                    DeckType::Treachery => self.decks.treachery.cards.shuffle(&mut rng),
                    DeckType::Storm => self.decks.storm.cards.shuffle(&mut rng),
                    DeckType::Spice => self.decks.spice.cards.shuffle(&mut rng),
                }
            }
            ChooseFaction { faction } => {
                let player_id = self.active_player.unwrap();
                self.unpicked_players.remove(&player_id);
                let faction_data = &self.data.factions[&faction];
                self.players.insert(
                    player_id,
                    Player {
                        faction,
                        spice: faction_data.starting_values.spice,
                        treachery_cards: Default::default(),
                        traitor_cards: Default::default(),
                        living_leaders: Default::default(),
                        offworld_forces: Default::default(),
                        shipped: Default::default(),
                        tanks: Default::default(),
                        prompt: Default::default(),
                        bonuses: Default::default(),
                    },
                );
            }
            ChooseTraitor { player_id, card_id } => {
                let player = self.players.get_mut(&player_id).unwrap();
                player
                    .traitor_cards
                    .drain_filter(|card| card.id != card_id)
                    .for_each(|card| self.decks.traitor.discard.push(card));
                player.prompt.take();
            }
            MakeFactionPrediction { faction } => {
                self.players
                    .values_mut()
                    .find(|p| p.faction == Faction::BeneGesserit)
                    .unwrap()
                    .prompt
                    .take();
                self.bg_predictions.faction.replace(faction);
            }
            MakeTurnPrediction { turn } => {
                self.players
                    .values_mut()
                    .find(|p| p.faction == Faction::BeneGesserit)
                    .unwrap()
                    .prompt
                    .take();
                self.bg_predictions.turn.replace(turn);
            }
            SetActive { player_id } => {
                self.active_player.replace(player_id);
            }
            Pass => {
                if let Some(active_player) = self.active_player.as_mut() {
                    let current_turn = self.play_order.iter().position(|id| active_player == id).unwrap();
                    *active_player = self.play_order[(current_turn + 1) % self.play_order.len()];
                } else {
                    self.active_player.replace(self.play_order[0]);
                }
            }
            CollectSpice { player_id, spice, from } => {
                if let Some(from) = from {
                    self.board
                        .entry(from.location)
                        .or_default()
                        .sectors
                        .entry(from.sector)
                        .or_default()
                        .spice -= spice;
                }
                self.players.get_mut(&player_id).unwrap().spice += spice;
            }
            Bribe {
                player_id,
                other_player_id,
                spice,
            } => {
                self.players.get_mut(&player_id).unwrap().spice -= spice;
                self.players.get_mut(&other_player_id).unwrap().spice += spice;
            }
            ShipForces { to, forces } => {
                let sector = self
                    .board
                    .entry(to.location)
                    .or_default()
                    .sectors
                    .entry(to.sector)
                    .or_default()
                    .forces
                    .entry(self.active_player.unwrap())
                    .or_default();
                let player = self.players.get_mut(self.active_player.as_ref().unwrap()).unwrap();
                for force_id in forces {
                    sector.forces.insert(player.offworld_forces.take(&force_id).unwrap());
                }
                player.shipped = true;
            }
            MoveForces { path, forces } => {
                let (from, to) = (path.first().unwrap(), path.last().unwrap());
                let from = self
                    .board
                    .get_mut(&from.location)
                    .unwrap()
                    .sectors
                    .get_mut(&from.sector)
                    .unwrap()
                    .forces
                    .get_mut(self.active_player.as_ref().unwrap())
                    .unwrap();
                let forces = forces
                    .into_iter()
                    .map(|id| from.forces.take(&id).unwrap())
                    .collect::<HashSet<_>>();
                self.board
                    .entry(to.location)
                    .or_default()
                    .sectors
                    .entry(to.sector)
                    .or_default()
                    .forces
                    .entry(self.active_player.unwrap())
                    .or_default()
                    .forces
                    .extend(forces);
            }
            AdvanceStorm { sectors } => {
                self.storm_sector = (self.storm_sector + sectors) % 18;
            }
            SpiceBlow => {
                let mut nexus = None;
                loop {
                    let card = self.decks.spice.cards.pop().unwrap();
                    match card.inner {
                        SpiceCard::ShaiHalud => {
                            nexus = self.decks.spice.discard.last();
                        }
                        _ => {
                            if let Some(nexus_card) = nexus {
                                self.nexus = true;
                                let SpiceLocationData { location, .. } =
                                    self.data.spice_cards[&nexus_card.inner].location_data.unwrap();
                                for forces in self
                                    .board
                                    .get_mut(&location)
                                    .unwrap()
                                    .sectors
                                    .drain()
                                    .map(|(_, s)| s.forces)
                                {
                                    for (player_id, Forces { forces }) in forces {
                                        let tanks = &mut self.players.get_mut(&player_id).unwrap().tanks;
                                        tanks.forces.extend(forces);
                                    }
                                }
                            }
                            let SpiceLocationData {
                                location,
                                sector,
                                spice,
                            } = self.data.spice_cards[&card.inner].location_data.unwrap();
                            if self.storm_sector != sector {
                                self.board
                                    .entry(location)
                                    .or_default()
                                    .sectors
                                    .entry(sector)
                                    .or_default()
                                    .spice += spice;
                            }
                            self.decks.spice.discard.push(card);
                            break;
                        }
                    }
                }
            }
            StartBidding => {
                for _ in 0..self.players.len() {
                    if let Some(card) = self.decks.treachery.cards.pop() {
                        self.bidding_cards.push(BidState {
                            card,
                            current_bid: Default::default(),
                            passed: Default::default(),
                        });
                    }
                }
            }
            MakeBid { spice } => {
                let player_id = self.active_player.unwrap();
                if let Some(spice) = spice {
                    self.bidding_cards
                        .first_mut()
                        .unwrap()
                        .current_bid
                        .replace(Bid { player_id, spice });
                } else {
                    self.bidding_cards.first_mut().unwrap().passed.insert(player_id);
                }
            }
            Revive { forces, leader } => {
                let player = self.players.get_mut(self.active_player.as_ref().unwrap()).unwrap();
                if let Some(leader) = leader {
                    player
                        .living_leaders
                        .insert(player.tanks.leaders.take(&leader).unwrap(), true);
                }
                player
                    .offworld_forces
                    .extend(player.tanks.forces.drain_filter(|f| forces.contains(&f.id)));
            }
            SetBattlePlan {
                player,
                forces,
                leader,
                treachery_cards,
            } => todo!(),
            DealCards { player_id, from, count } => {
                let player = self.players.get_mut(&player_id).unwrap();
                match from {
                    DeckType::Traitor => {
                        for _ in 0..count {
                            if let Some(card) = self.decks.traitor.cards.pop() {
                                player.traitor_cards.insert(card);
                            }
                        }
                    }
                    DeckType::Treachery => {
                        for _ in 0..count {
                            if let Some(card) = self.decks.treachery.cards.pop() {
                                player.treachery_cards.insert(card);
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}

pub trait EventReduce {
    type Event;

    fn validate(&self, event: &Self::Event) -> bool;

    fn consume(&mut self, event: Self::Event);
}
