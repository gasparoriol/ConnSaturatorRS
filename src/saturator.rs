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

    let totalRequests = self.config.requests as u64;
    let progresBar = ProgressBar::new(totalRequests);
    progresBar.set_style(
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

      let progresBarClone = progresBar.clone();

      let handle = tokio::spawn(async move {
        // here we acquire a permit
        let _permit = permit.acquire_owned().await.unwrap();

        let response = clonned_client.get(url).send().await;

        progresBarClone.inc(1);
        // here the permit is dropped and the slot is released
      });

      handles.push(handle);
    }

    let mut succesCounter = 0;
    let mut errorCounter = 0;
    


    //waiting for all requests to complete
    for handle in handles {
      match handle.await {
        Ok(_) => succesCounter += 1,
        Err(e) => errorCounter += 1,
      }
    }

    progresBar.finish_with_message("Done");
    let duration = start.elapsed();
    
    

    self.print_results(succesCounter, errorCounter, duration);
    println!("\nConnection saturation test completed\n");
  }
  
  fn print_results(&self, succesCounter: usize, errorCounter: usize, duration: Duration) {

    let totalRequests = succesCounter + errorCounter;
    let requestPerSecond = totalRequests as f64 / duration.as_secs_f64();

    println!("\n\nResults:");
    println!("{}", "=".repeat(30));
    println!("\nURL: {}", self.config.url);
    println!("Total requests: {}", totalRequests);
    println!("Total successful requests: {}", succesCounter);
    println!("Total failed requests: {}", errorCounter);
    println!("Total duration: {} ms", duration.as_millis());
    println!("Average duration: {} ms", duration.as_millis() / totalRequests as u128);
  } 
}