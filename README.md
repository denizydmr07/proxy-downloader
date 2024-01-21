## Simple Proxy Server with Caching for .txt Files
This Rust program is a simple proxy server with caching capabilities, specifically designed for handling .txt files. It listens for incoming HTTP requests, logs them, and forwards the requests to the specified server. The program caches server responses for .txt files, improving performance by serving cached content.

### Features
* **HTTP Proxy** : Listens for incoming HTTP requests on a specified port.
* **Request Logging**: Logs each incoming request with details such as method, URL, and version.
* **Caching for .txt Files**: Stores server responses for .txt files in a local cache directory, avoiding redundant requests to the server for the same content.
* **Gzip Decompression**: Supports decompression of Gzip-encoded content.

### Usage

#### Prerequisites
* [Rust](https://www.rust-lang.org/)

#### Build
Clone the repository and navigate to the project directory:
```bash
git clone https://github.com/denizydmr07/proxy-downloader.git
cd proxy-downloader
```
Build the project using Cargo:
```bash
cargo build
```
#### Run
Run the proxy server with the desired port number:
```bash
cargo run <port>
```

### Configuration
* **Buffer Size**: The buffer size for reading from the TCP stream is set to 4 KB. You can adjust this by modifying the BUFFER_SIZE constant in the code
* **Cache Directory**: Cached .txt files are stored in the src/cache directory. You can change the cache directory by modifying the CACHE_DIR constant.
* **Log File**: Request logs are written to src/log.txt. Change the log file location by modifying the LOG_FILE constant.
* **Timeout**: The timeout for the TCP stream is set to 10 seconds (TIMEOUT). Adjust this value based on your preferences.

### Dependencies
* [flate2](https://crates.io/crates/flate2)
* [chrono](https://crates.io/crates/chrono)
