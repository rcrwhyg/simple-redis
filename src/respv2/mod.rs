use bytes::BytesMut;

use crate::{RespError, RespFrame};

pub use parser::{parse_frame, parse_frame_length};

mod parser;

pub trait RespDecodeV2: Sized {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}

impl RespDecodeV2 for RespFrame {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let len = Self::expect_length(buf)?;
        let data = buf.split_to(len);

        parse_frame(&mut data.as_ref()).map_err(|e| RespError::InvalidFrame(e.to_string()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        parse_frame_length(buf)
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{RespFrame, RespNullArray, RespNullBulkString};
    use anyhow::Result;

    #[test]
    fn respv2_simple_string_length_should_work() -> Result<()> {
        let buf = b"+OK\r\n";
        let len = RespFrame::expect_length(buf)?;
        assert_eq!(len, buf.len());
        Ok(())
    }

    #[test]
    fn respv2_simple_string_length_bad_should_fail() -> Result<()> {
        let buf = b"+OK";
        let ret = RespFrame::expect_length(buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);
        Ok(())
    }

    #[test]
    fn respv2_simple_string_should_work() -> Result<()> {
        let mut buf = BytesMut::from("+OK\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, RespFrame::SimpleString("OK".into()));
        Ok(())
    }

    #[test]
    fn respv2_simple_error_length_should_work() -> Result<()> {
        let buf = b"-ERR\r\n";
        let len = RespFrame::expect_length(buf)?;
        assert_eq!(len, buf.len());
        Ok(())
    }

    #[test]
    fn respv2_simple_error_should_work() -> Result<()> {
        let mut buf = BytesMut::from("-ERR\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, RespFrame::Error("ERR".into()));
        Ok(())
    }

    #[test]
    fn respv2_integer_length_should_work() -> Result<()> {
        let buf = b":1000\r\n";
        let len = RespFrame::expect_length(buf)?;
        assert_eq!(len, buf.len());
        Ok(())
    }

    #[test]
    fn respv2_integer_should_work() -> Result<()> {
        let mut buf = BytesMut::from(":1000\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, RespFrame::Integer(1000));
        Ok(())
    }

    #[test]
    fn respv2_bulk_string_length_should_work() -> Result<()> {
        let buf = b"$5\r\nhello\r\n";
        let len = RespFrame::expect_length(buf)?;
        assert_eq!(len, buf.len());
        Ok(())
    }

    #[test]
    fn respv2_bulk_string_should_work() -> Result<()> {
        let mut buf = BytesMut::from("$5\r\nhello\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, RespFrame::BulkString("hello".into()));
        Ok(())
    }

    #[test]
    fn respv2_null_bulk_string_length_should_work() -> Result<()> {
        let buf = b"$-1\r\n";
        let len = RespFrame::expect_length(buf)?;
        assert_eq!(len, buf.len());
        Ok(())
    }

    #[test]
    fn respv2_null_bulk_string_should_work() -> Result<()> {
        let mut buf = BytesMut::from("$-1\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, RespFrame::NullBulkString(RespNullBulkString));
        Ok(())
    }

    #[test]
    fn respv2_array_length_should_work() -> Result<()> {
        let buf = b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n";
        let len = RespFrame::expect_length(buf)?;
        assert_eq!(len, buf.len());
        Ok(())
    }

    #[test]
    fn respv2_array_should_work() -> Result<()> {
        let mut buf = BytesMut::from("*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespFrame::Array(
                vec![
                    RespFrame::BulkString("set".into()),
                    RespFrame::BulkString("hello".into())
                ]
                .into()
            )
        );
        Ok(())
    }

    #[test]
    fn respv2_null_array_length_should_work() -> Result<()> {
        let buf = b"*-1\r\n";
        let len = RespFrame::expect_length(buf)?;
        assert_eq!(len, buf.len());
        Ok(())
    }

    #[test]
    fn respv2_null_array_should_work() -> Result<()> {
        let mut buf = BytesMut::from("*-1\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, RespFrame::NullArray(RespNullArray));
        Ok(())
    }

    #[test]
    fn respv2_map_length_should_work() -> Result<()> {
        let buf = b"%1\r\n+OK\r\n-ERR\r\n";
        let len = RespFrame::expect_length(buf).unwrap();
        assert_eq!(len, buf.len());
        Ok(())
    }

    #[test]
    fn respv2_map_should_work() -> Result<()> {
        let mut buf = BytesMut::from("%1\r\n+OK\r\n-ERR\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        let items: BTreeMap<String, RespFrame> =
            [("OK".to_string(), RespFrame::Error("ERR".into()))]
                .into_iter()
                .collect();
        assert_eq!(frame, RespFrame::Map(items.into()));
        Ok(())
    }

    #[test]
    fn respv2_map_with_real_data_should_work() -> Result<()> {
        let mut buf = BytesMut::from("%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n");
        let frame = RespFrame::decode(&mut buf).unwrap();
        let items: BTreeMap<String, RespFrame> = [
            ("hello".to_string(), RespFrame::BulkString("world".into())),
            ("foo".to_string(), RespFrame::BulkString("bar".into())),
        ]
        .into_iter()
        .collect();
        assert_eq!(frame, RespFrame::Map(items.into()));
        Ok(())
    }
}
