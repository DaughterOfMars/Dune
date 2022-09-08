mod client;
mod server;

use std::{
    env::VarError,
    net::{AddrParseError, SocketAddr, UdpSocket},
    thread,
    time::{Duration, Instant, SystemTime},
};

use bevy::prelude::*;
use iyes_loopless::prelude::IntoConditionalSystem;
use renet::{
    ClientAuthentication, RenetClient, RenetConnectionConfig, RenetError, ServerAuthentication, ServerConfig,
    ServerEvent, NETCODE_USER_DATA_BYTES,
};
use thiserror::Error;

pub use self::{client::*, server::*};
use crate::game::state::{EndGameReason, EventReduce, GameEvent, GameState, PlayerId};

pub const PROTOCOL_ID: u64 = 0;

#[derive(Debug, Error)]
pub enum RenetNetworkingError {
    #[error(transparent)]
    ParseAddress(#[from] AddrParseError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Env(#[from] VarError),
    #[error(transparent)]
    Serialization(#[from] bincode::Error),
    #[error(transparent)]
    Renet(#[from] RenetError),
}

pub struct RenetNetworkingPlugin;

impl Plugin for RenetNetworkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<RenetServerExitedEvent>()
            .add_system(await_server.run_if_resource_exists::<RenetServer>())
            .add_system(process_server_events.run_if_resource_exists::<RenetClient>());
    }
}

pub struct RenetServer {
    handle: Option<thread::JoinHandle<Result<(), RenetNetworkingError>>>,
}

pub struct RenetServerExitedEvent {
    pub result: Result<(), RenetNetworkingError>,
}

fn await_server(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut event_writer: EventWriter<RenetServerExitedEvent>,
) {
    if let Some(handle) = server.handle.as_ref() {
        if handle.is_finished() {
            event_writer.send(RenetServerExitedEvent {
                result: server.handle.take().unwrap().join().unwrap(),
            });
            commands.remove_resource::<RenetServer>();
        }
    }
}

fn process_server_events(
    mut client: ResMut<RenetClient>,
    mut game_state: ResMut<GameState>,
    mut game_events: EventWriter<GameEvent>,
) {
    while let Some(message) = client.receive_message(0) {
        // Whenever the server sends a message we know that it must be a game event
        let event: GameEvent = bincode::deserialize(&message).unwrap();
        trace!("{:#?}", event);

        // We trust the server - It's always been good to us!
        // No need to validate the events it is sending us
        game_state.consume(event.clone());

        // Send the event into the bevy event system so systems can react to it
        game_events.send(event);
    }
}

pub trait SendGameEvent {
    fn send_game_event(&mut self, event: GameEvent);
}

impl SendGameEvent for RenetClient {
    fn send_game_event(&mut self, event: GameEvent) {
        self.send_message(0, bincode::serialize(&event).unwrap());
    }
}
