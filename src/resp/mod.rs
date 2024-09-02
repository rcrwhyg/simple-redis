use std::collections::{HashMap, HashSet};

pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<RespFrame>),
    Null(RespNull),
    NullArray(RespNullArray),
    Boolean(bool),
    Double(f64),
    Map(HashMap<String, RespFrame>),
    Set(HashSet<RespFrame>),
}

#[allow(dead_code)]
pub struct SimpleString(String);
#[allow(dead_code)]
pub struct SimpleError(String);
pub struct RespNull;
pub struct RespNullArray;
