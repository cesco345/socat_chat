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
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
    sync::{Arc, Mutex},
};

struct NetworkChat {
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
        let _app = app::App::default().with_scheme(app::Scheme::Gtk);
        
        // Create the window title string first
        let title = format!("Network Chat - {}", mode);
        let mut window = Window::new(100, 100, 400, 350, title.as_str());
        
        let mut pack = Pack::new(10, 10, 380, 330, "");
        pack.set_spacing(10);
        
        // Status label at the top
        let mut status_label = Frame::new(0, 0, 380, 30, "Status: Connecting...");
        status_label.set_label_color(fltk::enums::Color::Red);
        
        // Message display area
        let display_buffer = TextBuffer::default();
        let mut text_display = TextDisplay::new(0, 0, 380, 200, "");
        text_display.set_buffer(display_buffer.clone());
        
        // Input area
        let input = Input::new(0, 0, 300, 30, "");
        let mut send_button = Button::new(310, 0, 70, 30, "Send");
        send_button.deactivate(); // Disabled until connected
        
        pack.end();
        window.end();
        window.show();
        
        NetworkChat {
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
        let mut status_label = self.status_label.clone();
        let mut display_buffer = self.display_buffer.clone();
        let mut send_button = self.send_button.clone();
        let stream_container = Arc::new(Mutex::new(None));
        let stream_container_clone = Arc::clone(&stream_container);

        thread::spawn(move || {
            let result = match mode.as_str() {
                "server" => {
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
                }
                _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid mode")),
            };

            match result {
                Ok(stream) => {
                    if let Ok(_) = stream.set_nonblocking(true) {
                        let mut lock = stream_container_clone.lock().unwrap();
                        *lock = Some(stream);
                        status_label.set_label("Status: Connected");
                        status_label.set_label_color(fltk::enums::Color::Green);
                        send_button.activate();
                    }
                }
                Err(e) => {
                    status_label.set_label(&format!("Status: Connection failed - {}", e));
                    status_label.set_label_color(fltk::enums::Color::Red);
                    display_buffer.append(&format!("Connection error: {}\n", e));
                }
            }
        });

        // Wait a bit for the connection
        thread::sleep(Duration::from_millis(100));
        let mut lock = stream_container.lock().unwrap();
        if let Some(stream) = lock.take() {
            self.stream = Some(stream);
        }
    }
    
    fn run(&mut self, mode: String, address: String) {
        self.connect(mode, address);
        
        // Set up callbacks only if we have a stream
        if let Some(stream) = self.stream.as_ref() {
            let mut stream_write = stream.try_clone().expect("Failed to clone stream");
            let mut input = self.input.clone();
            let mut display_buffer = self.display_buffer.clone();
            
            self.send_button.set_callback(move |_| {
                let message = input.value();
                if !message.is_empty() {
                    if let Ok(_) = writeln!(stream_write, "{}", message) {
                        display_buffer.append(&format!("Me: {}\n", message));
                        input.set_value("");
                    }
                }
            });
            
            let mut stream_read = stream.try_clone().expect("Failed to clone stream");
            let mut display_buffer = self.display_buffer.clone();
            
            thread::spawn(move || {
                let mut buffer = [0u8; 1024];
                loop {
                    match stream_read.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            if let Ok(message) = String::from_utf8(buffer[..n].to_vec()) {
                                display_buffer.append(&format!("Other: {}", message));
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
        println!("  Client: cargo run --bin network_chat client 192.168.1.100:8080");
        return;
    }
    
    let mode = args[1].clone();
    let address = args[2].clone();
    
    let mut chat = NetworkChat::new(mode.clone(), address.clone());
    chat.run(mode, address);
}