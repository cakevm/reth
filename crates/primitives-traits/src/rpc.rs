//! RPC conversion traits for node types.

use alloy_network::Network;

/// This error can be used if the conversion can not fail.
#[derive(Debug)]
pub struct NoopConversionError;

impl core::fmt::Display for NoopConversionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "This error is a no-op and should never been returned")
    }
}

impl core::error::Error for NoopConversionError {}

/// Trait for converting from alloy network block responses to primitive blocks.
pub trait TryFromBlockResponse<N: Network> {
    /// The error type returned when conversion fails.
    type Error: core::error::Error + Send + Sync + Unpin;

    /// Converts from a network block response to the primitive block type.
    fn from_block_response(block_response: N::BlockResponse) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
