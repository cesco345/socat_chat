// src/bin/network_chat.rs
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
};

struct NetworkChat {
    app: app::App,  // Keep app instance alive
    window: Window,
    input: Input,
    send_button: Button,
    text_display: TextDisplay,
    display_buffer: TextBuffer,
    status_label: Frame,
    stream: Option<TcpStream>,
}

impl NetworkChat {
    fn new(mode: String, _address: String) -> Self {
        let app = app::App::default().with_scheme(app::Scheme::Gtk);
        
        // Create the window title string first
        let title = format!("Network Chat - {}", mode);
        let mut window = Window::new(100, 100, 400, 350, &*title);
        
        let mut pack = Pack::new(10, 10, 380, 330, "");
        pack.set_spacing(10);
        
        // Status label at the top
        let mut status_label = Frame::new(0, 0, 380, 30, "Status: Connecting...");
        status_label.set_label_color(Color::Red);
        
        // Message display area
        let display_buffer = TextBuffer::default();
        let mut text_display = TextDisplay::new(0, 0, 380, 200, "");
        text_display.set_buffer(display_buffer.clone());
        text_display.set_frame(FrameType::FlatBox);
        text_display.set_color(Color::White);
        
        // Input area
        let input = Input::new(0, 0, 300, 30, "");
        let mut send_button = Button::new(310, 0, 70, 30, "Send");
        send_button.deactivate(); // Disabled until connected
        
        pack.end();
        window.end();
        window.show();
        
        NetworkChat {
            app,
            window,
            input,
            send_button,
            text_display,
            display_buffer,
            status_label,
            stream: None,
        }
    }
    
    fn connect(&mut self, mode: String, address: String) {
        let (sender, receiver) = app::channel::<String>();
        let stream_container = Arc::new(Mutex::new(None));
        let stream_container_clone = Arc::clone(&stream_container);
        let address_clone = address.clone();

        thread::spawn(move || {
            let result = match mode.as_str() {
                "server" => {
                    println!("Starting server on {}", address_clone);
                    sender.send("WAIT".to_string());
                    match TcpListener::bind(&address_clone) {
                        Ok(listener) => {
                            sender.send("SERVER_STARTED".to_string());
                            match listener.accept() {
                                Ok((stream, addr)) => {
                                    println!("Client connected from: {}", addr);
                                    sender.send(format!("CLIENT_CONNECTED:{}", addr));
                                    Ok(stream)
                                }
                                Err(e) => {
                                    sender.send(format!("ERROR:{}", e));
                                    Err(e)
                                }
                            }
                        }
                        Err(e) => {
                            sender.send(format!("ERROR:{}", e));
                            Err(e)
                        }
                    }
                }
                "client" => {
                    sender.send("CONNECTING".to_string());
                    match TcpStream::connect(&address_clone) {
                        Ok(stream) => {
                            println!("Client connected successfully");
                            sender.send("CONNECTED".to_string());
                            Ok(stream)
                        }
                        Err(e) => {
                            sender.send(format!("ERROR:{}", e));
                            Err(e)
                        }
                    }
                }
                _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid mode")),
            };

            match result {
                Ok(stream) => {
                    if let Ok(_) = stream.set_nonblocking(true) {
                        let mut lock = stream_container_clone.lock().unwrap();
                        *lock = Some(stream);
                        sender.send("SUCCESS".to_string());
                    }
                }
                Err(e) => {
                    sender.send(format!("ERROR:{}", e));
                }
            }
        });

        // Handle UI updates in the main thread
        while self.window.shown() {
            if let Some(msg) = receiver.recv() {
                match msg.as_str() {
                    "WAIT" => {
                        self.status_label.set_label("Status: Waiting for client...");
                        self.status_label.set_label_color(Color::Yellow);
                    }
                    "SERVER_STARTED" => {
                        self.display_buffer.append("Server started, waiting for connection...\n");
                    }
                    "CONNECTING" => {
                        self.status_label.set_label("Status: Connecting to server...");
                        self.status_label.set_label_color(Color::Yellow);
                        self.display_buffer.append(&format!("Connecting to {}...\n", address));
                    }
                    "CONNECTED" | "SUCCESS" => {
                        self.status_label.set_label("Status: Connected");
                        self.status_label.set_label_color(Color::Green);
                        self.send_button.activate();
                        break;
                    }
                    msg if msg.starts_with("CLIENT_CONNECTED:") => {
                        let addr = msg.split(':').nth(1).unwrap_or("unknown");
                        self.display_buffer.append(&format!("Client connected from: {}\n", addr));
                        self.status_label.set_label("Status: Connected");
                        self.status_label.set_label_color(Color::Green);
                        self.send_button.activate();
                        break;
                    }
                    msg if msg.starts_with("ERROR:") => {
                        let error = msg.split(':').nth(1).unwrap_or("Unknown error");
                        self.status_label.set_label(&format!("Status: Connection failed - {}", error));
                        self.status_label.set_label_color(Color::Red);
                        self.display_buffer.append(&format!("Connection error: {}\n", error));
                        break;
                    }
                    _ => {}
                }
                app::flush();
            }
            app::wait_for(0.1);
        }

        let mut lock = stream_container.lock().unwrap();
        if let Some(stream) = lock.take() {
            self.stream = Some(stream);
        }
    }
    
