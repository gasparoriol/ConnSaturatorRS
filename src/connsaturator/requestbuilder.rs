use crate::connsaturator::HttpMethods;
use crate::connsaturator::Config;

pub fn create_builder(client: &reqwest::Client, config: &Config) -> reqwest::RequestBuilder {
    let url = &config.url;
    
    let mut builder = match config.method {
        HttpMethods::Get => client.get(url),
        HttpMethods::Post => client.post(url),
        HttpMethods::Put => client.put(url),
        HttpMethods::Delete => client.delete(url),
    };

    if let Some(token) = &config.token {
        builder = builder.bearer_auth(token);
    }
    
    builder
}