mod saturator;

use clap::Parser;
use saturator::{Config, ConnSaturator};

#[derive(Parser, Debug)]
#[command(name = "ConnSaturatorRS", about = "A simple connection saturator tester", long_about = None)]
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
    };

    // create saturator and run
    let saturator = ConnSaturator::new(config);
    saturator.run().await;
}

