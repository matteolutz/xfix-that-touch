use crate::config::XFixConfig;

pub mod fix;

pub trait XFixCommandDelegate {
    fn run(&self, config: &XFixConfig) -> Result<(), Box<dyn std::error::Error>>;
}
