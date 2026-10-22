//! Passive platform evidence adapters. These never open or renegotiate streams.
mod alsa;
pub use alsa::alsa_pcm_report;
