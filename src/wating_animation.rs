use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::io::Write;
use std::sync::mpsc::{self, Sender, Receiver};

#[derive(PartialEq)]
enum ThreadMessage {
    End,
}

pub struct ThreadConnection {
    handle: JoinHandle<()>,
    sender: Sender<ThreadMessage>,
}

impl ThreadConnection {
    fn new(handle: JoinHandle<()>, sender: Sender<ThreadMessage>) -> ThreadConnection {
        ThreadConnection {
            handle,
            sender,
        }
    }

    pub fn end(self) {
        let _ = self.sender.send(ThreadMessage::End);
        let _ = self.handle.join();
    }
}

pub fn spawn_animation_thread(message: &str) -> ThreadConnection {
    let (tx, rx) = mpsc::channel();
    
    let message = message.to_string();
    let handle = thread::spawn(move || animation_thread(&message, rx));

    ThreadConnection::new(handle, tx)
}

fn animation_thread(message: &str, receiver: Receiver<ThreadMessage>) {
    let message = message.to_string();
    println!("\n");
    loop {
        for i in 0..4 {
            let needs_to_close = match receiver.try_recv() {
                Ok(m) => m == ThreadMessage::End,
                Err(_) => false
            };

            if needs_to_close == true {
                return;
            }

            let points_str = (0..4).map(|j| {
                    if j <= i {
                        '.'
                    } else {
                        ' '
                    }
                }).collect::<String>();

            print!("\r");
            print!("{}{}", message, points_str);
            let _ = std::io::stdout().flush();
            
            thread::sleep(Duration::from_millis(800));
        }
    }
}