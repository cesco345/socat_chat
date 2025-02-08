// src/bin/simple_chat.rs
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
    fs::OpenOptions,
    io::{Read, Write},
    thread,
    time::Duration,
};

struct SimpleChatApp {
    window: Window,
    input: Input,
    send_button: Button,
    text_display: TextDisplay,
    display_buffer: TextBuffer,
}

impl SimpleChatApp {
    fn new(name: &str) -> Self {
        let _app = app::App::default();
        let mut window = Window::new(100, 100, 400, 300, format!("Chat - {}", name).as_str());
        
        let mut pack = Pack::new(10, 10, 380, 280, "");
        pack.set_spacing(10);
        
        let display_buffer = TextBuffer::default();
        let mut text_display = TextDisplay::new(0, 0, 380, 200, "");
        text_display.set_buffer(display_buffer.clone());
        
        let input = Input::new(0, 0, 300, 30, "");
        let send_button = Button::new(310, 0, 70, 30, "Send");
        
        pack.end();
        window.end();
        window.show();
        
        SimpleChatApp {
            window,
            input,
            send_button,
            text_display,
            display_buffer,
        }
    }
    
    fn run(&mut self, read_pipe: &str, write_pipe: &str) {
        println!("Opening pipes...");
        println!("Read from: {}", read_pipe);
        println!("Write to: {}", write_pipe);

        // Set up write pipe
        let write_pipe = write_pipe.to_string();
        let mut input = self.input.clone();
        let mut display_buffer = self.display_buffer.clone();
        
        self.send_button.set_callback(move |_| {
            let message = input.value();
            if !message.is_empty() {
                if let Ok(mut file) = OpenOptions::new().write(true).open(&write_pipe) {
                    if let Ok(_) = writeln!(file, "{}", message) {
                        display_buffer.append(&format!("Me: {}\n", message));
                        input.set_value("");
                    }
                }
            }
        });
        
        // Set up read pipe
        let read_pipe = read_pipe.to_string();
        let mut display_buffer = self.display_buffer.clone();
        
        thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                if let Ok(mut file) = OpenOptions::new().read(true).open(&read_pipe) {
                    if let Ok(n) = file.read(&mut buffer) {
                        if n > 0 {
                            if let Ok(message) = String::from_utf8(buffer[..n].to_vec()) {
                                display_buffer.append(&format!("Other: {}", message));
                            }
                        }
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
        });
        
        while self.window.shown() {
            app::wait();
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        println!("Usage: cargo run --bin simple_chat <READ_PIPE> <WRITE_PIPE>");
        println!("Example for first instance:");
        println!("  cargo run --bin simple_chat /tmp/pipe1 /tmp/pipe2");
        println!("Example for second instance:");
        println!("  cargo run --bin simple_chat /tmp/pipe2 /tmp/pipe1");
        println!("\nMake sure to create pipes first with:");
        println!("  mkfifo /tmp/pipe1 /tmp/pipe2");
        return;
    }
    
    let mut app = SimpleChatApp::new(&args[1]);
    app.run(&args[1], &args[2]);
}
