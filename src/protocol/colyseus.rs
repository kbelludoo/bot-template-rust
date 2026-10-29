pub const COLYSEUS_OP_HANDSHAKE: u8 = 9;
pub const COLYSEUS_OP_JOIN_ROOM: u8 = 10;
pub const COLYSEUS_OP_ERROR: u8 = 11;
pub const COLYSEUS_OP_LEAVE_ROOM: u8 = 12;
pub const COLYSEUS_OP_ROOM_DATA: u8 = 13;
pub const COLYSEUS_OP_ROOM_STATE: u8 = 14;
pub const COLYSEUS_OP_ROOM_STATE_PATCH: u8 = 15;
pub const COLYSEUS_OP_PING: u8 = 18;
pub const COLYSEUS_OP_PONG: u8 = 19;

/// Decodifica o quadro binário OpJoinRoom (10) e retorna (reconnection_token, serializer_id, ack_frame)
pub fn parse_colyseus_join_room(frame: &[u8]) -> Result<(String, String, Vec<u8>), String> {
    if frame.len() < 2 || frame[0] != COLYSEUS_OP_JOIN_ROOM {
        return Err("frame inválido para OpJoinRoom".to_string());
    }

    let mut offset = 1;
    let token_len = frame[offset] as usize;
    offset += 1;

    if offset + token_len > frame.len() {
        return Err("buffer too short".to_string());
    }

    let reconnection_token = String::from_utf8_lossy(&frame[offset..offset + token_len]).to_string();
    offset += token_len;

    let mut serializer_id = String::new();
    if offset < frame.len() {
        let ser_len = frame[offset] as usize;
        offset += 1;
        if offset + ser_len <= frame.len() {
            serializer_id = String::from_utf8_lossy(&frame[offset..offset + ser_len]).to_string();
        }
    }

    Ok((reconnection_token, serializer_id, vec![COLYSEUS_OP_JOIN_ROOM]))
}

/// Monta o quadro binário OpRoomData (13) com cabeçalho msgpack fixstr/str8 e payload msgpack
pub fn encode_colyseus_room_data(msg_type: &str, msgpack_payload: &[u8]) -> Vec<u8> {
    let type_bytes = msg_type.as_bytes();
    let mut out = Vec::new();

    if type_bytes.len() <= 31 {
        // msgpack fixstr (0xa0 | len)
        out.push(COLYSEUS_OP_ROOM_DATA);
        out.push(0xa0 | (type_bytes.len() as u8));
    } else {
        // msgpack str8 (0xd9, len)
        out.push(COLYSEUS_OP_ROOM_DATA);
        out.push(0xd9);
        out.push(type_bytes.len() as u8);
    }

    out.extend_from_slice(type_bytes);
    out.extend_from_slice(msgpack_payload);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colyseus_join_room() {
        let mut frame = vec![COLYSEUS_OP_JOIN_ROOM];
        let token = "test-token-123";
        frame.push(token.len() as u8);
        frame.extend_from_slice(token.as_bytes());
        let ser = "schema";
        frame.push(ser.len() as u8);
        frame.extend_from_slice(ser.as_bytes());

        let (t, s, ack) = parse_colyseus_join_room(&frame).unwrap();
        assert_eq!(t, "test-token-123");
        assert_eq!(s, "schema");
        assert_eq!(ack, vec![COLYSEUS_OP_JOIN_ROOM]);
    }

    #[test]
    fn test_colyseus_room_data() {
        let payload = vec![1, 2, 3];
        let frame = encode_colyseus_room_data("ping", &payload);
        assert_eq!(frame[0], COLYSEUS_OP_ROOM_DATA);
        assert_eq!(frame[1], 0xa0 | 4);
        assert_eq!(&frame[2..6], b"ping");
        assert_eq!(&frame[6..], &[1, 2, 3]);
    }
}
