// src/bin/network_chat.rs
use fltk::{
    app,
    prelude::*,
    window::Window,
    input::Input,
    button::Button,
    text::{TextDisplay, TextBuffer},
    group::Pack,
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

struct NetworkChat {
    window: Window,
    input: Input,
    send_button: Button,
    text_display: TextDisplay,
    display_buffer: TextBuffer,
    stream: TcpStream,
}

impl NetworkChat {
    fn new(mode: &str, address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize FLTK application first
        let app = app::App::default().with_scheme(app::Scheme::Gtk);
        
        // Create GUI components
        let mut window = Window::new(100, 100, 400, 300, format!("Network Chat - {}", mode).as_str());
        
        let mut pack = Pack::new(10, 10, 380, 280, "");
        pack.set_spacing(10);
        
        // Message display area
        let display_buffer = TextBuffer::default();
        let mut text_display = TextDisplay::new(0, 0, 380, 200, "");
        text_display.set_buffer(display_buffer.clone());
        
        // Input area
        let input = Input::new(0, 0, 300, 30, "");
        let send_button = Button::new(310, 0, 70, 30, "Send");
        
        pack.end();
        window.end();
        
        // Show the window before establishing connection
        window.show();
        app::wait();
        
        println!("Establishing connection...");
        
        // Connect or create server based on mode
        let stream = match mode {
            "server" => {
                println!("Starting server on {}", address);
                let listener = TcpListener::bind(address)?;
                println!("Waiting for client to connect...");
                let (stream, addr) = listener.accept()?;
                println!("Client connected from: {}", addr);
                stream
            }
            "client" => {
                println!("Connecting to server at {}", address);
                TcpStream::connect(address)?
            }
            _ => return Err("Invalid mode - use 'server' or 'client'".into()),
        };
        
        // Configure stream
        stream.set_nonblocking(true)?;
        
        Ok(NetworkChat {
            window,
            input,
            send_button,
            text_display,
            display_buffer,
            stream,
        })
    }
    
    fn run(&mut self) {
        // Set up send button callback
        let mut stream_write = self.stream.try_clone().expect("Failed to clone stream");
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
        
        // Set up message receiving thread
        let mut stream_read = self.stream.try_clone().expect("Failed to clone stream");
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
                        // No data available right now, wait a bit
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
        
        // Main event loop
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
    
    let mode = &args[1];
    let address = &args[2];
    
    match NetworkChat::new(mode, address) {
        Ok(mut chat) => chat.run(),
        Err(e) => eprintln!("Error: {}", e),
    }
}