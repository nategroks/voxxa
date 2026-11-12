//! Whisper model downloads against a local HTTP server: a broken download is
//! never saved as a model (Q2-12, audit 12), and a download does not hold the
//! transcription lock (Q2-08, audit 6).

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use voxxa_lib::{download_and_load, download_file, TranscriptionEngine, WhisperModel};

/// Serve one canned response per connection: `status`, a Content-Length of
/// `length`, then `body`, sent in 10 pieces `gap` apart.
async fn serve(status: &'static str, length: usize, body: Vec<u8>, gap: Duration) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let (mut sock, _) = listener.accept().await.unwrap();
            let body = body.clone();
            tokio::spawn(async move {
                let mut req = [0u8; 4096];
                let _ = sock.read(&mut req).await;
                let head = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n"
                );
                sock.write_all(head.as_bytes()).await.unwrap();
                for piece in body.chunks(body.len().div_ceil(10).max(1)) {
                    tokio::time::sleep(gap).await;
                    if sock.write_all(piece).await.is_err() {
                        return;
                    }
                }
                let _ = sock.shutdown().await;
            });
        }
    });
    format!("http://{addr}/ggml-tiny.bin")
}

fn files_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[tokio::test]
async fn a_failed_download_leaves_no_model_file() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("ggml-tiny.bin");

    let url = serve("404 Not Found", 9, b"not found".to_vec(), Duration::ZERO).await;
    assert!(download_file(&url, &dest, |_, _| {}).await.is_err());
    assert_eq!(files_in(dir.path()), Vec::<String>::new());

    // The connection drops after half of the promised bytes.
    let url = serve("200 OK", 1000, vec![7u8; 500], Duration::ZERO).await;
    assert!(download_file(&url, &dest, |_, _| {}).await.is_err());
    assert_eq!(files_in(dir.path()), Vec::<String>::new());

    let url = serve("200 OK", 1000, vec![7u8; 1000], Duration::ZERO).await;
    download_file(&url, &dest, |_, _| {}).await.unwrap();
    assert_eq!(files_in(dir.path()), ["ggml-tiny.bin"]);
    assert_eq!(std::fs::read(&dest).unwrap().len(), 1000);
}

#[tokio::test]
async fn an_unreadable_model_is_moved_aside_so_it_can_be_downloaded_again() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("ggml-tiny.bin"),
        b"<html>not a model</html>",
    )
    .unwrap();
    let mut engine = TranscriptionEngine::with_model_dir(dir.path().to_path_buf());
    assert!(engine.is_model_downloaded(&WhisperModel::Tiny));
    assert!(engine.load_model(&WhisperModel::Tiny).is_err());
    assert!(!engine.is_model_downloaded(&WhisperModel::Tiny));
    assert_eq!(files_in(dir.path()), ["ggml-tiny.bin.unreadable"]);
}

#[tokio::test]
async fn a_download_does_not_hold_the_transcription_lock() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Arc::new(Mutex::new(TranscriptionEngine::with_model_dir(
        dir.path().to_path_buf(),
    )));
    // About 1.5 s in total.
    let url = serve("200 OK", 100, vec![1u8; 100], Duration::from_millis(150)).await;
    let task = {
        let engine = engine.clone();
        tokio::spawn(async move {
            download_and_load(&engine, &WhisperModel::Tiny, &url, |_, _| {}).await
        })
    };
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(!task.is_finished(), "the download should still be running");
    let locked = tokio::time::timeout(Duration::from_millis(100), engine.lock()).await;
    assert!(locked.is_ok(), "the listening loop would be blocked here");
    drop(locked);
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn two_downloads_of_one_model_never_mix_their_bytes() {
    // The operator clicks Download twice. Both must finish, and the file must
    // be exactly the body, not a mix of two writers on one .part file.
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("ggml-tiny.bin");
    let body: Vec<u8> = (0..1000u32).map(|i| (i % 251) as u8).collect();
    let slow = serve("200 OK", 1000, body.clone(), Duration::from_millis(100)).await;
    let fast = serve("200 OK", 1000, body.clone(), Duration::ZERO).await;
    let first = {
        let dest = dest.clone();
        tokio::spawn(async move { download_file(&slow, &dest, |_, _| {}).await })
    };
    tokio::time::sleep(Duration::from_millis(250)).await;
    let second = download_file(&fast, &dest, |_, _| {}).await;
    let first = first.await.unwrap();
    assert!(first.is_ok(), "first download: {first:?}");
    assert!(second.is_ok(), "second download: {second:?}");
    assert_eq!(std::fs::read(&dest).unwrap(), body);
    assert_eq!(files_in(dir.path()), ["ggml-tiny.bin"]);
}
