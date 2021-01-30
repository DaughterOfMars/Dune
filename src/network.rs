use std::{
    collections::{HashMap, VecDeque},
    io::Cursor,
    net::SocketAddr,
    time::Instant,
};

use bevy::prelude::*;
use bytecheck::CheckBytes;
use laminar::{Packet, Socket, SocketEvent};
use rkyv::{check_archive, Archive, ArchiveWriter, Seek, Unarchive, Write};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Network>()
            .add_system(server_system.system())
            .add_system(client_system.system());
    }
}

#[derive(Archive, Unarchive, PartialEq, Clone, Debug)]
#[archive(derive(CheckBytes))]
pub enum Message {
    Connect,
    Ping,
    Data(Vec<u8>),
}

impl Message {
    fn into_bytes(&self) -> Vec<u8> {
        let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()));
        writer
            .archive_root(self)
            .expect("Failed to serialize message!");
        writer.into_inner().into_inner()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let archived = check_archive::<Self>(bytes, 0).expect("Failed to validate message!");
        archived.unarchive()
    }
}

pub struct Network {
    pub network_type: NetworkType,
}

impl Default for Network {
    fn default() -> Self {
        Network {
            network_type: NetworkType::None,
        }
    }
}

#[derive(PartialEq)]
pub enum NetworkType {
    Server,
    Client,
    None,
}

pub struct Server {
    pub socket: Socket,
    pub clients: HashMap<SocketAddr, Connection>,
    pub messages: VecDeque<Vec<u8>>,
}

#[derive(Copy, Clone)]
pub struct Connection {
    pub address: SocketAddr,
    pub state: ConnectionState,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ConnectionState {
    Healthy,
    TimedOut,
    Disconnected,
}

impl Server {
    pub fn new(port: &str) -> Self {
        let socket =
            Socket::bind(format!("127.0.0.1:{}", port)).expect("Failed to bind server socket!");
        Server {
            socket,
            clients: HashMap::new(),
            messages: VecDeque::new(),
        }
    }

    pub fn send_to_all(&mut self, message: Vec<u8>) {
        for &address in self.clients.iter().filter_map(|(address, connection)| {
            if connection.state == ConnectionState::Healthy {
                Some(address)
            } else {
                None
            }
        }) {
            println!(
                "Sending {:?} to {}",
                Message::Data(message.clone()),
                address
            );
            self.socket
                .send(Packet::reliable_ordered(
                    address,
                    Message::Data(message.clone()).into_bytes(),
                    None,
                ))
                .expect("Failed to send connection message to server!");
        }
    }

    pub fn send_to(&mut self, address: SocketAddr, message: Vec<u8>) {
        if let Some(connection) = self.clients.get(&address) {
            if connection.state == ConnectionState::Healthy {
                self.socket
                    .send(Packet::reliable_ordered(
                        address,
                        Message::Data(message).into_bytes(),
                        None,
                    ))
                    .expect("Failed to send connection message to server!");
            }
        }
    }
}

pub struct Client {
    pub socket: Socket,
    pub server: Option<Connection>,
    pub messages: VecDeque<Vec<u8>>,
}

impl Client {
    pub fn new(port: &str) -> Self {
        let socket =
            Socket::bind(format!("127.0.0.1:{}", port)).expect("Failed to bind client socket!");
        Client {
            socket,
            server: None,
            messages: VecDeque::new(),
        }
    }

    pub fn connect_to(&mut self, address: SocketAddr) {
        //self.server = Some(Connection {
        //    address,
        //    state: ConnectionState::Healthy,
        //});
        self.socket
            .send(Packet::reliable_ordered(
                address,
                Message::Connect.into_bytes(),
                None,
            ))
            .expect("Failed to send connection message to server!");
    }
}

fn server_system(network: Res<Network>, mut server: Query<&mut Server>) {
    if network.network_type == NetworkType::Server {
        if let Some(mut server) = server.iter_mut().next() {
            //println!("Listening for client events");
            server.socket.manual_poll(Instant::now());
            match server.socket.recv() {
                Some(event) => match event {
                    SocketEvent::Packet(packet) => {
                        //println!(
                        //    "Received packet {:?} from {}",
                        //    Message::from_bytes(packet.payload()),
                        //    packet.addr()
                        //);
                        let message = Message::from_bytes(packet.payload());
                        match message {
                            Message::Connect => {
                                server
                                    .socket
                                    .send(Packet::reliable_ordered(
                                        packet.addr(),
                                        Message::Connect.into_bytes(),
                                        None,
                                    ))
                                    .expect(
                                        "Failed to send connection response message to client!",
                                    );
                            }
                            Message::Ping => {
                                server
                                    .socket
                                    .send(Packet::reliable_ordered(
                                        packet.addr(),
                                        Message::Ping.into_bytes(),
                                        None,
                                    ))
                                    .expect("Failed to send ping response message to client!");
                            }
                            Message::Data(data) => {
                                println!("Received data {:?} from {}", data, packet.addr());
                                server.messages.push_back(data);
                            }
                        }
                    }
                    SocketEvent::Connect(address) => {
                        // a client connected
                        server.clients.entry(address).or_insert_with(|| Connection {
                            address,
                            state: ConnectionState::Healthy,
                        });
                        println!("Client {} connected!", address);
                    }
                    SocketEvent::Timeout(address) => {
                        // a client timed out
                        if let Some(client) = server.clients.get_mut(&address) {
                            client.state = ConnectionState::TimedOut;
                        }
                        println!("Client {} timed out!", address);
                    }
                    SocketEvent::Disconnect(address) => {
                        // a client disconnected
                        if let Some(client) = server.clients.get_mut(&address) {
                            client.state = ConnectionState::Disconnected;
                        }
                        println!("Client {} disconnected!", address);
                    }
                },
                None => (),
            }
        }
    }
}

fn client_system(network: Res<Network>, mut client: Query<&mut Client>) {
    if network.network_type == NetworkType::Client {
        if let Some(mut client) = client.iter_mut().next() {
            //println!("Listening for server events");
            client.socket.manual_poll(Instant::now());
            match client.socket.recv() {
                Some(event) => match event {
                    SocketEvent::Packet(packet) => {
                        // the server sent a packet
                        //println!(
                        //    "Received packet {:?} from {}",
                        //    Message::from_bytes(packet.payload()),
                        //    packet.addr()
                        //);
                        let message = Message::from_bytes(packet.payload());
                        match message {
                            Message::Data(data) => {
                                println!("Received data {:?} from {}", data, packet.addr());
                                client.messages.push_back(data);
                            }
                            _ => (),
                        }
                    }
                    SocketEvent::Connect(address) => {
                        // the server connected
                        client.server = Some(Connection {
                            address,
                            state: ConnectionState::Healthy,
                        });
                        println!("Server {} connected!", address);
                    }
                    SocketEvent::Timeout(address) => {
                        // the server timed out
                        if let Some(ref mut server) = client.server {
                            server.state = ConnectionState::TimedOut;
                        }
                        println!("Server {} timed out!", address);
                    }
                    SocketEvent::Disconnect(address) => {
                        // the server disconnected
                        if let Some(ref mut server) = client.server {
                            server.state = ConnectionState::Disconnected;
                        }
                        println!("Server {} disconnected!", address);
                    }
                },
                None => (),
            }
            if let Some(server) = client.server {
                client
                    .socket
                    .send(Packet::reliable_ordered(
                        server.address,
                        Message::Ping.into_bytes(),
                        None,
                    ))
                    .expect("Failed to send ping response message to server!");
            }
        }
    }
}
