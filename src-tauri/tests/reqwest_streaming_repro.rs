//! sing-box 1.14 server-streaming 在 reqwest 0.12 + hyper 0.14 下的最小复现
//!
//! 目标:验证 keep-alive 池是否会让长 streaming 连接被错误放回 idle 池,以及
//! `Connection: close` 头或 `pool_max_idle_per_host(0)` 是否能稳定 streaming。
//!
//! 运行:需要 sing-box 1.14 正在监听 127.0.0.1:12081
//! `cargo test --test reqwest_streaming_repro -- --nocapture --ignored`

use futures::StreamExt;
use std::error::Error as _;
use std::time::Duration;

const API_PORT: u16 = 12081;

fn build_request_frame(interval_nanos: i64) -> Vec<u8> {
    // gRPC-Web SubscribeStatusRequest: field 1 (interval, varint)
    let mut body = Vec::new();
    body.push(0x08); // (1 << 3) | 0
    let mut v = interval_nanos as u64;
    loop {
        let mut b = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            b |= 0x80;
        }
        body.push(b);
        if v == 0 {
            break;
        }
    }
    let mut out = vec![0u8];
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(&body);
    out
}

async fn try_streaming(label: &str, client: reqwest::Client, use_close_header: bool) {
    println!("\n=== {label} (close={use_close_header}) ===");
    let url = format!("http://127.0.0.1:{API_PORT}/daemon.StartedService/SubscribeStatus");
    let req = client
        .post(&url)
        .header("Content-Type", "application/grpc-web+proto")
        .header("X-Grpc-Web", "1")
        .header("Accept", "application/grpc-web+proto")
        .header("TE", "trailers");
    let req = if use_close_header {
        req.header("Connection", "close")
    } else {
        req
    };

    let resp = match req.body(build_request_frame(1_000_000_000)).send().await {
        Ok(r) => r,
        Err(e) => {
            println!("  [SEND FAILED] {e}");
            println!("    is_connect={} is_timeout={} is_request={} is_body={} is_decode={}",
                e.is_connect(), e.is_timeout(), e.is_request(), e.is_body(), e.is_decode());
            if let Some(src) = e.source() {
                println!("    source: {src}");
                let mut cur = src.source();
                while let Some(s) = cur {
                    println!("      -> {s}");
                    cur = s.source();
                }
            }
            return;
        }
    };
    println!("  HTTP {}", resp.status());
    if !resp.status().is_success() {
        println!("  [NON-2xx] body={:?}", resp.text().await);
        return;
    }

    let mut stream = resp.bytes_stream();
    let t0 = std::time::Instant::now();
    let mut chunks = 0;
    let mut bytes = 0;
    while let Some(item) = stream.next().await {
        match item {
            Ok(chunk) => {
                chunks += 1;
                bytes += chunk.len();
                let head: Vec<String> = chunk
                    .iter()
                    .take(10)
                    .map(|b| format!("{:02x}", b))
                    .collect();
                println!("  [+{:>4}ms] chunk {}: {}B head={}", t0.elapsed().as_millis(), chunks, chunk.len(), head.join(" "));
                if chunks >= 15 {
                    println!("  [OK] 15 frames in {}ms", t0.elapsed().as_millis());
                    return;
                }
            }
            Err(e) => {
                println!("  [+{:>4}ms] [STREAM ERR] {e}", t0.elapsed().as_millis());
                return;
            }
        }
    }
    println!(
        "  [+{:>4}ms] [STREAM DONE/EOF] chunks={} bytes={}",
        t0.elapsed().as_millis(),
        chunks,
        bytes
    );
}

#[tokio::test]
#[ignore = "需要 sing-box 1.14 在跑;cargo test -- --ignored --nocapture"]
async fn repro_default_keepalive() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .no_proxy()
        .build()
        .expect("client");
    try_streaming("DEFAULT keep-alive", client, false).await;
}

#[tokio::test]
#[ignore = "需要 sing-box 1.14 在跑"]
async fn repro_connection_close_header() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .no_proxy()
        .build()
        .expect("client");
    try_streaming("DEFAULT + Connection: close", client, true).await;
}

#[tokio::test]
#[ignore = "需要 sing-box 1.14 在跑"]
async fn repro_pool_disabled() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .no_proxy()
        .pool_max_idle_per_host(0)
        .build()
        .expect("client");
    try_streaming("pool_max_idle_per_host(0)", client, false).await;
}
