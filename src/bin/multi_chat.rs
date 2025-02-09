// src/bin/multi_chat.rs
use fltk::{
    app,
    prelude::*,
    window::Window,
    input::Input,
    button::Button,
    text::{TextDisplay, TextBuffer},
    group::Pack,
    frame::Frame,
    enums::{Color, FrameType, Event, Key},
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
    sync::{Arc, Mutex},
    collections::HashMap,
};

#[derive(Debug)]
enum Message {
    UpdateDisplay(String),
    Error(String),
    UserList(String),
}

struct MultiChat {
    app: app::App,
    window: Window,
    input: Input,
    send_button: Button,
    text_display: TextDisplay,
    display_buffer: TextBuffer,
    status_label: Frame,
    users_label: Frame,
    stream: Option<TcpStream>,
    username: String,
}

type ClientMap = Arc<Mutex<HashMap<String, TcpStream>>>;

impl MultiChat {
    fn new(mode: String, username: String) -> Self {
        println!("Creating new MultiChat instance: mode={}, username={}", mode, username);
        let app = app::App::default().with_scheme(app::Scheme::Gtk);
        
        let title = format!("Multi Chat - {} - {}", mode, username);
        let mut window = Window::new(100, 100, 400, 450, &*title);
        
        let mut pack = Pack::new(10, 10, 380, 430, "");
        pack.set_spacing(10);
        
        let mut status_label = Frame::new(0, 0, 380, 30, "Status: Connecting...");
        status_label.set_label_color(Color::Red);
        
        let mut users_label = Frame::new(200, 0, 180, 30, "Users: 0");
        users_label.set_label_color(Color::Blue);
        
        let display_buffer = TextBuffer::default();
        let mut text_display = TextDisplay::new(0, 0, 380, 200, "");
        text_display.set_buffer(display_buffer.clone());
        text_display.set_frame(FrameType::FlatBox);
        text_display.set_color(Color::White);
        
        let mut input = Input::new(0, 0, 300, 30, "");
        let mut send_button = Button::new(300, 0, 80, 30, "Send");
        send_button.set_color(Color::from_rgb(50, 50, 255));
        send_button.set_label_color(Color::White);
        send_button.deactivate();
        
        pack.end();
        window.end();
        window.show();
        
        println!("Window created successfully");
        
        MultiChat {
            app,
            window,
            input,
            send_button,
            text_display,
            display_buffer,
            status_label,
            users_label,
            stream: None,
            username,
        }
    }

    fn handle_initial_connection(stream: &mut TcpStream, sender: app::Sender<Message>) -> std::io::Result<()> {
        println!("Handling initial connection setup");
        
        let mut username_buffer = Vec::new();
        let mut byte_buffer = [0u8; 1];
        
        loop {
            match stream.read(&mut byte_buffer) {
                Ok(1) => {
                    if byte_buffer[0] == b'\n' {
                        break;
                    }
                    username_buffer.push(byte_buffer[0]);
                }
                _ => break,
            }
        }
        
        if let Ok(name) = String::from_utf8(username_buffer) {
            println!("Client username (cleaned): {}", name.trim());
            let msg = format!("User {} joined the chat\n", name.trim());
            sender.send(Message::UpdateDisplay(msg));
            sender.send(Message::UserList(format!("Users: 1")));
        }
        
        Ok(())
    }
    
