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
<<<<<<< HEAD
=======
    enums::{Color, FrameType, Event, Key},
>>>>>>> 70713a8 (fix server-client connection issue)
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
    sync::{Arc, Mutex},
};

struct NetworkChat {
<<<<<<< HEAD
=======
    app: app::App,  // Keep app instance alive
>>>>>>> 70713a8 (fix server-client connection issue)
    window: Window,
    input: Input,
    send_button: Button,
    text_display: TextDisplay,
    display_buffer: TextBuffer,
    status_label: Frame,
    stream: Option<TcpStream>,
}

impl NetworkChat {
    fn new(mode: String, address: String) -> Self {
<<<<<<< HEAD
        let _app = app::App::default().with_scheme(app::Scheme::Gtk);
        
        // Create the window title string first
        let title = format!("Network Chat - {}", mode);
        let mut window = Window::new(100, 100, 400, 350, title.as_str());
=======
        let app = app::App::default().with_scheme(app::Scheme::Gtk);
        
        // Create the window title string first
        let title = format!("Network Chat - {}", mode);
        let mut window = Window::new(100, 100, 400, 350, &*title);
>>>>>>> 70713a8 (fix server-client connection issue)
        
        let mut pack = Pack::new(10, 10, 380, 330, "");
        pack.set_spacing(10);
        
        // Status label at the top
        let mut status_label = Frame::new(0, 0, 380, 30, "Status: Connecting...");
<<<<<<< HEAD
        status_label.set_label_color(fltk::enums::Color::Red);
=======
        status_label.set_label_color(Color::Red);
>>>>>>> 70713a8 (fix server-client connection issue)
        
        // Message display area
        let display_buffer = TextBuffer::default();
        let mut text_display = TextDisplay::new(0, 0, 380, 200, "");
        text_display.set_buffer(display_buffer.clone());
<<<<<<< HEAD
=======
        text_display.set_frame(FrameType::FlatBox);
        text_display.set_color(Color::White);
>>>>>>> 70713a8 (fix server-client connection issue)
        
        // Input area
        let input = Input::new(0, 0, 300, 30, "");
        let mut send_button = Button::new(310, 0, 70, 30, "Send");
        send_button.deactivate(); // Disabled until connected
        
        pack.end();
        window.end();
        window.show();
        
        NetworkChat {
<<<<<<< HEAD
=======
            app,
>>>>>>> 70713a8 (fix server-client connection issue)
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
<<<<<<< HEAD
        let mut status_label = self.status_label.clone();
        let mut display_buffer = self.display_buffer.clone();
        let mut send_button = self.send_button.clone();
        let stream_container = Arc::new(Mutex::new(None));
        let stream_container_clone = Arc::clone(&stream_container);
=======
        let (sender, receiver) = app::channel::<String>();
        let stream_container = Arc::new(Mutex::new(None));
        let stream_container_clone = Arc::clone(&stream_container);
        let address_clone = address.clone();  // Clone for thread
>>>>>>> 70713a8 (fix server-client connection issue)

        thread::spawn(move || {
            let result = match mode.as_str() {
                "server" => {
<<<<<<< HEAD
                    status_label.set_label("Status: Waiting for client...");
                    status_label.set_label_color(fltk::enums::Color::Yellow);
                    match TcpListener::bind(&address) {
                        Ok(listener) => {
                            display_buffer.append("Server started, waiting for connection...\n");
                            match listener.accept() {
                                Ok((stream, addr)) => {
                                    display_buffer.append(&format!("Client connected from: {}\n", addr));
                                    Ok(stream)
                                }
                                Err(e) => Err(e),
                            }
                        }
                        Err(e) => Err(e),
                    }
                }
                "client" => {
                    status_label.set_label("Status: Connecting to server...");
                    status_label.set_label_color(fltk::enums::Color::Yellow);
                    display_buffer.append(&format!("Connecting to {}...\n", address));
                    TcpStream::connect(&address)
=======
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
>>>>>>> 70713a8 (fix server-client connection issue)
                }
                _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid mode")),
            };

            match result {
                Ok(stream) => {
                    if let Ok(_) = stream.set_nonblocking(true) {
                        let mut lock = stream_container_clone.lock().unwrap();
                        *lock = Some(stream);
<<<<<<< HEAD
                        status_label.set_label("Status: Connected");
                        status_label.set_label_color(fltk::enums::Color::Green);
                        send_button.activate();
                    }
                }
                Err(e) => {
                    status_label.set_label(&format!("Status: Connection failed - {}", e));
                    status_label.set_label_color(fltk::enums::Color::Red);
                    display_buffer.append(&format!("Connection error: {}\n", e));
=======
                        sender.send("SUCCESS".to_string());
                    }
                }
                Err(e) => {
                    sender.send(format!("ERROR:{}", e));
>>>>>>> 70713a8 (fix server-client connection issue)
                }
            }
        });

<<<<<<< HEAD
        // Wait a bit for the connection
        thread::sleep(Duration::from_millis(100));
=======
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

>>>>>>> 70713a8 (fix server-client connection issue)
        let mut lock = stream_container.lock().unwrap();
        if let Some(stream) = lock.take() {
            self.stream = Some(stream);
        }
    }
    
