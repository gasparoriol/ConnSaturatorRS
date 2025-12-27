
mod connsaturator;
use connsaturator::{Config, HttpMethods, ConnSaturator, AuthMethods, CustomHeaders };
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "A simple connection saturator tester", long_about = None)]
struct Cli {
    /// URL to test
    #[arg(short, long)]
    url: String,
    
    /// Total number of requests
    #[arg(short, long, default_value_t = 100)]
    requests: usize,
    
    /// Number of concurrent requests
    #[arg(short, long, default_value_t = 10)]
    concurrency: usize,

    /// Método HTTP a utilizar
    #[arg(short, long, value_enum, default_value_t = HttpMethods::Get)]
    pub method: HttpMethods,

    /// Token de autenticación (Bearer, OAuth2, APIKey, Basic)
    #[arg(long, value_parser = AuthMethods::parse_auth)]
    pub token: Option<AuthMethods>,

    #[arg(long, value_parser = CustomHeaders::parse_header)]
    pub header: Option<CustomHeaders>,

    /// Body de la petición
    #[arg(short, long)]
    pub body: Option<String>,
    
    /// Timeout por petición en segundos
    #[arg(long, default_value_t = 30)]
    pub timeout: u64,
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
    };

    // create saturator and run
    let saturator = ConnSaturator::new(config);
    saturator.run().await;
}