    fn run(&mut self, mode: String, address: String) {
        self.connect(mode, address);
        
        // Set up callbacks only if we have a stream
        if let Some(stream) = self.stream.as_ref() {
            let stream_write = stream.try_clone().expect("Failed to clone stream");
            let mut input = self.input.clone();
            let mut display_buffer = self.display_buffer.clone();
            
            let stream_write = Arc::new(Mutex::new(stream_write));
            let stream_write_clone = stream_write.clone();
            
            // Set up send button callback
            self.send_button.set_callback(move |_| {
                let message = input.value();
                if !message.is_empty() {
                    if let Ok(mut stream) = stream_write.lock() {
                        match writeln!(&mut *stream, "{}", message) {
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
            
            // Set up Enter key handler
            let mut input = self.input.clone();
            let mut display_buffer = self.display_buffer.clone();
            
            input.handle(move |i, ev| {
                if ev == Event::KeyDown && app::event_key() == Key::Enter {
                    let message = i.value();
                    if !message.is_empty() {
                        if let Ok(mut stream) = stream_write_clone.lock() {
                            match writeln!(&mut *stream, "{}", message) {
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
            
            // Set up message receiving thread
            let mut stream_read = stream.try_clone().expect("Failed to clone stream");
            let mut display_buffer = self.display_buffer.clone();
            
            thread::spawn(move || {
                let mut buffer = [0u8; 1024];
                loop {
                    match stream_read.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            if let Ok(message) = String::from_utf8(buffer[..n].to_vec()) {
                                display_buffer.append(&format!("Other: {}", message));
                                app::awake();
                                app::flush();
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(50));
                        }
                        Err(e) => {
                            display_buffer.append(&format!("Error reading: {}\n", e));
                            app::awake();
                            app::flush();
                            thread::sleep(Duration::from_secs(1));
                        }
                        _ => thread::sleep(Duration::from_millis(50)),
                    }
                }
            });
        }
        
        while self.window.shown() {
            app::wait();
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        println!("Usage: cargo run --bin network_chat <mode> <address>");
        println!("\nExamples:");
        println!("  Server: cargo run --bin network_chat server 0.0.0.0:8080");
        println!("  Client: cargo run --bin network_chat client 192.168.0.108:8080");
        return;
    }
    
    let mode = args[1].clone();
    let address = args[2].clone();
    
    let mut chat = NetworkChat::new(mode.clone(), address.clone());
    chat.run(mode, address);
}