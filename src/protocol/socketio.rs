pub struct SocketIoPacket {
    pub engine_type: u8,
    pub socket_type: Option<u8>,
    pub namespace: String,
    pub payload: String,
}

pub fn parse_socketio_packet(raw: &str) -> Option<SocketIoPacket> {
    if raw.is_empty() {
        return None;
    }
    let mut chars = raw.chars();
    let engine_type = chars.next()?.to_digit(10)? as u8;

    let mut socket_type = None;
    let mut rest = chars.as_str();

    if engine_type == 4 && !rest.is_empty() {
        if let Some(st) = rest.chars().next().and_then(|c| c.to_digit(10)) {
            socket_type = Some(st as u8);
            rest = &rest[1..];
        }
    }

    Some(SocketIoPacket {
        engine_type,
        socket_type,
        namespace: "/".to_string(),
        payload: rest.to_string(),
    })
}
