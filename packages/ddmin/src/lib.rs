//! Generic sequential `DDMin` list reducer.

#[cfg(feature = "algorithm")]
mod algorithm;
mod types;

#[cfg(feature = "algorithm")]
pub use algorithm::ddmin;
#[cfg(feature = "algorithm")]
pub use types::{
    DdminGuarantee, DdminInput, DdminOptions, DdminOracle, DdminOutput, DdminStats,
    DdminStopReason, OracleOutcome, UnresolvedReason,
};
