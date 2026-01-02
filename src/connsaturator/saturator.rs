use reqwest::{Client};
use tokio::sync::Semaphore;
use std::sync::Arc;
use std::time::{Instant, Duration};
use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use crate::connsaturator::requestbuilder;
use crate::connsaturator::Config;
use crate::connsaturator::SummaryReport;
use crate::connsaturator::LoadResult;
use std::sync::atomic::{AtomicU64, Ordering};
use std::fs::File;
use std::io::Write;


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

    let total_requests = self.config.requests as u64;

    let concurrency = self.config.concurrency as usize;
    let url = self.config.url.clone();
   
    println!("\n\n🚀 Starting connection saturation test in {}", self.config.url);

    let warmup = if self.config.warmup == 0 {
        println!("\nWarmup: Applying 5% of total requests ({}) to stabilize connections...", total_requests * 5 / 100);
        total_requests * 5 / 100
    } else {
        self.config.warmup as u64
    };

    println!("Running with {} requests and {} concurrency", total_requests, concurrency);



    if warmup > 0 {
      let warmup_progress_bar = ProgressBar::new(warmup);
      warmup_progress_bar.set_style(
        ProgressStyle::default_bar()
          .template("{spinner:.green} {msg} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {eta}")
          .unwrap()
          .progress_chars("=> ")
      );
      warmup_progress_bar.set_message("Warmup");
      let _ = self.execute_requests(&url, warmup, concurrency, &warmup_progress_bar, true).await;
      tokio::time::sleep(Duration::from_millis(500)).await;
      warmup_progress_bar.finish_with_message("🔥 Warmup completed");
    } 

    let progress_bar = ProgressBar::new(total_requests);
    progress_bar.set_style(
      ProgressStyle::default_bar()
        .template("{spinner:.green} {msg} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {eta}")
        .unwrap()
        .progress_chars("=> ")
    );
    progress_bar.set_message("Running");

    let result = self.execute_requests(&url, total_requests, concurrency, &progress_bar, false).await;

    progress_bar.finish_with_message("📊 Benchmark finished");

    let latencies = result.latencies;
    let status_code = result.status_codes;
    let success_counter = result.success_counter;
    let error_counter = result.error_counter;
    let duration = result.duration;
    let total_bytes = result.total_bytes;

    self.print_results(success_counter, error_counter, duration, &status_code, &latencies);
    println!("\nConnection saturation test completed\n");

    if self.config.output {
      self.save_report_json(success_counter, error_counter, duration, &status_code, &latencies, total_bytes.load(Ordering::Relaxed));
      self.save_report_csv(success_counter, error_counter, duration, &status_code, &latencies, total_bytes.load(Ordering::Relaxed));
    } 
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
  
  fn print_results(&self, succes_counter: usize, error_counter: usize, duration: Duration, status_code: &HashMap<String, u64>, latencies: &Vec<Duration>) {
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

    let warmup = if self.config.warmup == 0 {
      self.config.requests * 5 / 100
    } else {
      self.config.warmup
    };

    println!("\nResults:");
    println!("{}", "=".repeat(60));
    
    println!("{:<35} {}", "Target URL:", self.config.url);
    println!("{:<35} {}", "Total Requests:", total_requests);
    println!("{:<35} {}", "Warmup Requests:", warmup);
    println!("{:<35} {}", "Total successful requests:", succes_counter);
    println!("{:<35} {}", "Total failed requests:", error_counter);
    println!("\nStatus Code Distribution:");
    for (status, count) in status_code {
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

  fn print_histogram(&self, latencies: &Vec<Duration>) {
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

   fn save_report_json(&self, succes_counter: usize, error_counter: usize, duration: Duration, status_code: &HashMap<String, u64>, latencies: &Vec<Duration>, total_bytes: u64) {
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


    let total_data_received_mb = self.format_bytes(total_bytes);
    let throughput_mbps = self.calculate_throughput(total_bytes, total_duration_secs);

    let warmup = if self.config.warmup == 0 {
      self.config.requests * 5 / 100
    } else {
      self.config.warmup
    };

    let summary_report = SummaryReport {
      target_url: self.config.url.clone(),
      warmup_requests: warmup as u64,
      total_requests: self.format_integer_value(total_requests as f64),
      total_successful_requests: self.format_integer_value(succes_counter as f64),
      total_failed_requests: self.format_integer_value(error_counter as f64),
      avg_latency_ms: average_latency.as_millis() as f64,
      success_rate: self.format_integer_value(success_rate),
      total_duration_secs: self.format_float_value(total_duration_secs),
      rps: self.format_float_value(rps),
      p50_latency_ms: percentiles["p50"],
      p90_latency_ms: percentiles["p90"],
      p95_latency_ms: percentiles["p95"],
      p99_latency_ms: percentiles["p99"],
      status_code_distribution: status_code.clone(),
      total_data_received_mb: self.format_float_value(total_data_received_mb),
      throughput_mbps: self.format_float_value(throughput_mbps),
      };

    let json = serde_json::to_string_pretty(&summary_report).unwrap();
    println!("\nSummary Report:\n{}", json);

    let mut file = std::fs::File::create("summary_report.json").unwrap();
    file.write_all(json.as_bytes()).unwrap(); 
  }

fn save_report_csv(&self, succes_counter: usize, error_counter: usize, duration: Duration, status_code: &HashMap<String, u64>, latencies: &Vec<Duration>, total_bytes: u64) {
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


    let total_data_received_mb = self.format_bytes(total_bytes);
    let throughput_mbps = self.calculate_throughput(total_bytes, total_duration_secs);

    let mut csv = String::new();
    
    let header = "target_url,total_requests,total_successful,total_failed,avg_latency_ms,success_rate,duration_secs,rps,p50,p90,p95,p99,total_mb,throughput_mbps";
        
    let row = format!(      
      "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
      self.config.url.clone(),
      self.format_integer_value(total_requests as f64),
      self.format_integer_value(succes_counter as f64),
      self.format_integer_value(error_counter as f64),
      average_latency.as_millis() as f64,
      self.format_integer_value(success_rate),
      self.format_float_value(total_duration_secs),
      self.format_float_value(rps),
      percentiles["p50"],
      percentiles["p90"],
      percentiles["p95"],
      percentiles["p99"],
      self.format_float_value(total_data_received_mb),
      self.format_float_value(throughput_mbps),
    );

    csv.push_str(&format!("{}\n{}", header, row));

    let mut file = std::fs::File::create("summary_report.csv").unwrap();
    file.write_all(csv.as_bytes()).unwrap(); 
  }

  
  fn format_bytes(&self, bytes: u64) -> f64 {
    let kb = bytes as f64 / 1024.0;
    let mb = kb / 1024.0;
    mb
  }

  fn calculate_throughput(&self, bytes: u64, duration_secs: f64) -> f64 {
    if duration_secs <= 0.0 {
        return 0.0;
    }
    let megabytes = bytes as f64 / (1024.0 * 1024.0);
    let megabits = megabytes * 8.0;
    let mbps = ((megabits / duration_secs) * 100.0).round() / 100.0;
    mbps
  }

  fn format_float_value(&self, value: f64) -> f64 {
    (value * 100.0).round() / 100.0
  }

    fn format_integer_value(&self, value: f64) -> u64 {
    (value.round() as u64)
  }

  async fn execute_requests(&self, target_url: &String,
    requests: u64,
    concurrency: usize,
    progress_bar: &ProgressBar,
    warmup: bool,
  ) -> LoadResult {
    let mut latencies: Vec<Duration> = if warmup { Vec::with_capacity(requests as usize) } else { Vec::new() };
    let mut status_codes = HashMap::new();
    let mut success_counter = 0;
    let mut error_counter = 0;
    let mut duration = Duration::from_secs(0);
    let mut total_bytes: AtomicU64 = AtomicU64::new(0);
    
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let client = Arc::new(self.client.clone());
    

    let config = Arc::new(self.config.clone());
    
    let start_time = Instant::now();

    let mut handles = Vec::new();

    for _ in 0..requests {
      let clonned_client = Arc::clone(&client);
      
      let clonned_progress_bar = progress_bar.clone();
      
      let clonned_config_for_thread = Arc::clone(&config);

      let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();


      let handle = tokio::spawn(async move {
        let _permit = permit;

        let request_start_time = Instant::now();
        let response = requestbuilder::create_builder(&clonned_client, &*clonned_config_for_thread).send().await;
        let duration = request_start_time.elapsed();
        
        clonned_progress_bar.inc(1);

        drop(_permit);

        match response {
          Ok(response) => {
            (Some(duration), Some(response))
          },
          Err(e) => (None, None)
        }
      });
      handles.push(handle);
    }
    
    for handle in handles {
      match handle.await {
        Ok((duration_option, result_response)) => {
          if !warmup {
            match result_response {
              Some(response) => {
                let status = response.status().to_string();
                if response.status().is_success() {
                  success_counter += 1;
                } else {
                  error_counter += 1;
                }

                if let Some(len) = response.content_length() {
                  total_bytes.fetch_add(len, Ordering::Relaxed);
                } 

                if let Some(d) = duration_option {
                  latencies.push(d);
                }

                *status_codes.entry(status).or_insert(0) += 1;
                
              },
              None => {
                error_counter += 1;
                *status_codes.entry("Network Error".to_string()).or_insert(0) += 1;
              }
            }
          }
        },
        Err(e) => {
          error_counter += 1;
          *status_codes.entry("Panic Error".to_string()).or_insert(0) += 1;
        }
      }
    }
    
    duration = start_time.elapsed();
    latencies.sort();

    LoadResult {
      latencies,
      status_codes,
      success_counter,
      error_counter,
      duration,
      total_bytes,
    }
  }
}