    fn start_message_receiver(mut stream: TcpStream, sender: app::Sender<Message>) {
        println!("Starting message receiver");
        thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match stream.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        if let Ok(message) = String::from_utf8(buffer[..n].to_vec()) {
                            let trimmed = message.trim();
                            if !trimmed.is_empty() {
                                println!("Receiver got message: {}", trimmed);
                                sender.send(Message::UpdateDisplay(format!("Other: {}\n", trimmed)));
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(e) => {
                        sender.send(Message::Error(format!("Error reading: {}\n", e)));
                        thread::sleep(Duration::from_secs(1));
                    }
                    _ => thread::sleep(Duration::from_millis(50)),
                }
            }
        });
    }
    
    fn connect(&mut self, mode: String, address: String) {
        println!("Starting connection: mode={}, address={}", mode, address);
        let (sender, receiver) = app::channel::<Message>();
        let stream_container = Arc::new(Mutex::new(None));
        let stream_container_clone = Arc::clone(&stream_container);
        let username = self.username.clone();

        thread::spawn(move || {
            let result = match mode.as_str() {
                "server" => {
                    println!("Starting server on {}", address);
                    sender.send(Message::UpdateDisplay("Starting server...\n".to_string()));
                    
                    match TcpListener::bind(&address) {
                        Ok(listener) => {
                            println!("Server bound to address successfully");
                            sender.send(Message::UpdateDisplay("Server started, waiting for clients...\n".to_string()));
                            match listener.accept() {
                                Ok((mut stream, addr)) => {
                                    println!("Client connected from: {}", addr);
                                    
                                    match stream.set_nonblocking(true) {
                                        Ok(_) => {
                                            match stream.try_clone() {
                                                Ok(receiver_stream) => {
                                                    let sender_clone = sender.clone();
                                                    Self::start_message_receiver(receiver_stream, sender_clone);
                                                    
                                                    match Self::handle_initial_connection(&mut stream, sender.clone()) {
                                                        Ok(_) => Ok(stream),
                                                        Err(e) => Err(e)
                                                    }
                                                }
                                                Err(e) => Err(e)
                                            }
                                        }
                                        Err(e) => Err(e)
                                    }
                                }
                                Err(e) => Err(e)
                            }
                        }
                        Err(e) => Err(e)
                    }
                }
                "client" => {
                    println!("Starting client connection to {}", address);
                    sender.send(Message::UpdateDisplay(format!("Connecting to {}...\n", address)));
                    match TcpStream::connect(&address) {
                        Ok(mut stream) => {
                            println!("Client connected successfully");
                            writeln!(stream, "{}", username).unwrap();
                            stream.flush().unwrap();
                            
                            sender.send(Message::UpdateDisplay("Connected successfully\n".to_string()));
                            Ok(stream)
                        }
                        Err(e) => {
                            println!("Error connecting to server: {}", e);
                            Err(e)
                        }
                    }
                }
                _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid mode"))
            };

            match result {
                Ok(stream) => {
                    println!("Setting stream to non-blocking mode");
                    if let Ok(_) = stream.set_nonblocking(true) {
                        let mut lock = stream_container_clone.lock().unwrap();
                        *lock = Some(stream);
                        sender.send(Message::UpdateDisplay("Connected successfully\n".to_string()));
                    }
                }
                Err(e) => {
                    println!("Connection error: {}", e);
                    sender.send(Message::Error(format!("Connection error: {}\n", e)));
                }
            }
        });

        println!("Starting UI update loop");
        while self.window.shown() {
            if let Some(msg) = receiver.recv() {
                println!("Received message: {:?}", msg);
                match msg {
                    Message::UpdateDisplay(text) => {
                        self.display_buffer.append(&text);
                        self.status_label.set_label("Status: Connected");
                        self.status_label.set_label_color(Color::Green);
                        self.send_button.activate();
                        if text.contains("Connected successfully") {
                            println!("Connection successful, breaking connect loop");
                            break;
                        }
                    }
                    Message::Error(text) => {
                        self.display_buffer.append(&text);
                        self.status_label.set_label("Status: Error");
                        self.status_label.set_label_color(Color::Red);
                    }
                    Message::UserList(text) => {
                        self.users_label.set_label(&text);
                    }
                }
                app::flush();
            }
            app::wait();
        }

        let mut lock = stream_container.lock().unwrap();
        if let Some(stream) = lock.take() {
            println!("Taking ownership of stream");
            self.stream = Some(stream);
        }
    }
    
    fn run(&mut self, mode: String, address: String) {
        self.connect(mode, address);
        
        if let Some(stream) = self.stream.as_ref() {
            println!("Setting up message handling");
            let stream_write = stream.try_clone().expect("Failed to clone stream");
            let mut input = self.input.clone();
            let mut display_buffer = self.display_buffer.clone();
            let username = self.username.clone();
            
            let stream_write = Arc::new(Mutex::new(stream_write));
            let stream_write_clone = stream_write.clone();
            
            println!("Setting up send button callback");
            self.send_button.set_callback(move |_| {
                let message = input.value();
                println!("Send button clicked, message: {}", message);
                if !message.is_empty() {
                    if let Ok(mut stream) = stream_write.lock() {
                        let full_message = format!("{}: {}", username, message);
                        println!("Attempting to send: {}", full_message);
                        match writeln!(&mut *stream, "{}", full_message) {
                            Ok(_) => {
                                match stream.flush() {
                                    Ok(_) => {
                                        println!("Message sent successfully");
                                        display_buffer.append(&format!("Me: {}\n", message));
                                        input.set_value("");
                                    }
                                    Err(e) => {
                                        println!("Error flushing stream: {}", e);
                                        display_buffer.append(&format!("Error flushing: {}\n", e));
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Error writing to stream: {}", e);
                                display_buffer.append(&format!("Error sending: {}\n", e));
                            }
                        }
                    } else {
                        println!("Failed to acquire stream lock");
                    }
                }
                app::flush();
            });
            
            let mut input = self.input.clone();
            let mut display_buffer = self.display_buffer.clone();
            let username = self.username.clone();
            
            println!("Setting up Enter key handler");
            input.handle(move |i, ev| {
                if ev == Event::KeyDown && app::event_key() == Key::Enter {
                    let message = i.value();
                    println!("Enter pressed, message: {}", message);
                    if !message.is_empty() {
                        if let Ok(mut stream) = stream_write_clone.lock() {
                            let full_message = format!("{}: {}", username, message);
                            println!("Attempting to send: {}", full_message);
                            match writeln!(&mut *stream, "{}", full_message) {
                                Ok(_) => {
                                    match stream.flush() {
                                        Ok(_) => {
                                            println!("Message sent successfully");
                                            display_buffer.append(&format!("Me: {}\n", message));
                                            i.set_value("");
                                        }
                                        Err(e) => {
                                            println!("Error flushing stream: {}", e);
                                            display_buffer.append(&format!("Error flushing: {}\n", e));
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("Error writing to stream: {}", e);
                                    display_buffer.append(&format!("Error sending: {}\n", e));
                                }
                            }
                        } else {
                            println!("Failed to acquire stream lock");
                        }
                    }
                    app::flush();
                    true
                } else {
                    false
                }
            });
            
            let mut stream_read = stream.try_clone().expect("Failed to clone stream");
            let (sender, receiver) = app::channel::<Message>();
            
            println!("Starting message receiving thread");
            thread::spawn(move || {
                let mut buffer = [0u8; 1024];
                loop {
                    match stream_read.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            println!("Received raw data, bytes: {}", n);
                            if let Ok(message) = String::from_utf8(buffer[..n].to_vec()) {
                                let trimmed = message.trim();
                                if !trimmed.is_empty() {
                                    println!("Processing message: {}", trimmed);
                                    sender.send(Message::UpdateDisplay(format!("Other: {}\n", trimmed)));
                                } else {
                                    println!("Skipping empty message");
                                }
                            } else {
                                println!("Failed to convert received data to UTF-8");
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_secs(1));
                        }
                        _ => {
                            thread::sleep(Duration::from_millis(50));
                        }
                    }
                }
            });

            println!("Starting UI message handling loop");
            let mut display_buffer = self.display_buffer.clone();
            while self.window.shown() {
                if let Some(msg) = receiver.recv() {
                    println!("Received UI message: {:?}", msg);
                    match msg {
                        Message::UpdateDisplay(text) => {
                            println!("Updating display with: {}", text);
                            display_buffer.append(&text);
                        }
                        Message::Error(text) => {
                            println!("Error received: {}", text);
                            display_buffer.append(&text);
                        }
                        Message::UserList(text) => {
                            println!("Updating user list: {}", text);
                            self.users_label.set_label(&text);
                        }
                    }
                }
                app::wait();
            }
        } else {
            println!("No stream available, running empty window loop");
            while self.window.shown() {
                app::wait();
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 4 {
        println!("Usage: cargo run --bin multi_chat <mode> <address> <username>");
        println!("\nExamples:");
        println!("  Server: cargo run --bin multi_chat server 0.0.0.0:8080 ServerUser");
        println!("  Client: cargo run --bin multi_chat client 192.168.0.108:8080 Alice");
        return;
    }
    
    let mode = args[1].clone();
    let address = args[2].clone();
    let username = args[3].clone();
    
    println!("Starting Multi Chat with mode={}, address={}, username={}", mode, address, username);
    let mut chat = MultiChat::new(mode.clone(), username);
    chat.run(mode, address);
}