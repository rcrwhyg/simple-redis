use anyhow::Result;
use bytes::BytesMut;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simple_redis::{parse_frame, parse_frame_length, RespFrame};

const DATA: &str = "*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n*1\r\n+OK\r\n*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n*4\r\n$4\r\nHSET\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n*1\r\n-ERR\r\n*3\r\n$4\r\nHGET\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n*3\r\n$4\r\nSADD\r\n$3\r\nkey\r\n$6\r\nmember\r\n:1\r\n";

fn v1_decode(buf: &mut BytesMut) -> Result<Vec<RespFrame>> {
    use simple_redis::RespDecode;
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let frame = RespFrame::decode(buf)?;
        frames.push(frame);
    }
    Ok(frames)
}

fn v2_decode(buf: &mut BytesMut) -> Result<Vec<RespFrame>> {
    use simple_redis::RespDecodeV2;
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let frame = RespFrame::decode(buf)?;
        frames.push(frame);
    }
    Ok(frames)
}

fn v2_decode_no_buf_clone(buf: &mut &[u8]) -> Result<Vec<RespFrame>> {
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let _len = parse_frame_length(buf)?;

        let frame = parse_frame(buf).unwrap();
        frames.push(frame);
    }
    Ok(frames)
}

fn v1_decode_parse_length(buf: &mut &[u8]) -> Result<()> {
    use simple_redis::RespDecode;
    while !buf.is_empty() {
        let len = RespFrame::expect_length(buf)?;
        *buf = &buf[len..];
    }
    Ok(())
}

fn v2_decode_parse_length(buf: &mut &[u8]) -> Result<()> {
    use simple_redis::RespDecodeV2;
    while !buf.is_empty() {
        let len = RespFrame::expect_length(buf)?;
        *buf = &buf[len..];
    }
    Ok(())
}

// fn v1_decode_parse_frame(buf: &mut &[u8]) -> Result<Vec<RespFrame>> {
//     let mut frames = Vec::new();
//     while !buf.is_empty() {
//         let frame = simple_redis::parse_frame(buf).unwrap();
//         frames.push(frame);
//     }
//     Ok(frames)
// }

fn v2_decode_parse_frame(buf: &mut &[u8]) -> Result<Vec<RespFrame>> {
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let frame = parse_frame(buf).unwrap();
        frames.push(frame);
    }
    Ok(frames)
}

fn criterion_benchmark(c: &mut Criterion) {
    let buf = BytesMut::from(DATA);

    c.bench_function("resp v1 decode", |b| {
        b.iter(|| v1_decode(black_box(&mut buf.clone())))
    });

    c.bench_function("resp v2 decode", |b| {
        b.iter(|| v2_decode(black_box(&mut buf.clone())))
    });

    c.bench_function("resp v2 decode no buf clone", |b| {
        b.iter(|| v2_decode_no_buf_clone(black_box(&mut DATA.as_bytes())))
    });

    c.bench_function("resp v1 decode parse length", |b| {
        b.iter(|| v1_decode_parse_length(black_box(&mut DATA.as_bytes())))
    });

    c.bench_function("resp v2 decode parse length", |b| {
        b.iter(|| v2_decode_parse_length(black_box(&mut DATA.as_bytes())))
    });

    // c.bench_function("resp v1 decode parse frame", |b| {
    //     b.iter(|| v1_decode_parse_frame(black_box(&mut DATA.as_bytes())))
    // });

    c.bench_function("resp v2 decode parse frame", |b| {
        b.iter(|| v2_decode_parse_frame(black_box(&mut DATA.as_bytes())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
