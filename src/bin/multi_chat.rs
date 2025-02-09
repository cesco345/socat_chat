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
    
    fn connect(&mut self, mode: String, address: String) {
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
                            sender.send(Message::UpdateDisplay("Server started, waiting for clients...\n".to_string()));
                            match listener.accept() {
                                Ok((mut stream, addr)) => {
                                    println!("Client connected from: {}", addr);
                                    
                                    let mut buffer = [0u8; 1024];
                                    match stream.read(&mut buffer) {
                                        Ok(n) => {
                                            if let Ok(name) = String::from_utf8(buffer[..n-1].to_vec()) {
                                                let msg = format!("User {} joined the chat\n", name);
                                                sender.send(Message::UpdateDisplay(msg));
                                                sender.send(Message::UserList(format!("Users: 1")));
                                            }
                                        }
                                        Err(e) => println!("Error reading username: {}", e),
                                    }
                                    Ok(stream)
                                }
                                Err(e) => Err(e),
                            }
                        }
                        Err(e) => Err(e),
                    }
                }
                "client" => {
                    sender.send(Message::UpdateDisplay(format!("Connecting to {}...\n", address)));
                    match TcpStream::connect(&address) {
                        Ok(mut stream) => {
                            writeln!(stream, "{}", username).unwrap();
                            stream.flush().unwrap();
                            
                            sender.send(Message::UpdateDisplay("Connected successfully\n".to_string()));
                            Ok(stream)
                        }
                        Err(e) => Err(e),
                    }
                }
                _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid mode")),
            };

            match result {
                Ok(stream) => {
                    if let Ok(_) = stream.set_nonblocking(true) {
                        let mut lock = stream_container_clone.lock().unwrap();
                        *lock = Some(stream);
                        sender.send(Message::UpdateDisplay("Connected successfully\n".to_string()));
                    }
                }
                Err(e) => {
                    sender.send(Message::Error(format!("Connection error: {}\n", e)));
                }
            }
        });

        while self.window.shown() {
            if let Some(msg) = receiver.recv() {
                match msg {
                    Message::UpdateDisplay(text) => {
                        self.display_buffer.append(&text);
                        self.status_label.set_label("Status: Connected");
                        self.status_label.set_label_color(Color::Green);
                        self.send_button.activate();
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
            self.stream = Some(stream);
        }
    }
    
    fn run(&mut self, mode: String, address: String) {
        self.connect(mode, address);
        
        if let Some(stream) = self.stream.as_ref() {
            let stream_write = stream.try_clone().expect("Failed to clone stream");
            let mut input = self.input.clone();
            let mut display_buffer = self.display_buffer.clone();
            let username = self.username.clone();
            
            let stream_write = Arc::new(Mutex::new(stream_write));
            let stream_write_clone = stream_write.clone();
            
            // Simplified send button callback to match network_chat.rs
            self.send_button.set_callback(move |_| {
                let message = input.value();
                if !message.is_empty() {
                    if let Ok(mut stream) = stream_write.lock() {
                        match writeln!(&mut *stream, "{}: {}", username, message) {
                            Ok(_) => {
                                if let Ok(_) = stream.flush() {
                                    display_buffer.append(&format!("Me: {}\n", message));
                                    input.set_value("");
                                }
                            }
                            Err(e) => {
                                display_buffer.append(&format!("Error sending: {}\n", e));
                            }
                        }
                    }
                }
            });
            
            let mut input = self.input.clone();
            let mut display_buffer = self.display_buffer.clone();
            let username = self.username.clone();
            
            // Simplified Enter key handler to match network_chat.rs
            input.handle(move |i, ev| {
                if ev == Event::KeyDown && app::event_key() == Key::Enter {
                    let message = i.value();
                    if !message.is_empty() {
                        if let Ok(mut stream) = stream_write_clone.lock() {
                            match writeln!(&mut *stream, "{}: {}", username, message) {
                                Ok(_) => {
                                    if let Ok(_) = stream.flush() {
                                        display_buffer.append(&format!("Me: {}\n", message));
                                        i.set_value("");
                                    }
                                }
                                Err(e) => {
                                    display_buffer.append(&format!("Error sending: {}\n", e));
                                }
                            }
                        }
                    }
                    true
                } else {
                    false
                }
            });
            
            let mut stream_read = stream.try_clone().expect("Failed to clone stream");
            let (sender, receiver) = app::channel::<Message>();
            
            thread::spawn(move || {
                let mut buffer = [0u8; 1024];
                loop {
                    match stream_read.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            if let Ok(message) = String::from_utf8(buffer[..n].to_vec()) {
                                if !message.trim().is_empty() {
                                    sender.send(Message::UpdateDisplay(format!("{}", message)));
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

            let mut display_buffer = self.display_buffer.clone();
            while self.window.shown() {
                if let Some(msg) = receiver.recv() {
                    match msg {
                        Message::UpdateDisplay(text) => {
                            display_buffer.append(&text);
                        }
                        Message::Error(text) => {
                            display_buffer.append(&text);
                        }
                        Message::UserList(text) => {
                            self.users_label.set_label(&text);
                        }
                    }
                }
                app::wait();
            }
        } else {
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
    
    let mut chat = MultiChat::new(mode.clone(), username);
    chat.run(mode, address);
}