use std::path::{Path, PathBuf};

use base64::Engine;
use ict_rs::chain::Chain;

/// Write a host file into the chain container via base64 encoding.
async fn write_file_to_chain(
    chain: &dyn Chain,
    host_path: &Path,
    container_path: &str,
) -> anyhow::Result<()> {
    let data = std::fs::read(host_path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", host_path.display(), e))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    let cmd = format!(
        "printf '%s' '{}' | base64 -d > {}",
        b64, container_path
    );
    chain.exec(&["sh", "-c", &cmd], &[]).await?;
    Ok(())
}

/// Resolve the terp-core root directory.
///
/// Checks `ZK_ROOT` env var first, then walks up from the workspace
/// looking for `terp-core/` alongside `terp-rs/`.
fn resolve_zk_root() -> anyhow::Result<PathBuf> {
    if let Ok(root) = std::env::var("ZK_ROOT") {
        return Ok(PathBuf::from(root));
    }

    // Walk up from workspace root (terp-rs/) → parent (ZK/) → terp-core/
    let home = std::env::var("HOME").unwrap_or_default();
    let candidate = PathBuf::from(&home).join("ZK/terp-core");
    if candidate.exists() {
        return Ok(candidate);
    }

    Err(anyhow::anyhow!(
        "Cannot find terp-core. Set ZK_ROOT env var or ensure ~/ZK/terp-core/ exists."
    ))
}

/// Deploy the zk-wasmvm-test contract via `terpd tx wasm headstash`.
///
/// This uses ict-rs chain_exec (not cw-orch) because the `headstash` upload
/// command bundles WASM + verification key in a single transaction.
///
/// Returns the deployed contract address.
pub async fn deploy_zk_contract(chain: &dyn Chain) -> anyhow::Result<String> {
    let zk_root = resolve_zk_root()?;

    let wasm_path = zk_root.join("tests/interchaintest/contracts/zk_wasmvm_test.wasm");
    let vk_path = zk_root.join("tests/interchaintest/circuits/no_rick.bin");

    if !wasm_path.exists() {
        return Err(anyhow::anyhow!(
            "WASM not found: {}. Build terp-core first.",
            wasm_path.display()
        ));
    }
    if !vk_path.exists() {
        return Err(anyhow::anyhow!(
            "VK not found: {}. Build terp-core first.",
            vk_path.display()
        ));
    }

    tracing::info!("Copying ZK artifacts to container...");
    write_file_to_chain(chain, &wasm_path, "/tmp/zk_wasmvm_test.wasm").await?;
    write_file_to_chain(chain, &vk_path, "/tmp/no_rick.bin").await?;

    // Upload contract + VK via headstash command
    tracing::info!("Uploading ZK contract via headstash...");
    let upload_out = chain
        .chain_exec(&[
            "tx",
            "wasm",
            "headstash",
            "/tmp/zk_wasmvm_test.wasm",
            "/tmp/no_rick.bin",
            "--from",
            "validator",
            "--gas-prices",
            "0uterp",
            "--gas",
            "auto",
            "--gas-adjustment",
            "1.5",
            "-y",
            "--output",
            "json",
        ])
        .await?;
    tracing::debug!(
        stdout = %String::from_utf8_lossy(&upload_out.stdout),
        "headstash upload result"
    );

    // Wait for tx inclusion
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Instantiate the contract (code_id = 1, first upload)
    tracing::info!("Instantiating ZK contract...");
    chain
        .chain_exec(&[
            "tx",
            "wasm",
            "instantiate",
            "1",
            "{}",
            "--from",
            "validator",
            "--label",
            "zk-wasmvm-test",
            "--no-admin",
            "--gas-prices",
            "0uterp",
            "--gas",
            "auto",
            "--gas-adjustment",
            "1.5",
            "-y",
            "--output",
            "json",
        ])
        .await?;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Query contract address
    let query_out = chain
        .chain_exec(&[
            "query",
            "wasm",
            "list-contract-by-code",
            "1",
            "--output",
            "json",
        ])
        .await?;

    let stdout_str = String::from_utf8_lossy(&query_out.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse contract query: {}", e))?;

    let addr = json["contracts"][0]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No contract address found in query response"))?
        .to_string();

    tracing::info!(contract = %addr, "ZK contract deployed");
    Ok(addr)
}
