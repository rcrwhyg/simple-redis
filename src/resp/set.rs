use std::ops::Deref;

use bytes::{Buf, BytesMut};

use super::{
    calc_total_length, parse_length, RespDecode, RespEncode, RespError, RespFrame, BUF_CAP,
    CRLF_LEN,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespSet(pub(crate) Vec<RespFrame>);

impl RespSet {
    pub fn new(v: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(v.into())
    }
}

// - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
impl RespEncode for RespSet {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

// - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
impl RespDecode for RespSet {
    const PREFIX: &'static str = "~";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;

        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let mut frames = Vec::new();
        for _ in 0..len {
            frames.push(RespFrame::decode(buf)?);
        }

        Ok(Self::new(frames))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BulkString, RespArray};
    use anyhow::Result;

    #[test]
    fn test_set_encode() {
        let frame: RespFrame = RespSet::new(vec![
            RespArray::new([1234.into(), true.into()]).into(),
            BulkString::new("world").into(),
        ])
        .into();

        assert_eq!(
            &frame.encode(),
            b"~2\r\n*2\r\n:+1234\r\n#t\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn test_set_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"~2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespSet::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespSet::new(vec![
                BulkString::new(b"set").into(),
                BulkString::new(b"hello").into()
            ])
        );

        Ok(())
    }
}
