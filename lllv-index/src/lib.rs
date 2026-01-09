#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod errors;
mod evidence;
mod hash;
pub mod merkle;
mod pack;
mod search;

pub use errors::IndexError;
pub use evidence::{QueryEvidence, QueryRequest, QueryResult};
pub use pack::{IndexPack, IndexPackBuilder};
