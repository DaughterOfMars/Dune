use super::*;

pub fn connect_to_server(commands: &mut Commands) -> Result<(), RenetNetworkingError> {
    let client = client()?;
    let client_id = client.client_id();
    commands.insert_resource(client);
    commands.insert_resource(PlayerId(client_id));
    Ok(())
}

fn client() -> Result<RenetClient, RenetNetworkingError> {
    let server_addr: SocketAddr =
        format!("{}:{}", std::env::var("SERVER_HOST")?, std::env::var("SERVER_PORT")?).parse()?;
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id = current_time.as_millis() as u64;

    let user_data = [0u8; NETCODE_USER_DATA_BYTES];

    Ok(RenetClient::new(
        current_time,
        socket,
        client_id,
        RenetConnectionConfig::default(),
        ClientAuthentication::Unsecure {
            client_id,
            protocol_id: PROTOCOL_ID,
            server_addr,
            user_data: Some(user_data),
        },
    )?)
}
