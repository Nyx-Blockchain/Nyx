// src/rpc.rs

//! RPC server using Axum.

use axum::{
    routing::{get, post},
    Router, Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// RPC server state
#[derive(Clone)]
pub struct RpcState {
    /// Node reference
    pub node: Arc<RwLock<crate::node::NyxNode>>,
}

/// RPC server
pub struct RpcServer {
    /// Listen address
    listen_addr: std::net::SocketAddr,

    /// Server state
    state: RpcState,
}

impl RpcServer {
    /// Creates a new RPC server
    pub fn new(
        listen_addr: std::net::SocketAddr,
        node: Arc<RwLock<crate::node::NyxNode>>,
    ) -> Self {
        Self {
            listen_addr,
            state: RpcState { node },
        }
    }

    /// Starts the RPC server
    pub async fn start(self) -> crate::Result<()> {
        let app = Router::new()
            .route("/", get(root))
            .route("/status", get(get_status))
            .route("/balance", get(get_balance))
            .route("/send", post(send_transaction))
            .route("/submit", post(submit_transaction))
            .with_state(self.state);

        let listener = tokio::net::TcpListener::bind(self.listen_addr).await?;

        tracing::info!("RPC server listening on {}", self.listen_addr);

        axum::serve(listener, app)
            .await
            .map_err(|e| crate::NodeError::RpcError(format!("{}", e)))?;

        Ok(())
    }
}

// RPC handlers

async fn root() -> &'static str {
    "Nyx Node RPC Server"
}

async fn get_status(
    State(state): State<RpcState>,
) -> Json<StatusResponse> {
    let node = state.node.read().await;

    Json(StatusResponse {
        version: crate::NODE_VERSION.to_string(),
        peers: node.network_stats().peer_count,
        mempool_size: node.mempool_size().await,
        syncing: node.is_syncing().await,
    })
}

async fn get_balance(
    State(state): State<RpcState>,
) -> Json<BalanceResponse> {
    let node = state.node.read().await;
    let balance = node.get_balance().await;

    Json(BalanceResponse { balance })
}

async fn send_transaction(
    State(state): State<RpcState>,
    Json(req): Json<SendRequest>,
) -> Json<SendResponse> {
    let node = state.node.read().await;

    match node.send(req.to, req.amount).await {
        Ok(tx_hash) => Json(SendResponse {
            success: true,
            tx_hash: Some(hex::encode(tx_hash)),
            error: None,
        }),
        Err(e) => Json(SendResponse {
            success: false,
            tx_hash: None,
            error: Some(format!("{}", e)),
        }),
    }
}

async fn submit_transaction(
    State(_state): State<RpcState>,
    Json(_req): Json<SubmitRequest>,
) -> Json<SubmitResponse> {
    // TODO: Implement transaction submission
    Json(SubmitResponse {
        success: false,
        error: Some("Not implemented".to_string()),
    })
}

// RPC request/response types

#[derive(Debug, Serialize)]
struct StatusResponse {
    version: String,
    peers: usize,
    mempool_size: usize,
    syncing: bool,
}

#[derive(Debug, Serialize)]
struct BalanceResponse {
    balance: u64,
}

#[derive(Debug, Deserialize)]
struct SendRequest {
    to: String,
    amount: u64,
}

#[derive(Debug, Serialize)]
struct SendResponse {
    success: bool,
    tx_hash: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SubmitRequest {
    transaction: String,
}

#[derive(Debug, Serialize)]
struct SubmitResponse {
    success: bool,
    error: Option<String>,
}
