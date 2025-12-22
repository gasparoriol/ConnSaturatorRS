use reqwest::Client;
use tokio::sync::Semaphore;
use std::sync::Arc;
use std::time::{Instant, Duration};
use indicatif::{ProgressBar, ProgressStyle};

// internal configuration
pub struct Config {
  pub url: String,
  pub requests: usize,
  pub concurrency: usize,
}

pub struct ConnSaturator {
  config: Config,
  client: Client,
}

impl ConnSaturator {
  //constructor: initialize the connections pool 
  pub fn new(config: Config) -> Self {
    Self {
      config,
      client: Client::new(),
    }
  }

  pub async fn run(&self) {
    println!("\n\nStarting connection saturation test in {}", self.config.url);
    println!("Running with {} requests and {} concurrency", self.config.requests, self.config.concurrency);

    let total_requests = self.config.requests as u64;
    let progress_bar = ProgressBar::new(total_requests);
    progress_bar.set_style(
      ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {eta}")
        .unwrap()
        .progress_chars("=> ")
    );
    

    let start = Instant::now();

    let semaphore = Arc::new(Semaphore::new(self.config.concurrency));
    let client = Arc::new(self.client.clone());
    let mut handles = vec![];

    for _ in 0..self.config.requests {
      let clonned_client = Arc::clone(&client);
      let url = self.config.url.clone();

      let permit = Arc::clone(&semaphore);

      let progress_bar_clone = progress_bar.clone();

      let handle = tokio::spawn(async move {
        // here we acquire a permit
        let _permit = permit.acquire_owned().await.unwrap();

        let _response = clonned_client.get(url).send().await;

        progress_bar_clone.inc(1);
        // here the permit is dropped and the slot is released
      });

      handles.push(handle);
    }

    let mut succes_counter = 0;
    let mut error_counter = 0;
    


    //waiting for all requests to complete
    for handle in handles {
      match handle.await {
        Ok(_) => succes_counter += 1,
        Err(_e) => error_counter += 1,
      }
    }

    progress_bar.finish_with_message("Done");
    let duration = start.elapsed();
    
    

    self.print_results(succes_counter, error_counter, duration);
    println!("\nConnection saturation test completed\n");
  }
  
  fn print_results(&self, succes_counter: usize, error_counter: usize, duration: Duration) {

    let total_requests = succes_counter + error_counter;
    let _request_per_second = total_requests as f64 / duration.as_secs_f64();

    println!("\n\nResults:");
    println!("{}", "=".repeat(30));
    println!("\nURL: {}", self.config.url);
    println!("Total requests: {}", total_requests);
    println!("Total successful requests: {}", succes_counter);
    println!("Total failed requests: {}", error_counter);
    println!("Total duration: {} ms", duration.as_millis());
    println!("Average duration: {} ms", duration.as_millis() / total_requests as u128);
  } 
}