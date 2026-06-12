//! Core DevPane library for loading, validating, resolving, and running
//! `.dpane` terminal workspace files.

/// Configuration loading, YAML models, validation, settings, and path helpers.
pub mod config;

/// Pure formatting helpers for CLI output.
pub mod output;

/// Process launch planning, lifecycle state transitions, and process runners.
pub mod process;

/// Resolved workspace models and runtime pane state.
pub mod workspace;
