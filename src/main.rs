use flate2::read::GzDecoder;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

const BUFFER_SIZE: u16 = 4096; // 4 KB
const CACHE_DIR: &str = "src/cache";
const LOG_FILE: &str = "src/log.txt";
const TIMEOUT: u64 = 10; // 10 seconds

fn get_request(stream: &mut TcpStream) -> std::io::Result<String> {
    // create a buffer to store the request
    let mut buffer = [0; BUFFER_SIZE as usize];

    // read the request from the stream
    let _ = stream.read(&mut buffer)?;

    // convert the request to a string
    let request = String::from_utf8_lossy(&buffer[..]);

    // return the request
    Ok(request.to_string())
}

fn check_version(version: &str) -> bool {
    // check if the version is valid
    if version != "HTTP/1.1" {
        return false;
    }

    // return true if the version is valid
    true
}

fn open_file(path: &str) -> std::io::Result<File> {
    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .expect("Could not open file."))
}

// a function to extract method, url, and version from the request
fn parse_request(request: &str) -> (String, String, String) {
    // split the request into lines
    let lines: Vec<&str> = request.split("\r\n").collect();

    // get the first line
    let first_line = lines[0];

    // split the first line into words
    let words: Vec<&str> = first_line.split(" ").collect();

    // get the method, url, and version
    let method = words[0].to_string();
    let url = words[1].to_string();
    let version = words[2].to_string();

    // return the method, url, and version
    (method, url, version)
}

// get the file name from the url
fn get_file_name(url: &str) -> String {
    // split the url into parts
    let parts: Vec<&str> = url.split("/").collect();

    // get the last part
    let last_part = parts[parts.len() - 1];

    // return the last part
    last_part.to_string()
}

fn get_server_name(url: &str) -> String {
    // split the url into parts
    let parts: Vec<&str> = url.split("/").collect();

    if parts.len() < 3 {
        return "".to_string();
    }

    // get the server name
    let server_name = parts[2];

    // return the server name
    server_name.to_string()
}

fn log_request(request: &str) -> std::io::Result<()> {
    // get the current time
    let now = chrono::Local::now();

    // get the current time as a string
    let now = now.format("%Y-%m-%d %H:%M:%S").to_string();

    // create a new variable to store the request, remove the encoded part
    let _request = request.split("Accept-Encoding:").collect::<Vec<&str>>()[0];

    // create the log message
    let log_message = format!("{}\n{}\n", now, _request);

    // open the log file
    let mut file = open_file(LOG_FILE)?;

    // write the log message to the file
    let _ = file
        .write_all(log_message.as_bytes())
        .expect("Could not write to log file.");

    // stop the file
    let _ = file.sync_all();
    // return
    Ok(())
}

fn check_cache(file_name: &str) -> bool {
    // get the path to the file
    let path = format!("{}/{}", CACHE_DIR, file_name);

    // check if the file exists
    if std::path::Path::new(&path).exists() {
        return true;
    }

    // return false if the file does not exist
    false
}

