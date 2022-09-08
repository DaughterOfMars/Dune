use rand::seq::SliceRandom;

use super::*;
use crate::{
    components::{Faction, Troop},
    game::{
        state::{Prompt, SpawnType},
        ObjectIdGenerator, Phase, SetupPhase,
    },
};

pub fn spawn_server(commands: &mut Commands) {
    commands.insert_resource(RenetServer {
        handle: Some(std::thread::spawn(server)),
    });
}

pub struct Server {
    renet_server: renet::RenetServer,
    game_state: GameState,
    ids: ObjectIdGenerator,
}

impl Server {
    /// This is the server logic, which is run whenever the game state changes.
    fn game_logic(&mut self, last_event: GameEvent) -> Result<(), RenetNetworkingError> {
        use GameEvent::*;
        match last_event {
            AdvancePhase => match &self.game_state.phase {
                Phase::Setup(s) => match s {
                    SetupPhase::Prediction => {
                        if self
                            .game_state
                            .players
                            .values()
                            .find(|p| p.faction == Faction::BeneGesserit)
                            .is_none()
                        {
                            self.consume(AdvancePhase)?;
                        }
                    }
                    _ => (),
                },
                _ => (),
            },
            StartGame => {
                // TODO: Perhaps allow other ways to determine play order
                let mut play_order = self.game_state.unpicked_players.keys().copied().collect::<Vec<_>>();
                let mut rng = rand::thread_rng();
                play_order.shuffle(&mut rng);
                self.consume(SetPlayOrder { play_order })?;
                self.consume(SetActive {
                    player_id: self.game_state.play_order[0],
                })?;
                self.consume(ShowPrompt {
                    player_id: self.game_state.play_order[0],
                    prompt: Prompt::Faction,
                })?;
            }
            ChooseFaction { faction } => {
                for leader in self
                    .game_state
                    .data
                    .leaders
                    .clone()
                    .into_iter()
                    .filter_map(|(leader, data)| (data.faction == faction).then_some(leader))
                {
                    let leader = self.ids.spawn(leader);
                    self.consume(SpawnObject {
                        spawn_type: SpawnType::Leader {
                            player_id: self.game_state.active_player.unwrap(),
                            leader,
                        },
                    })?;
                }
                for unit in std::iter::repeat_with(|| Troop { is_special: false })
                    .take(20 - self.game_state.data.factions[&faction].special_forces as usize)
                    .chain(
                        std::iter::repeat_with(|| Troop { is_special: true })
                            .take(self.game_state.data.factions[&faction].special_forces as usize),
                    )
                {
                    let unit = self.ids.spawn(unit);
                    self.consume(SpawnObject {
                        spawn_type: SpawnType::Troop {
                            player_id: self.game_state.active_player.unwrap(),
                            unit,
                        },
                    })?;
                }
                self.consume(Pass)?;
                if self.game_state.unpicked_players.is_empty() {
                    self.consume(AdvancePhase)?;
                } else {
                    self.consume(ShowPrompt {
                        player_id: self.game_state.active_player.unwrap(),
                        prompt: Prompt::Faction,
                    })?;
                }
            }
            ChooseTraitor { .. } => {
                if self.game_state.players.values().all(|p| {
                    p.traitor_cards.len()
                        == match p.faction {
                            Faction::Harkonnen => 4,
                            _ => 1,
                        }
                }) {
                    self.consume(AdvancePhase)?;
                }
            }
            _ => (),
        }
        Ok(())
    }

    /// Consume an event and broadcast it to all clients.
    fn consume(&mut self, event: GameEvent) -> Result<(), RenetNetworkingError> {
        let serialized_event = bincode::serialize(&event)?;
        self.game_state.consume(event.clone());
        self.renet_server.broadcast_message(0, serialized_event);
        self.game_logic(event)?;
        Ok(())
    }

    /// Process the current buffer of events.
    fn process_events(&mut self) -> Result<(), RenetNetworkingError> {
        // Receive connection events from clients
        while let Some(event) = self.renet_server.get_event() {
            match event {
                ServerEvent::ClientConnected(id, ..) => {
                    let event = GameEvent::PlayerJoined { player_id: id.into() };
                    if self.game_state.validate(&event) {
                        // Tell the recently joined player about the other players
                        for player_id in self.game_state.players.keys() {
                            let event = GameEvent::PlayerJoined { player_id: *player_id };
                            self.renet_server.send_message(id, 0, bincode::serialize(&event)?);
                        }

                        // Add the new player to the game
                        self.consume(event)?;

                        info!("Client {} connected.", id);
                    } else {
                        warn!("Player sent conflicting client id:\n\t{:#?}", event);
                    }
                }
                ServerEvent::ClientDisconnected(id) => {
                    // First consume a disconnect event
                    self.consume(GameEvent::PlayerDisconnected { player_id: id.into() })?;
                    info!("Client {} disconnected", id);

                    // Then end the game
                    self.consume(GameEvent::EndGame {
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
                if let Ok(event) = bincode::deserialize::<GameEvent>(&message) {
                    if self.game_state.validate(&event) {
                        trace!("Player {} sent:\n\t{:#?}", client_id, event);
                        self.consume(event)?;
                    } else {
                        warn!("Player {} sent invalid event:\n\t{:#?}", client_id, event);
                    }
                }
            }
        }

        self.renet_server.send_packets()?;
        Ok(())
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
        game_state,
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
