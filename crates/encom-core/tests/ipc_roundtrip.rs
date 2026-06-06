//! Integration test: bring up the daemon on an ephemeral port, connect a
//! client, run one Chat turn end-to-end, assert chunks + done arrive.

use encom_core::{Config, Daemon};
use encom_ipc::{connect, handshake, write_frame, Frame};
use std::time::Duration;
use tokio::net::TcpListener;

async fn pick_free_port() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = l.local_addr().unwrap().port();
    drop(l);
    port
}

#[tokio::test]
async fn chat_round_trip() {
    let port = pick_free_port().await;
    let addr = format!("127.0.0.1:{port}");
    let cfg = Config::default_with_anthropic();
    let state_dir = std::env::temp_dir().join(format!("encom-test-{port}"));

    let daemon_addr = addr.clone();
    let daemon = tokio::spawn(async move {
        Daemon::new(cfg, state_dir)
            .with_bind_addr(daemon_addr)
            .run()
            .await
            .ok();
    });

    // Give the listener a beat to bind.
    for _ in 0..30 {
        if tokio::net::TcpStream::connect(&addr).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let (mut rd, mut wr) = connect(&addr).await.expect("connect");
    let (server, _v) = handshake(&mut rd, &mut wr, "test-client", "0.0.1")
        .await
        .expect("handshake");
    assert_eq!(server, "encom");

    write_frame(
        &mut wr,
        &Frame::Chat {
            text: "ping".into(),
        },
    )
    .await
    .unwrap();

    let mut chunks: Vec<String> = Vec::new();
    let mut tokens: Option<(u32, u32)> = None;
    while let Some(frame) = rd.next().await.expect("read frame") {
        match frame {
            Frame::ChatChunk { text } => chunks.push(text),
            Frame::ChatDone {
                input_tokens,
                output_tokens,
            } => {
                tokens = Some((input_tokens, output_tokens));
                break;
            }
            other => panic!("unexpected frame: {other:?}"),
        }
    }

    assert!(!chunks.is_empty(), "expected at least one chunk");
    let assembled: String = chunks.join("");
    assert!(
        assembled.contains("you said: ping"),
        "stub reply mismatch: {assembled}"
    );
    let (input, output) = tokens.expect("got chat_done");
    assert_eq!(input, 4); // "ping"
    assert!(output > 0);

    drop(wr);
    drop(rd);
    daemon.abort();
}
