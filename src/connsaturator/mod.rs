
pub mod saturator; 
pub mod requestbuilder;

use clap::ValueEnum;

pub use saturator::ConnSaturator;
pub use requestbuilder::create_builder;

use reqwest::header::{HeaderName, HeaderValue};
use std::str::FromStr;

//Methods
#[derive(ValueEnum, Clone, Debug, Copy, PartialEq)]
pub enum HttpMethods {
  Get,
  Post,
  Put,
  Delete,
}

#[derive(Clone, Debug)]
pub enum AuthMethods {
  Bearer(String),
  OAuth2 {config: OAuth2Config, },
  APIKey {key: String, },
  Basic {username: String, password: String, },
}

impl AuthMethods {
  pub fn parse_auth(token_entry: &str) -> Result<Self, String> {
    let parts: Vec<&str> = token_entry.splitn(2, ' ').collect();
    
    if parts.len() < 2 {
      return Err("Invalid token entry format".to_string());
    }

    let type_token = parts[0].to_lowercase();
    let token = parts[1].to_string();

    match type_token.as_str() {
      "bearer" => Ok(AuthMethods::Bearer(token)),
      "oauth2" => Ok(AuthMethods::OAuth2 {config: OAuth2Config { client_id: token.to_string(), access_token: token.to_string(), refresh_token: token.to_string(), scope: token.to_string(), token_type: token.to_string() }}),
      "apikey" => Ok(AuthMethods::APIKey {key: token.to_string()}),
      "basic" => {
        let parts: Vec<&str> = token.splitn(2, ':').collect();
        if parts.len() < 2 {
          return Err("Invalid basic token entry format".to_string());
        }
        Ok(AuthMethods::Basic {username: parts[0].to_string(), password: parts[1].to_string()}) 
      },
      _ => Err("Invalid token entry format".to_string()),
    } 

  }
}

#[derive(Clone, Debug)]
pub struct OAuth2Config {
  pub client_id: String,
  pub access_token: String,
  pub refresh_token: String,
  pub scope: String,
  pub token_type: String,
}

#[derive(Clone, Debug)]
pub struct CustomHeaders {
  pub name: HeaderName,
  pub value: HeaderValue,
}

impl CustomHeaders {
  pub fn parse_header(header_entry: &str) -> Result<Self, String> {
    let parts: Vec<&str> = header_entry.splitn(2, ':').collect();
    
    if parts.len() < 2 {
      return Err("Invalid header entry format".to_string());
    }

    let name = HeaderName::from_str(parts[0]).unwrap();
    let value = HeaderValue::from_str(parts[1]).unwrap();

    Ok(CustomHeaders { name, value })
  }
}

// internal configuration
#[derive(Clone, Debug)]
pub struct Config {
  pub url: String,
  pub requests: usize,
  pub concurrency: usize,
  pub token: Option<AuthMethods>,
  pub method: HttpMethods,
  pub body: Option<String>,
  pub timeout: u64,
  pub header: Option<CustomHeaders>,
  pub user_agent: Option<String>,
  pub content_type: String,
  pub insecure: bool,
}