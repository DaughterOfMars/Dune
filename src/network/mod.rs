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
    NETCODE_USER_DATA_BYTES,
};
use serde::Serialize;
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
        app.init_resource::<GameState>()
            .add_event::<GameEvent>()
            .add_event::<ServerEvent>()
            .add_event::<RenetServerExitedEvent>()
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
    mut server_events: EventWriter<ServerEvent>,
) {
    while let Some(message) = client.receive_message(0) {
        // Route the message types appropriately
        if let Ok(event) = bincode::deserialize::<GameEvent>(&message) {
            trace!("{:#?}", event);

            game_state.consume(event.clone());

            game_events.send(event);
        } else if let Ok(event) = bincode::deserialize::<ServerEvent>(&message) {
            trace!("{:#?}", event);

            server_events.send(event);
        } else {
            warn!("Received invalid message from the server: {:x?}", message);
        }
    }
}

pub trait SendEvent {
    fn send_event<T: Serialize>(&mut self, event: T);
}

impl SendEvent for RenetClient {
    fn send_event<T: Serialize>(&mut self, event: T) {
        self.send_message(0, bincode::serialize(&event).unwrap());
    }
}