    fn run(&mut self, mode: String, address: String) {
        self.connect(mode, address);
        
        // Set up callbacks only if we have a stream
        if let Some(stream) = self.stream.as_ref() {
<<<<<<< HEAD
            let mut stream_write = stream.try_clone().expect("Failed to clone stream");
            let mut input = self.input.clone();
            let mut display_buffer = self.display_buffer.clone();
            
            self.send_button.set_callback(move |_| {
                let message = input.value();
                if !message.is_empty() {
                    if let Ok(_) = writeln!(stream_write, "{}", message) {
                        display_buffer.append(&format!("Me: {}\n", message));
                        input.set_value("");
=======
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
                                // Explicitly flush the stream after writing
                                if let Ok(_) = stream.flush() {
                                    display_buffer.append(&format!("Me: {}\n", message));
                                    input.set_value("");
                                    app::flush();
                                }
                            }
                            Err(e) => {
                                display_buffer.append(&format!("Error sending: {}\n", e));
                                app::flush();
                            }
                        }
>>>>>>> 70713a8 (fix server-client connection issue)
                    }
                }
            });
            
<<<<<<< HEAD
=======
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
                                        app::flush();
                                    }
                                }
                                Err(e) => {
                                    display_buffer.append(&format!("Error sending: {}\n", e));
                                    app::flush();
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
>>>>>>> 70713a8 (fix server-client connection issue)
            let mut stream_read = stream.try_clone().expect("Failed to clone stream");
            let mut display_buffer = self.display_buffer.clone();
            
            thread::spawn(move || {
                let mut buffer = [0u8; 1024];
                loop {
                    match stream_read.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            if let Ok(message) = String::from_utf8(buffer[..n].to_vec()) {
                                display_buffer.append(&format!("Other: {}", message));
<<<<<<< HEAD
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(100));
                        }
                        Err(e) => {
                            display_buffer.append(&format!("Error reading: {}\n", e));
                            thread::sleep(Duration::from_secs(1));
                        }
                        _ => thread::sleep(Duration::from_millis(100)),
=======
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
>>>>>>> 70713a8 (fix server-client connection issue)
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
<<<<<<< HEAD
        println!("  Server: cargo run --bin network_chat server 0.0.0.0:8080");
        println!("  Client: cargo run --bin network_chat client 192.168.1.100:8080");
=======
        println!("  Server: cargo run --bin network_chat server 192.168.0.108:8080");
        println!("  Client: cargo run --bin network_chat client 192.168.0.108:8080");
>>>>>>> 70713a8 (fix server-client connection issue)
        return;
    }
    
    let mode = args[1].clone();
    let address = args[2].clone();
    
    let mut chat = NetworkChat::new(mode.clone(), address.clone());
    chat.run(mode, address);
}