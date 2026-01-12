use crate::config::XFixConfig;

pub mod assign;

pub trait XFixCommandDelegate {
    fn run(&self, config: &XFixConfig) -> Result<(), Box<dyn std::error::Error>>;
}
