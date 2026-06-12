mod model;
mod paths;
mod settings;
mod validation;

pub use model::*;
pub use settings::{DEFAULT_AUTO_START, DEFAULT_SCROLLBACK, DEFAULT_SHELL};
pub use validation::validate_config;
