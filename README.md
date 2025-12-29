
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

## 🧠 Lessons Learned

During the development of this tool, I explored several core Rust and systems engineering concepts:
- **Concurrency vs. Parallelism**: Mastering how Rust schedules asynchronous tasks vs. OS threads.
- **Ownership & Lifetimes**: Solving the challenges of sharing resources and managing object lifetimes in an async context.
- **I/O Optimization**: Understanding the performance gap between blocking and non-blocking I/O operations.
- **Robust Error Handling**: Implementing idiomatic Rust error patterns to ensure the tool remains stable even when the target server fails.
- **DoS Mitigation Strategies**: Analyzed server-side defenses such as *Rate Limiting* and *Circuit Breakers* (specifically in Spring Boot) to understand how to protect applications from high-traffic surges.

## 🛠️ Usage

### Installation

```basg
1. **Clone the repository:**
   git clone https://github.com/gasparoriol/ConnSaturatorRS.git


2. **Navigate to the directory:**
  cd ConnSaturatorRS




3. **Build the production release:**
  cargo build --release

```



### Running the Tool

This is a CLI-based tool. You can run it directly using `cargo`:

```bash
cargo run -- --url http://localhost:8080/api --requests 1000 --concurrency 50

```

#### Parameters:

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
  -h, --help                         Print help
  -V, --version                      Print version

## 📊 Execution Example

```text
Starting connection saturation test on http://localhost:8080/api
Running with 100 requests and 10 concurrency
  [00:00:04] ======================================== 100/100 0s                                                                                                                                                            
Results:
============================================================
Target URL:                         http://localhost:8080/api
Total Requests:                     100
Total successful requests:          99
Total failed requests:              1

Status Code Distribution:
502 Bad Gateway                     1 requests
200 OK                              99 requests

Success Rate:                       99.00%
------------------------------------------------------------
Total duration:                     4.31 s
Throughput (Requests per Second):   23.22 req/s
Average latency:                    408.00 ms
p50 latency:                        360.00 ms
p90 latency:                        884.00 ms
p95 latency:                        1098.00 ms
p99 latency:                        1711.00 ms

Latency Histogram:
    92ms -  253ms  [#############                 ] 44
   253ms -  414ms  [####                          ] 16
   414ms -  575ms  [#####                         ] 18
   575ms -  736ms  [##                            ] 7
   736ms -  897ms  [#                             ] 6
   897ms - 1058ms  [                              ] 3
  1058ms - 1219ms  [#                             ] 4
  1219ms - 1380ms  [                              ] 0
  1380ms - 1541ms  [                              ] 1
  1541ms - 1711ms  [                              ] 1

Connection saturation test completed

```

## 🔮 Future Work

To enhance the diagnostic capabilities of ConnSaturatorRS, the following features are planned:
* **Report Export**: Exporting results to JSON or CSV formats for further analysis.

## Mitigation
Want to know more about the mitigation strategies? Check out the [MITIGATION.md](./MITIGATION.md) file.


## 📄 License
This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for more details.
