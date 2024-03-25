use core::fmt;
use std::{
    collections::HashMap,
    fs,
    hash,
    io::{ BufReader, Read, Write },
    net::{ TcpListener, TcpStream },
    sync::{ Arc, Mutex },
    thread,
};
use colored::Colorize;
use lazy_static::lazy_static;
use serde_json::Value;
use serde::{ Serialize, Deserialize };
use once_cell::sync::Lazy;

type RouteHandler = fn(&HttpRequest) -> HttpResponse;

static mut routes: Lazy<HashMap<&str, RouteHandler>> = Lazy::new(|| HashMap::new());
static server: Lazy<HttpServer> = Lazy::new(|| HttpServer::new("127.0.0.1:7878"));
static mut data: Lazy<HashMap<String, Value>> = Lazy::new(||
    serde_json::from_str(fs::read_to_string("./src/users.json").unwrap().as_str()).unwrap()
);

#[derive(Clone)]
struct HttpServer {
    host_addr: String,
}

impl HttpServer {
    fn new(host_addr: &str) -> Self {
        return HttpServer { host_addr: host_addr.to_string() };
    }

    fn start(&'static self) {
        let listener: TcpListener = match TcpListener::bind(&self.host_addr) {
            Ok(v) => v,
            Err(e) => panic!("Error accepting the connection: {:?}", e),
        };
        println!("{}{}", "Server Listening ".bold().green(), format!(" on {}...", self.host_addr));
        for stream in listener.incoming() {
            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    eprint!("{}", "Error accepting the connection: ".bold().red());
                    eprintln!("{:?}", e);
                    continue;
                }
            };

            // println!("{}", "Connection Established!".bold().on_green());

            // self.handle_connection(stream);
            // let copy = self.clone();
            thread::spawn(|| self.handle_connection(stream));
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let mut req = HttpRequest::new();
        req.parse(&stream);
        // println!("{}", req);
        // let mut resp_json: HashMap<String, Value> = HashMap::new();
        // resp_json.insert("success".to_string(), Value::Bool(true));

        // handle the connection
        let route = &req.header[0]
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()[..2]
            .join(" ");
        println!("Route: {:?}", route);
        unsafe {
            match routes.get(route.as_str()) {
                Some(v) => {
                    let resp = v(&req);
                    resp.send(&mut stream);
                }
                None => { println!("No Such route found!") }
            }
        }
        // let resp = HttpResponse::new("200 Ok", Some(&self.data));
        // resp.send(&mut stream);
    }
}

struct HttpRequest {
    header: Vec<String>,
    body: Option<HashMap<String, Value>>,
}

impl HttpRequest {
    fn new() -> Self {
        HttpRequest {
            header: Vec::new(),
            body: None,
        }
    }

    fn parse(&mut self, stream: &TcpStream) {
        let mut buf_reader = BufReader::new(stream);
        let mut header_line = String::new();
        let mut byte_buf: [u8; 1] = [0; 1];
        loop {
            match buf_reader.read(&mut byte_buf) {
                Ok(bytes_read) if bytes_read > 0 => {
                    header_line.push(byte_buf[0] as char);
                    if (byte_buf[0] as char) == '\n' {
                        // println!("Read a header line: {}", header_line);
                        self.header.push(header_line.clone());
                        if header_line.trim().is_empty() {
                            // finish reading the header
                            break;
                        }
                        header_line.clear();
                    }
                }
                Ok(_) => {
                    // println!("End of stream!");
                    break;
                }
                Err(e) => {
                    eprintln!("{}{:?}", "Error reading from the stream: ".bold().red(), e);
                    break;
                }
            };
        }
        // read the body
        match self.header.iter().find(|line| line.contains("Content-Length")) {
            Some(l) => {
                let cont_len: i32 = l[16..].trim().parse().unwrap();
                let mut body_json = String::new();
                for _ in 0..cont_len {
                    buf_reader.read(&mut byte_buf).unwrap();
                    body_json.push(byte_buf[0] as char);
                }
                self.body = Some(serde_json::from_str(&body_json).unwrap());
            }
            None => {/* println!("No content type detected!") */}
        }
    }
}

impl fmt::Display for HttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", "   REQUEST HEADER   ".on_blue()).unwrap();
        for line in &self.header {
            write!(f, "{}", line).unwrap();
        }
        match &self.body {
            Some(hm) => {
                write!(f, "{}\n", "   REQUEST BODY   ".on_blue()).unwrap();
                writeln!(f, "{:#?}", hm).unwrap();
            }
            None => {}
        }
        Ok(())
    }
}

struct HttpResponse {
    response: String,
}

impl HttpResponse {
    fn new(status: &str, json: Option<&HashMap<String, Value>>) -> Self {
        let mut resp = String::new();
        resp.push_str(format!("HTTP/1.1 {}\r\n", status).as_str());
        match json {
            Some(hm) => {
                resp.push_str("Content-Type: application/json\r\n\r\n");
                let stringified = serde_json::to_string(&hm).expect("Failed to serialize");
                resp.push_str(&stringified);
            }
            None => {}
        }
        HttpResponse { response: resp }
    }

    fn send(&self, stream: &mut TcpStream) {
        stream.write_all(self.response.as_bytes()).expect("Failed to send the response!");
        // println!("Successfully written the response!");
    }
}

fn main() {
    println!("{}", "JSON Data Loaded!".bold().green());
    unsafe {
        routes.insert("GET /", |req| {
            HttpResponse::new(
                "200 Ok",
                Some(&data)
            )
        });
        routes.insert("POST /login", |req| {
            match &req.body {
                Some(b) => {
                    // get the body
                    let username = b.get("username").unwrap();
                    let password = b.get("password").unwrap();
                    println!("username: {}, password: {}", username, password);
                    let users = data.get("users").unwrap().as_array().unwrap();
                    // find the user with given username
                    let found_user = users
                        .iter()
                        .find(|u| u.as_object().unwrap().get("username").unwrap() == username);
                    let found_user = match found_user {
                        Some(u) => {
                            println!("Found User: {:#?}", u);
                            u
                        }
                        None => {
                            println!("User with such username not found!");
                            return HttpResponse::new(
                                "404 Not Found",
                                Some(&HashMap::from([("success".to_string(), Value::Bool(false))]))
                            );
                        }
                    };
                    // compare the passwords
                    let pw = found_user.get("password").unwrap();
                    if pw != password {
                        return HttpResponse::new(
                            "401 Not Authorized",
                            Some(&HashMap::from([("success".to_string(), Value::Bool(false))]))
                        );
                    }
                    HttpResponse::new(
                        "200 Ok",
                        Some(&HashMap::from([("success".to_string(), Value::Bool(true))]))
                    )
                }
                None => {
                    println!("POST /login called without a body!");
                    HttpResponse::new("400 Bad Request", None)
                }
            }
        });
    }
    server.start();
}
