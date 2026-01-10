//! LLM provider implementations.
//!
//! Each provider implements [`NeuralBackend`] for different LLM services.

pub mod local;

#[cfg(feature = "providers-openai")]
#[cfg_attr(docsrs, doc(cfg(feature = "providers-openai")))]
pub mod openai;

#[cfg(feature = "providers-anthropic")]
#[cfg_attr(docsrs, doc(cfg(feature = "providers-anthropic")))]
pub mod anthropic;
