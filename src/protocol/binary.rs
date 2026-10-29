#[derive(Debug, PartialEq, Eq)]
pub enum WireType {
    Varint = 0,
    Fixed64 = 1,
    LengthDelimited = 2,
    Fixed32 = 5,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProtocolError {
    BufferTooShort,
    Overflow,
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::BufferTooShort => write!(f, "Buffer too short"),
            ProtocolError::Overflow => write!(f, "Varint overflow (> 64 bits)"),
        }
    }
}

impl std::error::Error for ProtocolError {}

pub fn decode_varint(buf: &[u8]) -> Result<(u64, usize), ProtocolError> {
    let mut val: u64 = 0;
    let mut shift: u32 = 0;

    for (i, &b) in buf.iter().enumerate() {
        if i >= 10 {
            return Err(ProtocolError::Overflow);
        }
        val |= ((b & 0x7F) as u64) << shift;
        if (b & 0x80) == 0 {
            return Ok((val, i + 1));
        }
        shift += 7;
    }
    Err(ProtocolError::BufferTooShort)
}

pub fn decode_zigzag(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 1) as i64))
}

pub fn decode_tag(buf: &[u8]) -> Result<(u32, u8, usize), ProtocolError> {
    let (v, n) = decode_varint(buf)?;
    let wire_type = (v & 0x07) as u8;
    let field_number = (v >> 3) as u32;
    Ok((field_number, wire_type, n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_and_zigzag() {
        let buf = [0xAC, 0x02]; // 300
        let (val, len) = decode_varint(&buf).unwrap();
        assert_eq!(val, 300);
        assert_eq!(len, 2);

        assert_eq!(decode_zigzag(0), 0);
        assert_eq!(decode_zigzag(1), -1);
        assert_eq!(decode_zigzag(2), 1);
        assert_eq!(decode_zigzag(3), -2);
    }
}
