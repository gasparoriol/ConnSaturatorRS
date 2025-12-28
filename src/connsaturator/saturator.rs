use reqwest::{Client};
use tokio::sync::Semaphore;
use std::sync::Arc;
use std::time::{Instant, Duration};
use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use crate::connsaturator::requestbuilder;
use crate::connsaturator::Config;



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

    let config = Arc::new(self.config.clone());
   

    for _ in 0..self.config.requests {
      let clonned_client = Arc::clone(&client);
  

      let permit = Arc::clone(&semaphore);

      let progress_bar_clone = progress_bar.clone();

      let config_for_thread = Arc::clone(&config);

      let handle = tokio::spawn(async move {
        // here we acquire a permit
        let _permit = permit.acquire_owned().await.unwrap();

        

        let response = requestbuilder::create_builder(&clonned_client, &*config_for_thread).send().await;
        
        progress_bar_clone.inc(1);
        // here the permit is dropped and the slot is released

        response
      });

      handles.push(handle);
    }

    let mut succes_counter = 0;
    let mut error_counter = 0;
    let mut status_code: HashMap<String, u64> = HashMap::new();


    //waiting for all requests to complete
    for handle in handles {
      match handle.await {
        Ok(Ok(response)) => {
          let status = response.status().to_string();
          *status_code.entry(status).or_insert(0) += 1;

          if response.status().is_success() {
            succes_counter += 1;
          } else {
            error_counter += 1;
          }
        }
        Ok(Err(_e)) => error_counter += 1,
        Err(e) =>eprintln!("Error de pánico en el hilo: {}", e),
      }
    }

    progress_bar.finish_with_message("Done");
    let duration = start.elapsed();
    
    

    self.print_results(succes_counter, error_counter, duration, status_code);
    println!("\nConnection saturation test completed\n");
  }
  
  fn print_results(&self, succes_counter: usize, error_counter: usize, duration: Duration, status_code: HashMap<String, u64>) {

    let total_requests = succes_counter + error_counter;
    let _request_per_second = total_requests as f64 / duration.as_secs_f64();

    let total_duration_secs = duration.as_secs_f64();

    let rps = if total_duration_secs > 0.0 {
      (succes_counter as f64 + error_counter as f64) / total_duration_secs
    } else {
      0.0
    };

    let success_rate = if total_requests > 0 {
      (succes_counter as f64 / total_requests as f64) * 100.0
    } else {
      0.0
    };

    println!("\nResults:");
    println!("{}", "=".repeat(60));
    
    println!("{:<35} {}", "Target URL:", self.config.url);
    println!("{:<35} {}", "Total Requests:", total_requests);
    println!("{:<35} {}", "Total successful requests:", succes_counter);
    println!("{:<35} {}", "Total failed requests:", error_counter);
    println!("\nStatus Code Distribution:");
    for (status, count) in &status_code {
      println!("{:<34}  {:<1} requests", format!("{}", status), count);
    }
    println!("\n{:<35} {:.2}%", "Success Rate:", success_rate);
    println!("{}", "-".repeat(60));
    println!("{:<35} {:.2} s", "Total duration:", total_duration_secs);
    println!("{:<35} {} ms", "Average latency:", duration.as_millis() / total_requests as u128);
    println!("{:<35} {:.2} req/s", "Throughput (Requests per Second):", rps);
  } 
}