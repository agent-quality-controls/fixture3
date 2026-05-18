//! Sequential `DDMin` reducer for ordered candidate lists.
//!
//! The crate reduces a list while preserving an oracle-defined property.
//! It does not inspect candidates, mutate files, or run commands by itself.
//! Callers provide candidates and an oracle that reports whether a candidate
//! list is still interesting.

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
