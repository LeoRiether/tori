pub mod controller;
mod output;
mod resampler;
pub mod source;

use controller::Controller;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Player {
    pub controller: Controller,
}
