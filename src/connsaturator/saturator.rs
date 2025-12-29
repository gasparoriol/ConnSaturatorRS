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
  pub fn new(config: Config) -> Result<Self, reqwest::Error> {
    let client = Client::builder().danger_accept_invalid_certs(config.insecure).build()?;

    Ok(Self {
      config,
      client,
    })
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

        let start_request = Instant::now();
        let response = requestbuilder::create_builder(&clonned_client, &*config_for_thread).send().await;
        let duration = start_request.elapsed();
        
        progress_bar_clone.inc(1);
        // here the permit is dropped and the slot is released

        (response, duration)
      });

      handles.push(handle);
    }

    let mut succes_counter = 0;
    let mut error_counter = 0;
    let mut status_code: HashMap<String, u64> = HashMap::new();
    let mut latencies: Vec<Duration> = Vec::new();

    //waiting for all requests to complete
    for handle in handles {
      match handle.await {
        Ok((result_response, duration)) => {
          match result_response {
            Ok(response) => {
              latencies.push(duration);
              let status = response.status().to_string();
              *status_code.entry(status).or_insert(0) += 1;

              if response.status().is_success() {
                succes_counter += 1;
              } else {
                error_counter += 1;
              }
            
          },
          Err(e) => {
            error_counter += 1;
            *status_code.entry("Network Error".to_string()).or_insert(0) += 1;
          }
        }
        },
        Err(e) =>   {
          error_counter += 1;
          *status_code.entry("Panic Error".to_string()).or_insert(0) += 1;
        }
      } 
    }

    progress_bar.finish_with_message("Done");
    let duration = start.elapsed();
    
    latencies.sort();

    self.print_results(succes_counter, error_counter, duration, status_code, latencies);
    println!("\nConnection saturation test completed\n");
  }

  fn calculate_percentiles(&self, latencies: &Vec<Duration>) -> HashMap<String, f64> {
    let mut latencies = latencies.clone();
    latencies.sort();
    let mut percentiles = HashMap::new();
    percentiles.insert("p50".to_string(), latencies[latencies.len() / 2].as_millis() as f64);
    percentiles.insert("p90".to_string(), latencies[latencies.len() * 9 / 10].as_millis() as f64);
    percentiles.insert("p95".to_string(), latencies[latencies.len() * 95 / 100].as_millis() as f64);
    percentiles.insert("p99".to_string(), latencies[latencies.len() * 99 / 100].as_millis() as f64);

    percentiles
  }
  
  fn print_results(&self, succes_counter: usize, error_counter: usize, duration: Duration, status_code: HashMap<String, u64>, latencies: Vec<Duration>) {
    let percentiles = self.calculate_percentiles(&latencies);
    
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

    // To calculate average
    let total_duration_millis: Duration = latencies.iter().sum();
    
    let average_latency = total_duration_millis / latencies.len() as u32;

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
    println!("{:<35} {:.2} req/s", "Throughput (Requests per Second):", rps);
    println!("{:<35} {:.2} ms", "Average latency:", average_latency.as_millis() as f64);
    if latencies.len() > 0 {
      println!("{:<35} {:.2} ms", "p50 latency:", percentiles["p50"]);
      println!("{:<35} {:.2} ms", "p90 latency:", percentiles["p90"]);
      println!("{:<35} {:.2} ms", "p95 latency:", percentiles["p95"]);
      println!("{:<35} {:.2} ms", "p99 latency:", percentiles["p99"]);
    }
    self.print_histogram(latencies);
  } 

  fn print_histogram(&self, latencies: Vec<Duration>) {
    if (latencies.is_empty()) {
      return;
    }

    let min_latency = latencies[0].as_millis();
    let max_latency = latencies.last().unwrap().as_millis();
    let range = max_latency - min_latency;
    let bucket_count = 10;
    let step = range / bucket_count;

    println!("\nLatency Histogram:");
  
    for i in 0..bucket_count {
      let start_value = min_latency + (i * step);
      let end_value = if i == bucket_count - 1 { max_latency } else { min_latency + (i + 1) * step };

      let count = latencies.iter().filter(|latency| {
        let millis = latency.as_millis();
        if i == bucket_count - 1 {
          millis >= start_value && millis <= end_value
        } else {
          millis >= start_value && millis < end_value
        }
      }).count();

      let bar_width = if latencies.len() > 0 {
        (count * 30) / latencies.len()
      } else {
        0
      };

      let bar = "#".repeat(bar_width as usize);
      println!("  {:4}ms - {:4}ms  [{:30}] {}", start_value, end_value, bar, count);
      
    } 
    
    
  }
}