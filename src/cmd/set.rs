use crate::{Backend, RespArray, RespFrame};

use super::{
    extract_args, validate_command, CommandError, CommandExecutor, SAdd, SIsMember, RESP_OK,
};

impl CommandExecutor for SAdd {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.sadd(self.key, self.member);

        RESP_OK.clone()
    }
}

impl CommandExecutor for SIsMember {
    fn execute(self, backend: &Backend) -> RespFrame {
        match backend.sismember(&self.key, &self.member) {
            true => RespFrame::Integer(1),
            false => RespFrame::Integer(0),
        }
    }
}

impl TryFrom<RespArray> for SAdd {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sadd"], 2)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(member)) => Ok(SAdd {
                key: String::from_utf8(key.0)?,
                member,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid key or member".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for SIsMember {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sismember"], 2)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(member))) => {
                Ok(SIsMember {
                    key: String::from_utf8(key.0)?,
                    member: String::from_utf8(member.0)?,
                })
            }
            _ => Err(CommandError::InvalidArgument(
                "Invalid key or member".to_string(),
            )),
        }
    }
}
