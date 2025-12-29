
mod connsaturator;
use connsaturator::{Config, HttpMethods, ConnSaturator, AuthMethods, CustomHeaders };
use clap::Parser;
use reqwest::header::HeaderValue;

#[derive(Parser, Debug)]
#[command(author, version, about = "A simple connection saturator tester", long_about = None)]
struct Cli {
    /// URL to test (Required)
    #[arg(short, long)]
    url: String,
    
    /// Total number of requests 
    #[arg(short, long, default_value_t = 100)]
    requests: usize,
    
    /// Number of concurrent requests 
    #[arg(short, long, default_value_t = 10)]
    concurrency: usize,

    /// HTTP method to use 
     #[arg(short, long, value_enum, default_value_t = HttpMethods::Get)]
    pub method: HttpMethods,

    /// Authentication method (Bearer, OAuth2, APIKey, Basic)
    #[arg(long, value_parser = AuthMethods::parse_auth)]
    pub token: Option<AuthMethods>,

    /// Custom headers
    #[arg(long, value_parser = CustomHeaders::parse_header)]
    pub header: Option<CustomHeaders>,

    /// Body of the request
    #[arg(short, long)]
    pub body: Option<String>,
    
    /// Timeout in seconds 
    #[arg(long, default_value_t = 30)]
    pub timeout: u64,

    /// User agent (Default: None)
    #[arg(long = "user-agent", short = 'a')]
    pub user_agent: Option<String>,

    /// Content type 
    #[arg(long = "content-type", short = 't', default_value = "application/json")]
    pub content_type: String,

    /// Insecure (Default: false)
    #[arg(long, short = 'i', default_value_t = false)]
    pub insecure: bool,
}

#[tokio::main]
pub async fn main() {
    // parse arguments
    let arguments = Cli::parse();

    // initialize saturator
    let config = Config {
        url: arguments.url,
        requests: arguments.requests,
        concurrency: arguments.concurrency, 
        token: arguments.token,
        method: arguments.method,
        body: arguments.body,
        timeout: arguments.timeout,
        header: arguments.header,
        user_agent: arguments.user_agent,
        content_type: arguments.content_type,
        insecure: arguments.insecure,
    };

    // create saturator and run
    match ConnSaturator::new(config) {
        Ok(saturator) => {
            saturator.run().await;
        }
        Err(e) => {
            eprintln!("Error crítico al configurar el saturator: {}", e);
            return;
        }
    }
}

