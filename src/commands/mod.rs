use crate::config::XFixConfig;

pub mod assign;
pub mod map;

pub trait XFixCommandDelegate {
    fn run(&self, config: &XFixConfig) -> Result<Option<XFixConfig>, Box<dyn std::error::Error>>;
}