fn handle_connection(stream: &mut TcpStream) {
    // set the timeout for the stream
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(TIMEOUT)))
        .unwrap();

    // get the request from the stream
    let request = match get_request(stream) {
        Ok(request) => request,
        Err(_) => return,
    };

    // parse the request
    let (method, url, version) = parse_request(&request);

    // check if the version is valid
    if !check_version(&version) {
        println!("Invalid version: {}", version);
        return;
    }

    // log the request
    if let Err(error) = log_request(&request) {
        println!("Could not log request.");
        println!("Error: {}", error);
    }

    // get server name and file name
    let server_name = get_server_name(&url);
    let file_name = get_file_name(&url);

    // if there is no file, return after giving proper message on the terminal
    if file_name == "" {
        println!("No file name acquired, aborting the connection.");
        return;
    }

    // if there is no server name, return after giving proper message on the terminal
    if server_name == "" {
        println!("No server name acquired, aborting the connection.");
        return;
    }

    // print the server name and file name
    println!("Server name: {}", server_name);
    println!("File name: {}", file_name);

    // check if the file is cached, if it is, return the cached file
    if check_cache(&file_name) {
        println!("{} is cached, returning the cached file.", file_name);

        // get the path to the file
        let path = format!("{}/{}", CACHE_DIR, file_name);

        // read the file
        let file = match open_file(&path) {
            Ok(file) => file,
            Err(_) => {
                println!("Could not open file.");
                return;
            }
        };

        // create the response
        let response = format!("{} 200 OK\r\n\r\n", version);

        // send the response
        let _ = stream.write(response.as_bytes());

        // send the file
        let _ = std::io::copy(&mut BufReader::new(file), stream);

        // return
        return;
    }

    // create a connection to the server in port 80
    let mut server_stream = match TcpStream::connect(format!("{}:80", server_name)) {
        Ok(server_stream) => server_stream,
        Err(_) => {
            println!("Could not connect to server: {}", server_name);
            return;
        }
    };

    // create the request to send to the server
    let request = format!("{} {} HTTP/1.1\r\nHost: {}\r\nConnection: keep-alive\r\nAccept-Encoding: gzip, deflate\r\n\r\n", method, url, server_name);

    // send the request to the server
    let _ = server_stream.write(request.as_bytes());

    // recieve the response from the server
    let mut response = String::new();
    let _ = server_stream.read_to_string(&mut response);

    // get the status code
    let status_code = response.split(" ").collect::<Vec<&str>>()[1];

    println!("Status code: {}", status_code);

    // if status is not 200, return after giving proper message on the terminal
    if status_code != "200" {
        println!("Retrieving the file from the server is unsuccessful. Aborting the connection with status code {}.", status_code);
        // send the response
        let _ = stream.write(response.as_bytes());

        // return
        return;
    }

    // separate the response into headers and body
    let parts: Vec<&str> = response.split("\r\n\r\n").collect();

    // get the headers and body
    let headers = parts[0];
    let body = parts[1];

    // get content endoding and content length from the headers
    let mut content_encoding = "";
    let mut _content_length = "";

    // split the headers into lines
    let lines: Vec<&str> = headers.split("\r\n").collect();

    // get the content encoding and content length
    for line in lines {
        // split the line into words
        let words: Vec<&str> = line.split(": ").collect();

        // get the first word
        let first_word = words[0];

        // check if the first word is content encoding
        if first_word == "Content-Encoding" {
            // get the content encoding
            content_encoding = words[1];
        }

        // check if the first word is content length
        if first_word == "Content-Length" {
            // get the content length
            _content_length = words[1];
        }
    }

    // if status is 200, download the file using the proper method
    if content_encoding == "gzip" {
        // create a decoder
        let mut decoder = GzDecoder::new(body.as_bytes());
        let mut buffer = Vec::new(); // create a buffer to store the decoded file

        let _ = decoder.read_to_end(&mut buffer); // read the decoded file

        let path = format!("{}/{}", CACHE_DIR, file_name); // write the file to the cache
        let mut file = match open_file(&path) {
            // open the file
            Ok(file) => file,
            Err(_) => {
                println!("Could not open file.");
                return;
            }
        };

        let _ = file.write_all(&buffer); // write the file to the cache

        let response = format!("{} 200 OK\r\n\r\n", version); // create the response
        let _ = stream.write(response.as_bytes()); // send the response
        let _ = stream.write(&buffer); // send the file
    } else {
        let path = format!("{}/{}", CACHE_DIR, file_name); // write the file to the cache
        let mut file = match open_file(&path) {
            // open the file
            Ok(file) => file,
            Err(_) => {
                println!("Could not open file.");
                return;
            }
        };

        let _ = file.write_all(body.as_bytes()); // write the file to the cache

        let response = format!("{} 200 OK\r\n\r\n", version); // create the response
        let _ = stream.write(response.as_bytes()); // send the response
        let _ = stream.write(body.as_bytes()); // send the file
    }

    println!(
        "File is retrieved successfully with status code {}.",
        status_code
    );

    // close the connection to the server
    drop(server_stream);

    // return
    return;
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // check if the user has provided a port number
    if args.len() < 2 {
        println!("Usage: cargo run <port>");
        return Ok(());
    }

    // get the port number from the command line
    let port = &args[1];

    // check if the port number is valid
    let port: u32 = match port.parse() {
        Ok(port) => port,
        Err(_) => {
            println!("Please provide a valid port number.");
            return Ok(());
        }
    };

    // check if the port number is in the valid range
    if port < 1024 || port > 65535 {
        println!("Please provide a port number that is in the range.");
        return Ok(());
    }

    // create a listener on the port
    let adress = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(adress) {
        Ok(listener) => listener,
        Err(_) => {
            println!("Could not bind to port {}, internal error.", port);
            return Ok(());
        }
    };

    println!("-----------------------------");
    println!("Listening on port {}.", port);
    println!("-----------------------------");

    loop {
        // accept connections from clients
        let (mut stream, _) = match listener.accept() {
            Ok(stream) => stream,
            Err(_) => {
                println!("Could not accept connection, internal error.");
                continue;
            }
        };

        // handle the connection in a new thread
        thread::spawn(move || {
            println!("-----------------------------");
            handle_connection(&mut stream);
            println!("-----------------------------");
        });

        // sleep for 1 second
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
