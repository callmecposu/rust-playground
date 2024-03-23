use std::{
    collections::HashMap,
    fs,
    io::{ prelude::*, BufReader },
    net::{ TcpListener, TcpStream },
    thread
};
use serde::{ Deserialize, Serialize };
use serde_json::Value;
use colored::Colorize;

static mut users: Vec<User> = Vec::new();

#[derive(Debug, Deserialize, Serialize)]
struct User {
    username: String,
    password: String,
}

fn print_http_request(req: &[String]) {
    for line in req {
        print!("\t{}", line);
    }
}

fn post_login(hmap: HashMap<String, Value>) -> Option<()> {
    let username = hmap.get("username")?;
    println!("\tusername: {}", username);
    let password = hmap.get("password")?;
    println!("\tpassword: {}", password);
    unsafe {
        let user = users.iter().find(|u| u.username == username.as_str().unwrap().to_string())?;
        println!("User found : {:#?}", user);
    }
    Some(())
}

fn create_response(status: &str, json: &str) -> String {
    let mut resp = String::from("HTTP/1.1 ");
    resp.push_str(format!("{}\r\n", status).as_str());
    resp.push_str("Content-Type: applicaiton/json\r\n\r\n");
    resp.push_str(json);
    return resp;
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&stream);
    let mut header: Vec<String> = Vec::new();
    let mut line = String::new();
    let mut byte_buffer = [0; 1];
    loop {
        match buf_reader.read(&mut byte_buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                let byte = byte_buffer[0]; // Extract the byte read
                // Process the byte here
                line.push(byte as char);
                if (byte as char) == '\n' {
                    header.push(line.to_string());
                    println!("Line: {}", line);
                    if line.trim().is_empty() {
                        // finished reading header
                        break;
                    }
                    line.clear();
                }
            }
            Ok(_) => {
                // End of stream
                println!("End of stream");
                break;
            }
            Err(err) => {
                // Handle errors
                eprintln!("Error reading from stream: {}", err);
                break;
            }
        }
    }

    println!("Request Header:");
    print_http_request(&header);

    let mut hmap: HashMap<String, Value> = HashMap::new();

    // parse the header
    if !header[0].trim().contains("GET") {
        let cont_len_str = header
            .iter()
            .find(|h| h.contains("Content-Length:"))
            .unwrap();
        println!("{}", cont_len_str);
        let cont_len: i32 = cont_len_str.trim()[16..].parse().unwrap();
        println!("clen: {}", cont_len);

        // read the body
        let mut body = String::new();
        line.clear();
        for _ in 0..cont_len {
            buf_reader.read(&mut byte_buffer).unwrap();
            body.push(byte_buffer[0] as char);
        }

        println!("\nRequest Body:");
        println!("{}", body);

        // deserialize the body
        hmap = serde_json::from_str(&body).unwrap();
        println!("hmap: {:?}", hmap);
    }

    // map the logic to the route
    match
        header[0]
            .trim()
            .split(' ')
            .take(2)
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(" ")
            .as_str()
    {
        "POST /login" => {
            println!("{}", "LOGIN ROUTE".on_bright_green());
            match post_login(hmap) {
                Some(()) => println!("{} - {}", "LOGIN_ROUTE".on_blue().bold(), "all good"),
                None => println!("{} - {}", "LOGIN ROUTE".on_red().bold(), "smth went wrong"),
            }
        }
        "GET /" => {
            println!("{}", "GET / ROUTE".on_bright_green());
        }
        _ => println!("UNKNOWN ROUTE"),
    }
    // create the response
    unsafe {
        let response = create_response("200 OK", serde_json::to_string(&users).unwrap().as_str());
        match stream.write_all(response.as_bytes()) {
            Ok(()) => println!("Successfully written the response!"),
            Err(e) => panic!("Error while writing the response: {:?}", e),
        }
    }
}

fn main() {
    // read the user data
    let users_string = fs::read_to_string("./src/users.json").unwrap();
    unsafe {
        users = serde_json::from_str(users_string.as_str()).unwrap();
        println!("Users data: {:?}", users);
    }
    // spin up the server
    let listener = match TcpListener::bind("127.0.0.1:7878") {
        Ok(l) => l,
        Err(e) => { panic!("Error binding the socket: {:?}", e) }
    };
    println!("{}", "Now Listening on port 7878...".green().bold());
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => { panic!("Error accepting the connection: {:?}", e) }
        };
        println!("Connection established! - {:?}", stream);
        // stream.set_read_timeout(Some(Duration::from_micros(100))).unwrap();
        thread::spawn(|| {
            handle_connection(stream);
        });
    }
}
