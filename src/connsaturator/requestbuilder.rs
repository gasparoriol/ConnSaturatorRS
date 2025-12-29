use crate::connsaturator::HttpMethods;
use crate::connsaturator::Config;
use crate::connsaturator::AuthMethods;
use crate::connsaturator::CustomHeaders;
use crate::connsaturator::OAuth2Config;

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, CONTENT_TYPE};

use std::time::Duration;

pub fn create_builder(client: &reqwest::Client, config: &Config) -> reqwest::RequestBuilder {
    let url = &config.url;
     
    
    let mut builder = match config.method {
        HttpMethods::Get => client.get(url),
        HttpMethods::Post => client.post(url),
        HttpMethods::Put => client.put(url),
        HttpMethods::Delete => client.delete(url),
    };

    if let Some(token) = &config.token {
        match token {
            AuthMethods::Bearer(token) => builder = builder.bearer_auth(token),
            AuthMethods::OAuth2 { config } => builder = builder.bearer_auth(&config.access_token),
            AuthMethods::APIKey { key } => builder = builder.header("X-API-Key", key),
            AuthMethods::Basic { username, password } => builder = builder.basic_auth(username, Some(password)),
        }
    }

    if let Some(body) = &config.body {
        builder = builder.body(body.clone());
    }

    builder = builder.timeout(Duration::from_secs(config.timeout));

    if let Some(header) = &config.header {
        builder = builder.header(&header.name.clone(), &header.value.clone());
    }

    if let Some(user_agent)= &config.user_agent {
        builder = builder.header(USER_AGENT, user_agent);
    }

    if HttpMethods::Get != config.method {
        let content_type = HeaderValue::from_str(&config.content_type).unwrap_or_else(|_| HeaderValue::from_static("application/json"));
        builder = builder.header(CONTENT_TYPE, content_type);
    }

 
    builder
}