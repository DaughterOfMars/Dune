use std::collections::HashSet;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use super::*;
use crate::{
    components::{Faction, Leader, TraitorCard, Troop},
    game::{
        phase::{setup::SetupPhase, Phase},
        state::{DeckType, Prompt, SpawnType},
        Object, ObjectIdGenerator,
    },
};

pub fn spawn_server(commands: &mut Commands) {
    commands.insert_resource(RenetServer {
        handle: Some(std::thread::spawn(server)),
    });
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerEvent {
    LoadAssets,
    StartGame,
}

pub struct Server {
    renet_server: renet::RenetServer,
    state: GameState,
    waiting_players: HashSet<PlayerId>,
    ready_players: HashSet<PlayerId>,
    ids: ObjectIdGenerator,
}

impl Server {
    /// This is the server logic, which is run whenever the game state changes.
    fn game_logic(&mut self, last_event: GameEvent) -> Result<(), RenetNetworkingError> {
        use GameEvent::*;
        match last_event {
            AdvancePhase => match &self.state.phase {
                Phase::Setup(s) => match s {
                    SetupPhase::ChooseFactions => {
                        // TODO: Perhaps allow other ways to determine play order
                        let mut play_order = self.ready_players.drain().collect::<Vec<_>>();
                        let mut rng = rand::thread_rng();
                        play_order.shuffle(&mut rng);
                        self.generate(SetPlayOrder { play_order })?;
                        self.generate(StartRound)?;
                    }
                    SetupPhase::Prediction => {
                        if let Some(bg_player) = self.state.factions.get(&Faction::BeneGesserit).copied() {
                            self.generate(SetActive { player_id: bg_player })?;
                            self.generate(ShowPrompt {
                                player_id: bg_player,
                                prompt: Prompt::FactionPrediction,
                            })?;
                        } else {
                            self.generate(AdvancePhase)?;
                        }
                    }
                    SetupPhase::AtStart => {
                        for card in self.state.data.treachery_deck.clone() {
                            let card = self.spawn(card);
                            self.generate(SpawnObject {
                                spawn_type: SpawnType::TreacheryCard(card),
                            })?;
                        }
                        for leader in Leader::iter() {
                            let faction = self.state.data.leaders[&leader].faction;
                            if self.state.factions.contains_key(&faction) {
                                let card = self.spawn(TraitorCard { leader });
                                self.generate(SpawnObject {
                                    spawn_type: SpawnType::TraitorCard(card),
                                })?;
                            }
                        }
                        let mut rng = rand::thread_rng();
                        let mut deck_order = self
                            .state
                            .decks
                            .traitor
                            .cards
                            .iter()
                            .map(|card| card.id)
                            .collect::<Vec<_>>();
                        deck_order.shuffle(&mut rng);
                        self.generate(SetDeckOrder {
                            deck_order,
                            deck_type: DeckType::Traitor,
                        })?;
                        let mut deck_order = self
                            .state
                            .decks
                            .treachery
                            .cards
                            .iter()
                            .map(|card| card.id)
                            .collect::<Vec<_>>();
                        deck_order.shuffle(&mut rng);
                        self.generate(SetDeckOrder {
                            deck_order,
                            deck_type: DeckType::Treachery,
                        })?;
                        self.generate(AdvancePhase)?;
                    }
                    SetupPhase::DealTraitors => {
                        for player_id in std::iter::repeat(self.state.play_order.clone()).take(4).flatten() {
                            self.generate(DealCard {
                                player_id,
                                from: DeckType::Traitor,
                            })?;
                        }
                        for player_id in self.state.play_order.clone() {
                            if !matches!(self.state.players[&player_id].faction, Faction::Harkonnen) {
                                self.generate(ShowPrompt {
                                    player_id,
                                    prompt: Prompt::Traitor,
                                })?;
                            }
                        }
                    }
                    SetupPhase::PlaceForces => {
                        self.generate(StartRound)?;
                    }
                    SetupPhase::DealTreachery => {
                        for player_id in self.state.play_order.clone() {
                            self.generate(DealCard {
                                player_id,
                                from: DeckType::Treachery,
                            })?;
                        }
                        // Harkonnen gets two
                        if let Some(hk_player) = self.state.factions.get(&Faction::Harkonnen).copied() {
                            self.generate(DealCard {
                                player_id: hk_player,
                                from: DeckType::Treachery,
                            })?;
                        }
                        self.generate(AdvancePhase)?;
                    }
                },
                _ => (),
            },
            StartRound | Pass => match self.state.phase {
                Phase::Setup(s) => match s {
                    SetupPhase::ChooseFactions => {
                        if let Some(player_id) = self.state.active_player {
                            self.generate(ShowPrompt {
                                player_id,
                                prompt: Prompt::Faction,
                            })?;
                        } else {
                            self.generate(AdvancePhase)?;
                        }
                    }
                    SetupPhase::DealTraitors => {
                        if self.state.prompts.is_empty() {
                            self.generate(AdvancePhase)?;
                        }
                    }
                    SetupPhase::PlaceForces => {
                        if let Some(player_id) = self.state.active_player {
                            if self.state.data.factions[&self.state.players[&player_id].faction]
                                .starting_values
                                .units
                                == 0
                            {
                                self.generate(Pass)?;
                            }
                        } else {
                            self.generate(AdvancePhase)?;
                        }
                    }
                    _ => (),
                },
                _ => (),
            },
            ChooseFaction { faction } => {
                for leader in self
                    .state
                    .data
                    .leaders
                    .clone()
                    .into_iter()
                    .filter_map(|(leader, data)| (data.faction == faction).then_some(leader))
                {
                    let leader = self.spawn(leader);
                    self.generate(SpawnObject {
                        spawn_type: SpawnType::Leader {
                            player_id: self.state.active_player.unwrap(),
                            leader,
                        },
                    })?;
                }
                for unit in std::iter::repeat_with(|| Troop { is_special: false })
                    .take(20 - self.state.data.factions[&faction].special_forces as usize)
                    .chain(
                        std::iter::repeat_with(|| Troop { is_special: true })
                            .take(self.state.data.factions[&faction].special_forces as usize),
                    )
                {
                    let unit = self.spawn(unit);
                    self.generate(SpawnObject {
                        spawn_type: SpawnType::Troop {
                            player_id: self.state.active_player.unwrap(),
                            unit,
                        },
                    })?;
                }
                self.generate(Pass)?;
            }
            ChooseTraitor { player_id, card_id } => {
                // Discard the cards that weren't picked
                for card_id in self.state.players[&player_id]
                    .traitor_cards
                    .iter()
                    .filter_map(|card| (card.id != card_id).then_some(card.id))
                    .collect::<Vec<_>>()
                {
                    self.generate(DiscardCard {
                        player_id,
                        card_id,
                        to: DeckType::Traitor,
                    })?;
                }
                self.generate(Pass)?;
            }
            MakeFactionPrediction { .. } => {
                self.generate(ShowPrompt {
                    player_id: self.state.active_player.unwrap(),
                    prompt: Prompt::TurnPrediction,
                })?;
            }
            MakeTurnPrediction { .. } => {
                self.generate(AdvancePhase)?;
            }
            ShipForces { .. } => {
                if matches!(self.state.phase, Phase::Setup(SetupPhase::PlaceForces)) {
                    if let Some(player_id) = &self.state.active_player {
                        let player = &self.state.players[player_id];
                        let faction_data = &self.state.data.factions[&player.faction];
                        if player.offworld_forces.len() == 20 - faction_data.starting_values.units as usize {
                            self.generate(Pass)?;
                        }
                    }
                } else {
                    // TODO: shipping during ship n' move
                }
            }
            _ => (),
        }
        Ok(())
    }

    /// Consume an event and broadcast it to all clients.
    fn generate(&mut self, event: GameEvent) -> Result<(), RenetNetworkingError> {
        let serialized_event = bincode::serialize(&event)?;
        self.state.consume(event.clone());
        self.renet_server.broadcast_message(0, serialized_event);
        self.game_logic(event)?;
        Ok(())
    }

    /// Process the current buffer of events.
    fn process_events(&mut self) -> Result<(), RenetNetworkingError> {
        // Receive connection events from clients
        while let Some(event) = self.renet_server.get_event() {
            match event {
                renet::ServerEvent::ClientConnected(id, ..) => {
                    self.waiting_players.insert(id.into());
                    let event = GameEvent::PlayerJoined { player_id: id.into() };
                    // Tell the recently joined player about the other players
                    for player_id in self.waiting_players.iter() {
                        let event = GameEvent::PlayerJoined { player_id: *player_id };
                        self.renet_server.send_message(id, 0, bincode::serialize(&event)?);
                    }

                    // Add the new player to the game
                    self.generate(event)?;

                    info!("Client {} connected.", id);
                }
                renet::ServerEvent::ClientDisconnected(id) => {
                    let player_id = id.into();
                    self.waiting_players.remove(&player_id);
                    self.ready_players.remove(&player_id);
                    self.generate(GameEvent::PlayerDisconnected { player_id })?;
                    info!("Client {} disconnected", id);

                    // Then end the game
                    self.generate(GameEvent::EndGame {
                        reason: EndGameReason::PlayerLeft { player_id: id.into() },
                    })?;

                    // NOTE: Since we don't authenticate users we can't do any reconnection attempts.
                    // We simply have no way to know if the next user is the same as the one that disconnected.
                }
            }
        }

        // Receive GameEvents from clients. Consume valid events.
        for client_id in self.renet_server.clients_id().into_iter() {
            while let Some(message) = self.renet_server.receive_message(client_id, 0) {
                if let Ok(event) = bincode::deserialize::<ServerEvent>(&message) {
                    match &event {
                        ServerEvent::LoadAssets | ServerEvent::StartGame => {
                            if self.waiting_players.len() + self.ready_players.len() < 2 {
                                warn!("Player {} sent invalid event:\n\t{:#?}", client_id, event);
                                continue;
                            }
                        }
                    }
                    if let ServerEvent::StartGame = &event {
                        if let Some(player_id) = self.waiting_players.take(&client_id.into()) {
                            self.ready_players.insert(player_id);
                            if self.waiting_players.len() == 0 {
                                self.generate(GameEvent::AdvancePhase)?;
                            }
                        } else {
                            warn!("Player {} sent invalid event:\n\t{:#?}", client_id, event);
                        }
                    }
                    let serialized_event = bincode::serialize(&event)?;
                    self.renet_server.broadcast_message(0, serialized_event);
                } else if let Ok(event) = bincode::deserialize::<GameEvent>(&message) {
                    if self.state.validate(&event) {
                        trace!("Player {} sent:\n\t{:#?}", client_id, event);
                        self.generate(event)?;
                    } else {
                        warn!("Player {} sent invalid event:\n\t{:#?}", client_id, event);
                    }
                }
            }
        }

        self.renet_server.send_packets()?;
        Ok(())
    }

    fn spawn<T>(&mut self, t: T) -> Object<T> {
        self.ids.spawn(t)
    }
}

fn server() -> Result<(), RenetNetworkingError> {
    let server_addr: SocketAddr =
        format!("{}:{}", std::env::var("SERVER_HOST")?, std::env::var("SERVER_PORT")?).parse()?;
    let renet_server = renet::RenetServer::new(
        // Pass the current time to renet, so it can use it to order messages
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap(),
        // Pass a server configuration specifying that we want to allow only 2 clients to connect
        // and that we don't want to authenticate them. Everybody is welcome!
        ServerConfig::new(2, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure),
        // Pass the default connection configuration. This will create a reliable, unreliable and blocking channel.
        // We only actually need the reliable one, but we can just not use the other two.
        RenetConnectionConfig::default(),
        UdpSocket::bind(server_addr)?,
    )?;

    info!("Dune server listening on {}", server_addr);

    let game_state = GameState::default();
    let mut last_updated = Instant::now();

    let mut server = Server {
        renet_server,
        state: game_state,
        waiting_players: Default::default(),
        ready_players: Default::default(),
        ids: Default::default(),
    };

    loop {
        // Update server time
        let now = Instant::now();
        server.renet_server.update(now - last_updated)?;
        last_updated = now;

        server.process_events()?;
        thread::sleep(Duration::from_millis(50));
    }
}
