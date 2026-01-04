
# ConnSaturatorRS: A Rust Resiliency Testing Tool

A high-performance connection saturator written in Rust, designed to evaluate the load capacity and stress resilience of web services. This project implements an efficient HTTP GET request engine and serves as a deep dive into asynchronous concurrency, memory safety, and connection pool behavior under heavy load.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

## ⚠️ Legal Disclaimer

This tool is strictly for **educational and research purposes** within the field of cybersecurity. It is designed to help developers and security professionals test application resiliency and understand network bottlenecks.
**Using this tool against any infrastructure without explicit prior authorization is illegal and may result in criminal charges.** The author assumes no liability for any damage or misuse caused by this software.

## 🚀 Features

- **Asynchronous Engine**: Built on `Tokio` to manage thousands of concurrent connections without blocking system threads.
- **Efficient Resource Management**: Leverages `std::sync::Arc` to safely share the HTTP client across asynchronous tasks.
- **Backpressure & Flow Control**: Implements `tokio::sync::Semaphore` to strictly manage concurrency levels and prevent local resource exhaustion.
- **Real-time Progress Tracking**: Interactive CLI featuring dynamic progress bars via `indicatif`, providing instant feedback on success and failure rates.
- **Support for HTTP Methods**: Adding POST, PUT, and DELETE support with custom JSON payloads.
- **Detailed Analytics**: Reporting status code distribution (e.g., 2xx, 4xx, 5xx) and percentiles (p95, p99) for latency and histogram of latencies.
- **Custom Headers**: Ability to pass authentication tokens or custom User-Agents via CLI.
- **Report Export**: Exporting results to JSON or CSV formats for further analysis.
- **Warmup**: Implementing a warmup phase to ensure the target server is ready to handle the load.

## 🧠 Lessons Learned

During the development of this tool, I explored several core Rust and systems engineering concepts:
- **Concurrency vs. Parallelism**: Mastering how Rust schedules asynchronous tasks vs. OS threads.
- **Ownership & Lifetimes**: Solving the challenges of sharing resources and managing object lifetimes in an async context.
- **I/O Optimization**: Understanding the performance gap between blocking and non-blocking I/O operations.
- **Robust Error Handling**: Implementing idiomatic Rust error patterns to ensure the tool remains stable even when the target server fails.
- **DoS Mitigation Strategies**: Analyzed server-side defenses such as *Rate Limiting* and *Circuit Breakers* (specifically in Spring Boot) to understand how to protect applications from high-traffic surges.


## 📦 Installation

```bash
1. **Clone the repository:**
   git clone https://github.com/gasparoriol/ConnSaturatorRS.git


2. **Navigate to the directory:**
  cd ConnSaturatorRS




3. **Build the production release:**
  cargo build --release

```



## 🛠️ Usage

This is a CLI-based tool. You can run it directly using `cargo`:

```bash
cargo run -- --url http://localhost:8080/api --requests 1000 --concurrency 50

```

#### Parameters:
```bash
Usage: ConnSaturatorRS [OPTIONS] --url <URL>

Options:
  -u, --url <URL>                    URL to test (Required)
  -r, --requests <REQUESTS>          Total number of requests [default: 100]
  -c, --concurrency <CONCURRENCY>    Number of concurrent requests [default: 10]
  -m, --method <METHOD>              HTTP method to use [default: get] [possible values: get, post, put, delete]
      --token <TOKEN>                Authentication method (Bearer, OAuth2, APIKey, Basic)
      --header <HEADER>              Custom headers
  -b, --body <BODY>                  Body of the request
      --timeout <TIMEOUT>            Timeout in seconds [default: 30]
  -a, --user-agent <USER_AGENT>      User agent (Default: None)
  -t, --content-type <CONTENT_TYPE>  Content type [default: application/json]
  -i, --insecure                     Insecure (Default: false)
  -o, --output                       Output report (Default: false)
  -w, --warmup <WARMUP>              Warmup requests (Default: 0) [default: 0]
  -h, --help                         Print help
  -V, --version                      Print version
```

## 📊 Execution Example

```text
Command: target/debug/ConnSaturatorRS --url http://localhost:3000/protected --requests 100000 --concurrency 25 --method get --token [MASKED] --output --warmup 1500


🚀 Starting connection saturation test in http://localhost:3000/protected
Running with 100000 requests and 25 concurrency
  🔥 Warmup completed [00:00:00] ======================================== 1500/1500 0s 
  📊 Benchmark finished [00:00:07] ======================================== 100000/100000 0s 
Results:
============================================================
Target URL:                         http://localhost:3000/protected
Total Requests:                     100000
Warmup Requests:                    1500
Total successful requests:          100000
Total failed requests:              0

Status Code Distribution:
200 OK                              100000 requests

Success Rate:                       100.00%
------------------------------------------------------------
Total duration:                     7.25 s
Throughput (Requests per Second):   13798.75 req/s
Average latency:                    1.00 ms
p50 latency:                        1.00 ms
p90 latency:                        2.00 ms
p95 latency:                        2.00 ms
p99 latency:                        10.00 ms

Latency Histogram:
     0ms -    4ms  [############################# ] 97432
     4ms -    8ms  [                              ] 1235
     8ms -   12ms  [                              ] 418
    12ms -   16ms  [                              ] 214
    16ms -   20ms  [                              ] 137
    20ms -   24ms  [                              ] 96
    24ms -   28ms  [                              ] 257
    28ms -   32ms  [                              ] 201
    32ms -   36ms  [                              ] 6
    36ms -   48ms  [                              ] 4

Connection saturation test completed

```

## Mitigation
Want to know more about the mitigation strategies? Check out the [MITIGATION.md](./MITIGATION.md) file.


## 📄 License
This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for more details.
