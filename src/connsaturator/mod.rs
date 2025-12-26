
pub mod saturator; 
pub mod requestbuilder;

use clap::ValueEnum;

pub use saturator::ConnSaturator;
pub use requestbuilder::create_builder;


//Methods
#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum HttpMethods {
  Get,
  Post,
  Put,
  Delete,
}

// internal configuration
pub struct Config {
  pub url: String,
  pub requests: usize,
  pub concurrency: usize,
  pub token: Option<String>,
  pub method: HttpMethods,
  pub body: Option<String>,
  pub timeout: u64,
}