use super::LaunchNode;
use crate::{rpc::RethRpcAddOns, EngineNodeLauncher, NodeHandle};
use alloy_provider::network::AnyNetwork;
use reth_chainspec::EthChainSpec;
use reth_consensus_debug_client::{DebugConsensusClient, EtherscanBlockProvider, RpcBlockProvider};
use reth_node_api::FullNodeComponents;
use std::sync::Arc;
use tracing::info;

/// Node launcher with support for launching various debugging utilities.
///
/// This launcher wraps an existing launcher and adds debugging capabilities when
/// certain debug flags are enabled. It provides two main debugging features:
///
/// ## RPC Consensus Client
///
/// When `--debug.rpc-consensus-ws <URL>` is provided, the launcher will:
/// - Connect to an external RPC `WebSocket` endpoint
/// - Fetch blocks from that endpoint
/// - Submit them to the local engine for execution
/// - Useful for testing engine behavior with real network data
///
/// ## Etherscan Consensus Client
///
/// When `--debug.etherscan [URL]` is provided, the launcher will:
/// - Use Etherscan API as a consensus client
/// - Fetch recent blocks from Etherscan
/// - Submit them to the local engine
/// - Requires `ETHERSCAN_API_KEY` environment variable
/// - Falls back to default Etherscan URL for the chain if URL not provided
#[derive(Debug, Clone)]
pub struct DebugNodeLauncher<L = EngineNodeLauncher> {
    inner: L,
}

impl<L> DebugNodeLauncher<L> {
    /// Creates a new instance of the [`DebugNodeLauncher`].
    pub const fn new(inner: L) -> Self {
        Self { inner }
    }
}

impl<L, Target, N, AddOns> LaunchNode<Target> for DebugNodeLauncher<L>
where
    N: FullNodeComponents,
    AddOns: RethRpcAddOns<N>,
    L: LaunchNode<Target, Node = NodeHandle<N, AddOns>>,
{
    type Node = NodeHandle<N, AddOns>;

    async fn launch_node(self, target: Target) -> eyre::Result<Self::Node> {
        let handle = self.inner.launch_node(target).await?;

        let config = &handle.node.config;
        if let Some(ws_url) = config.debug.rpc_consensus_ws.clone() {
            info!(target: "reth::cli", "Using RPC WebSocket consensus client: {}", ws_url);

            // Create a block provider that works with the debug consensus client
            // Note: This is a simplified implementation that assumes Ethereum-style blocks
            // TODO: Use the new TryFromBlockResponse trait once the debug client is updated
            let block_provider: RpcBlockProvider<AnyNetwork, _> =
                RpcBlockProvider::new(ws_url.as_str(), |block_response| {
                    // Convert through JSON for compatibility - this preserves the old behavior
                    let json = serde_json::to_value(block_response)
                        .expect("Block serialization cannot fail");
                    let rpc_block: alloy_rpc_types::Block =
                        serde_json::from_value(json).expect("Block deserialization cannot fail");
                    let consensus_block: alloy_consensus::Block<alloy_consensus::TxEnvelope> =
                        rpc_block.into_consensus().convert_transactions();

                    // Convert to the node's specific block type through JSON serialization
                    let block_json = serde_json::to_value(&consensus_block)
                        .expect("Block serialization cannot fail");
                    serde_json::from_value(block_json)
                        .expect("Block deserialization to node type cannot fail")
                })
                .await?;

            let rpc_consensus_client = DebugConsensusClient::new(
                handle.node.add_ons_handle.beacon_engine_handle.clone(),
                Arc::new(block_provider),
            );

            handle.node.task_executor.spawn_critical("rpc-ws consensus client", async move {
                rpc_consensus_client.run().await
            });
        }

        if let Some(maybe_custom_etherscan_url) = config.debug.etherscan.clone() {
            info!(target: "reth::cli", "Using etherscan as consensus client");

            let chain = config.chain.chain();
            let etherscan_url = maybe_custom_etherscan_url.map(Ok).unwrap_or_else(|| {
                // If URL isn't provided, use default Etherscan URL for the chain if it is known
                chain
                    .etherscan_urls()
                    .map(|urls| urls.0.to_string())
                    .ok_or_else(|| eyre::eyre!("failed to get etherscan url for chain: {chain}"))
            })?;

            // Create an Etherscan block provider that works with the debug consensus client
            // TODO: Use the new TryFromBlockResponse trait once the debug client is updated
            let block_provider = EtherscanBlockProvider::new(
                etherscan_url,
                chain.etherscan_api_key().ok_or_else(|| {
                    eyre::eyre!(
                        "etherscan api key not found for rpc consensus client for chain: {chain}"
                    )
                })?,
                |rpc_block: alloy_rpc_types::Block| {
                    // For Etherscan, we directly convert the RPC block since it's not from an alloy
                    // network response This preserves the old behavior for now
                    let consensus_block: alloy_consensus::Block<alloy_consensus::TxEnvelope> =
                        rpc_block.into_consensus().convert_transactions();

                    // Convert to the node's specific block type through JSON serialization
                    let block_json = serde_json::to_value(&consensus_block)
                        .expect("Block serialization cannot fail");
                    serde_json::from_value(block_json)
                        .expect("Block deserialization to node type cannot fail")
                },
            );
            let rpc_consensus_client = DebugConsensusClient::new(
                handle.node.add_ons_handle.beacon_engine_handle.clone(),
                Arc::new(block_provider),
            );
            handle.node.task_executor.spawn_critical("etherscan consensus client", async move {
                rpc_consensus_client.run().await
            });
        }

        Ok(handle)
    }
}
