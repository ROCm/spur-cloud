use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, AttachParams};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, warn};

/// Bridge a WebSocket connection to a kubectl exec session in a pod.
///
/// Flow: xterm.js (browser) <-> WebSocket <-> kube exec <-> bash (pod)
pub async fn handle_terminal(
    socket: WebSocket,
    kube_client: kube::Client,
    namespace: String,
    pod_name: String,
) {
    debug!(pod = %pod_name, ns = %namespace, "terminal session starting");

    let pods: Api<Pod> = Api::namespaced(kube_client, &namespace);

    // Start exec session with interactive TTY
    let attach_params = AttachParams {
        stdin: true,
        stdout: true,
        stderr: true,
        tty: true,
        container: None,
        max_stdin_buf_size: Some(1024),
        max_stdout_buf_size: Some(1024),
        max_stderr_buf_size: Some(1024),
    };

    let mut exec = match pods
        .exec(&pod_name, vec!["bash", "-l"], &attach_params)
        .await
    {
        Ok(e) => e,
        Err(e) => {
            error!("kube exec failed: {e}");
            return;
        }
    };

    let mut stdin = match exec.stdin() {
        Some(s) => s,
        None => {
            error!("no stdin from kube exec");
            return;
        }
    };

    let mut stdout = match exec.stdout() {
        Some(s) => s,
        None => {
            error!("no stdout from kube exec");
            return;
        }
    };

    let (mut ws_sink, mut ws_stream) = socket.split();

    // Task 1: WebSocket → pod stdin
    let stdin_handle = tokio::spawn(async move {
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if stdin.write_all(text.as_bytes()).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Binary(data)) => {
                    if stdin.write_all(&data).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    });

    // Task 2: pod stdout → WebSocket
    let stdout_handle = tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        loop {
            match stdout.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    if ws_sink.send(Message::Text(data)).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Wait for either direction to finish
    tokio::select! {
        _ = stdin_handle => {
            debug!(pod = %pod_name, "terminal stdin closed");
        }
        _ = stdout_handle => {
            debug!(pod = %pod_name, "terminal stdout closed");
        }
    }

    warn!(pod = %pod_name, "terminal session ended");
}
