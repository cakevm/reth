use alloy_network::{AnyNetwork, Ethereum, Network};
use reth_primitives_traits::{NoopConversionError, TryFromBlockResponse};

/// Error type for converting from block response to primitive block.
#[derive(Debug)]
pub struct BlockConversionError(pub String);

impl core::fmt::Display for BlockConversionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Block conversion error: {}", self.0)
    }
}

impl core::error::Error for BlockConversionError {}

/// Wrapper type for Ethereum blocks to work around orphan rule limitations.
/// This type implements `TryFromBlockResponse` and can be converted to/from the primitive Block
/// type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthBlockWrapper(pub crate::Block);

impl From<EthBlockWrapper> for crate::Block {
    fn from(wrapper: EthBlockWrapper) -> Self {
        wrapper.0
    }
}

impl From<crate::Block> for EthBlockWrapper {
    fn from(block: crate::Block) -> Self {
        Self(block)
    }
}

/// Try to convert the block response for `AnyNetwork` into the `EthBlockWrapper` type.
impl TryFromBlockResponse<AnyNetwork> for EthBlockWrapper {
    type Error = serde_json::Error;

    fn from_block_response(
        block_response: <AnyNetwork as Network>::BlockResponse,
    ) -> Result<Self, Self::Error> {
        let rpc_block_json = serde_json::to_value(block_response.into_inner())?;
        let rpc_block: alloy_rpc_types_eth::Block = serde_json::from_value(rpc_block_json)?;

        Ok(Self(rpc_block.into_consensus().convert_transactions()))
    }
}

/// Try to convert the block response for `Ethereum` networks into the `EthBlockWrapper` type.
impl TryFromBlockResponse<Ethereum> for EthBlockWrapper {
    type Error = NoopConversionError;

    fn from_block_response(
        block_response: <Ethereum as Network>::BlockResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self(block_response.into_consensus().convert_transactions()))
    }
}
