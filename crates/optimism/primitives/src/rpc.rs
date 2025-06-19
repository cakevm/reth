use crate::{OpBlockBody, OpTransactionSigned};
use alloc::string::String;
use alloy_network::{AnyNetwork, Network};
use op_alloy_network::Optimism;
use reth_primitives_traits::{NoopConversionError, TryFromBlockResponse};

/// Wrapper type for Optimism blocks to work around orphan rule limitations.
/// This type implements `TryFromBlockResponse` and can be converted to/from the primitive `OpBlock`
/// type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpBlockWrapper(pub crate::OpBlock);

impl From<OpBlockWrapper> for crate::OpBlock {
    fn from(wrapper: OpBlockWrapper) -> Self {
        wrapper.0
    }
}

impl From<crate::OpBlock> for OpBlockWrapper {
    fn from(block: crate::OpBlock) -> Self {
        Self(block)
    }
}

/// Try to convert the block response for `AnyNetwork` into the `OpBlockWrapper` type.
impl TryFromBlockResponse<AnyNetwork> for OpBlockWrapper {
    type Error = serde_json::Error;

    fn from_block_response(
        block_response: <AnyNetwork as Network>::BlockResponse,
    ) -> Result<Self, Self::Error> {
        let rpc_block_json = serde_json::to_value(block_response.into_inner())?;

        let rpc_block: alloy_rpc_types_eth::Block<op_alloy_consensus::OpTxEnvelope> =
            serde_json::from_value(rpc_block_json)?;

        Ok(Self(rpc_block.into_consensus()))
    }
}

/// Try to convert the block response for `Optimism` networks into the `OpBlockWrapper` type.
impl TryFromBlockResponse<Optimism> for OpBlockWrapper {
    type Error = NoopConversionError;

    fn from_block_response(
        block_response: <Optimism as Network>::BlockResponse,
    ) -> Result<Self, Self::Error> {
        // TODO: Have a type conversion simple as for eth
        let block = block_response
            .into_consensus()
            .map_transactions(|tx| OpTransactionSigned::from(<alloy_rpc_types_eth::Transaction<op_alloy_consensus::OpTxEnvelope> as Clone>::clone(&tx).into_inner()));

        Ok(Self(block))
    }
}
