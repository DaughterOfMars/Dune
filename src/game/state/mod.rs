mod data;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

pub use self::data::*;
use super::{Object, ObjectId};
use crate::{
    components::{Faction, Location, LocationSector, SpiceCard},
    data::Data,
    game::phase::{setup::SetupPhase, Phase},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent {
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
    StartRound,
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
    DealCard {
        player_id: PlayerId,
        from: DeckType,
    },
    DiscardCard {
        player_id: PlayerId,
        card_id: ObjectId,
        to: DeckType,
    },
    SetDeckOrder {
        deck_order: Vec<ObjectId>,
        deck_type: DeckType,
    },
    ChooseFaction {
        player_id: PlayerId,
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
        player_id: PlayerId,
        to: LocationSector,
        forces: HashSet<ObjectId>,
    },
    MoveForces {
        player_id: PlayerId,
        path: Vec<LocationSector>,
        forces: HashSet<ObjectId>,
    },
    RevealStorm,
    MoveStorm {
        sectors: u8,
    },
    RevealSpiceBlow,
    PlaceSpice {
        location: LocationSector,
        spice: u8,
    },
    RideTheWorm {
        location: Location,
    },
    StartBidding,
    MakeBid {
        player_id: PlayerId,
        spice: Option<u8>,
    },
    WinBid {
        player_id: PlayerId,
        card_id: ObjectId,
    },
    Revive {
        player_id: PlayerId,
        forces: HashSet<ObjectId>,
        leader: Option<ObjectId>,
    },
    SetBattlePlan {
        player_id: PlayerId,
        forces: u8,
        leader: Option<ObjectId>,
        treachery_cards: Vec<ObjectId>,
    },
}

impl EventReduce for GameState {
    type Event = GameEvent;

    fn validate(&self, data: &Data, event: &Self::Event) -> bool {
        use GameEvent::*;
        match event {
            Pass => {
                return self.play_order.len() == self.players.len();
            }
            ChooseFaction { player_id, .. } => {
                if matches!(self.phase, Phase::Setup(SetupPhase::ChooseFactions)) {
                    return Some(player_id) == self.active_player.as_ref();
                }
            }
            ChooseTraitor { player_id, card_id } => {
                if matches!(self.phase, Phase::Setup(SetupPhase::DealTraitors)) {
                    if let Some(player) = self.players.get(player_id) {
                        if player.traitor_cards.contains(card_id) {
                            return !matches!(player.faction, Faction::Harkonnen);
                        }
                    }
                }
            }
            MakeFactionPrediction { faction } => {
                return matches!(self.phase, Phase::Setup(SetupPhase::Prediction))
                    && self.factions.contains_key(&Faction::BeneGesserit)
                    && self.players.values().find(|p| p.faction == *faction).is_some();
            }
            MakeTurnPrediction { turn } => {
                return matches!(self.phase, Phase::Setup(SetupPhase::Prediction))
                    && self.factions.contains_key(&Faction::BeneGesserit)
                    && *turn < 15;
            }
            Bribe {
                player_id,
                other_player_id,
                spice,
            } => {
                todo!()
            }
            ShipForces { player_id, to, forces } => {
                if Some(player_id) == self.active_player.as_ref() {
                    let player = &self.players[player_id];
                    if forces.iter().all(|id| player.offworld_forces.contains(id)) {
                        if matches!(self.phase, Phase::Setup(SetupPhase::PlaceForces)) {
                            if let Some(possible_locations) =
                                &data.factions[&player.faction].starting_values.possible_locations
                            {
                                if possible_locations.contains(&to.location) {
                                    return true;
                                }
                            } else {
                                return true;
                            }
                        } else {
                            // TODO: validate ship n' move
                        }
                    }
                }
            }
            MoveForces {
                player_id,
                path,
                forces,
            } => {
                todo!()
            }
            MakeBid { player_id, spice } => {
                if Some(player_id) == self.active_player.as_ref() {
                    if let Some(bid_state) = self.bidding_cards.last() {
                        if let Some(current_bid) = &bid_state.current_bid {
                            if let Some(spice) = spice {
                                return *spice > current_bid.spice;
                            } else {
                                return true;
                            }
                        }
                    }
                }
            }
            WinBid { player_id, card_id } => {
                if let Some(bid_state) = self.bidding_cards.last() {
                    if bid_state.passed.len() == self.players.len() && !bid_state.passed.contains(player_id) {
                        if let Some(current_bid) = &bid_state.current_bid {
                            return &bid_state.card.id == card_id && &current_bid.player_id == player_id;
                        }
                    }
                }
            }
            Revive {
                player_id,
                forces,
                leader,
            } => {
                todo!()
            }
            SetBattlePlan {
                player_id,
                forces,
                leader,
                treachery_cards,
            } => {
                todo!()
            }

            // These events should only be created by the server, and are always invalid if coming from a client
            ShowPrompt { .. } => (),
            DealCard { .. } => (),
            // TODO: there may be situations where clients can send this event
            DiscardCard { .. } => (),
            SetActive { .. } => (),
            SetDeckOrder { .. } => (),
            EndGame { .. } => (),
            PlayerJoined { .. } => (),
            PlayerDisconnected { .. } => (),
            SetPlayOrder { .. } => (),
            AdvancePhase => (),
            StartBidding => (),
            RevealStorm => (),
            MoveStorm { .. } => (),
            RevealSpiceBlow => (),
            CollectSpice { .. } => (),
            SpawnObject { .. } => (),
            StartRound => (),
            PlaceSpice { .. } => (),
            RideTheWorm { .. } => (),
        }
        false
    }

    fn consume(&mut self, data: &Data, event: Self::Event) {
        use GameEvent::*;
        match &event {
            PlayerJoined { .. } | PlayerDisconnected { .. } => (),
            _ => {
                self.history.push_back(event.clone());
                if self.history.len() > 10 {
                    self.history.pop_front();
                }
            }
        }
        match event {
            EndGame { .. } => {
                self.phase = Phase::EndGame;
            }
            PlayerJoined { .. } => {}
            PlayerDisconnected { player_id } => {
                self.players.remove(&player_id);
            }
            ShowPrompt { prompt, player_id } => {
                self.prompts.insert(player_id, prompt);
            }
            AdvancePhase => {
                self.phase = self.phase.next();
                self.active_player.take();
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
                    self.decks.traitor.add(card);
                }
                SpawnType::TreacheryCard(card) => {
                    self.decks.treachery.add(card);
                }
                SpawnType::SpiceCard(card) => {
                    self.decks.spice.add(card);
                }
                SpawnType::StormCard(card) => {
                    self.decks.storm.add(card);
                }
                SpawnType::Worm { location, id } => {
                    self.board.get_mut(&location).unwrap().worm.replace(id);
                }
            },
            SetPlayOrder { play_order } => {
                self.play_order = play_order;
            }
            SetDeckOrder { deck_order, deck_type } => match deck_type {
                DeckType::Traitor => {
                    self.decks.traitor.set_order(deck_order);
                }
                DeckType::Treachery => {
                    self.decks.treachery.set_order(deck_order);
                }
                DeckType::Storm => {
                    self.decks.storm.set_order(deck_order);
                }
                DeckType::Spice => {
                    self.decks.spice.set_order(deck_order);
                }
            },
            ChooseFaction { player_id, faction } => {
                self.players.remove(&player_id);
                let faction_data = &data.factions[&faction];
                self.factions.insert(faction, player_id);
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
                        bonuses: Default::default(),
                    },
                );
                self.prompts.remove(&player_id);
            }
            ChooseTraitor { player_id, .. } => {
                self.prompts.remove(&player_id);
            }
            MakeFactionPrediction { faction } => {
                self.prompts.remove(&self.factions[&Faction::BeneGesserit]);
                self.bg_predictions.faction.replace(faction);
            }
            MakeTurnPrediction { turn } => {
                self.prompts.remove(&self.factions[&Faction::BeneGesserit]);
                self.bg_predictions.turn.replace(turn);
            }
            SetActive { player_id } => {
                self.active_player.replace(player_id);
            }
            Pass => {
                if let Some(player_id) = &self.active_player {
                    let current_turn = self.play_order.iter().position(|id| player_id == id).unwrap();
                    if current_turn + 1 == self.play_order.len() {
                        self.active_player.take();
                    } else {
                        self.active_player.replace(self.play_order[current_turn + 1]);
                    }
                }
            }
            StartRound => {
                self.active_player.replace(self.play_order[0]);
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
            ShipForces { player_id, to, forces } => {
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
                let player = self.players.get_mut(&player_id).unwrap();
                for force_id in forces {
                    sector.forces.insert(player.offworld_forces.take(&force_id).unwrap());
                }
                player.shipped = true;
            }
            MoveForces {
                player_id: _,
                path,
                forces,
            } => {
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
            RevealStorm => {
                self.storm_card.replace(self.decks.storm.draw().unwrap());
            }
            MoveStorm { sectors } => {
                self.storm_sector = (self.storm_sector + sectors) % 18;
                if let Some(storm_card) = self.storm_card.take() {
                    self.decks.storm.add(storm_card);
                }
            }
            RevealSpiceBlow => {
                let card = self.decks.spice.draw().unwrap();
                if let SpiceCard::ShaiHalud = &card.inner {
                    if self.game_turn > 0 && self.nexus.is_none() {
                        self.nexus = self.decks.spice.last_discarded().cloned();
                    }
                }
                if let Some(old_card) = self.spice_card.replace(card) {
                    self.decks.spice.discard(old_card);
                }
            }
            PlaceSpice {
                location: LocationSector { location, sector },
                spice,
            } => {
                if let Some(spice_card) = self.spice_card.take() {
                    self.decks.spice.discard(spice_card);
                }
                if self.storm_sector != sector {
                    self.board
                        .entry(location)
                        .or_default()
                        .sectors
                        .entry(sector)
                        .or_default()
                        .spice += spice;
                }
            }
            RideTheWorm { location } => {
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
            StartBidding => {
                for _ in 0..self.players.len() {
                    if let Some(card) = self.decks.treachery.draw() {
                        self.bidding_cards.push(BidState {
                            card,
                            current_bid: Default::default(),
                            passed: Default::default(),
                        });
                    }
                }
            }
            MakeBid { player_id, spice } => {
                if let Some(bid_state) = self.bidding_cards.last_mut() {
                    if let Some(spice) = spice {
                        bid_state.current_bid.replace(Bid { player_id, spice });
                        bid_state.passed.remove(&player_id);
                    } else {
                        bid_state.passed.insert(player_id);
                    }
                }
            }
            WinBid { player_id, card_id } => {
                let bid_state = self.bidding_cards.pop().unwrap();
                self.players
                    .get_mut(&player_id)
                    .unwrap()
                    .treachery_cards
                    .insert(bid_state.card);
            }
            Revive {
                player_id,
                forces,
                leader,
            } => {
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
                player_id,
                forces,
                leader,
                treachery_cards,
            } => todo!(),
            DealCard { player_id, from } => {
                let player = self.players.get_mut(&player_id).unwrap();
                match from {
                    DeckType::Traitor => {
                        if let Some(card) = self.decks.traitor.draw() {
                            player.traitor_cards.insert(card);
                        }
                    }
                    DeckType::Treachery => {
                        if let Some(card) = self.decks.treachery.draw() {
                            player.treachery_cards.insert(card);
                        }
                    }
                    _ => unreachable!(),
                }
            }
            DiscardCard { player_id, card_id, to } => {
                let player = self.players.get_mut(&player_id).unwrap();
                match to {
                    DeckType::Traitor => {
                        if let Some(card) = player.traitor_cards.take(&card_id) {
                            self.decks.traitor.discard(card);
                        }
                    }
                    DeckType::Treachery => {
                        if let Some(card) = player.treachery_cards.take(&card_id) {
                            self.decks.treachery.discard(card);
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

    fn validate(&self, data: &Data, event: &Self::Event) -> bool;

    fn consume(&mut self, data: &Data, event: Self::Event);
}
