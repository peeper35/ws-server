use std::env;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

use crate::types::HTTP_CLIENT;

use anyhow::{Error, Result};

pub fn get_env_var(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Missing environment variable: {}", key))
}

async fn restart_nodejs_ws_conn() -> Result<()> {
    let nodejs_service_restart_ws_conn =
        "https://paper-trading-aznxt.onrender.com/websocket/restart";

    let bearer_token = get_env_var("NODE_JS_SERVICE_AUTH_TOKEN");

    let response = HTTP_CLIENT
        .post(nodejs_service_restart_ws_conn)
        .header(AUTHORIZATION, format!("Bearer {}", bearer_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        info!("Restart Nodejs WS: {}", response.text().await?);
    } else {
        return Err(Error::msg(format!(
            "Request failed with status: {}",
            response.status()
        )));
    }

    Ok(())
}

pub async fn ensure_nodejs_ws_connection() {
    loop {
        if let Err(e) = restart_nodejs_ws_conn().await {
            warn!("Failed to restart Node.js WebSocket connection: {}", e);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        } else {
            info!("Successfully restarted the Node.js WebSocket connection.");
            break;
        }
    }
}
