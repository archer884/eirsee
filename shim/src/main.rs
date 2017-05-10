extern crate eirsee;

use eirsee::core::Core;
use eirsee::message::OutgoingMessage;
use eirsee::responder::Responder;

pub struct DummyResponder;

unsafe impl Send for DummyResponder {}

impl Responder for DummyResponder {
    fn private_message(&self, sender: String, content: String) -> Option<OutgoingMessage> {
        println!("PM from {}: {}", sender, content);
        Some(OutgoingMessage::PrivateMessage { recipient: sender, content: String::from("Sorry, I'm AFK.") })
    }

    fn channel_message(&self, sender: String, channel: String, content: String) -> Option<OutgoingMessage> {
        println!("{} in {}: {}", sender, channel, content);
        Some(OutgoingMessage::ChannelMessage { content: format!("ECHO: {}", content) })
    }

    fn user_join(&self, user: String) -> Option<OutgoingMessage> {
        Some(OutgoingMessage::to_channel(
            format!("Hey, {}. Welcome!", user)
        ))
    }

    fn user_part(&self, user: String) -> Option<OutgoingMessage> {
        Some(OutgoingMessage::to_channel(
            format!("Thank God, {} is gone. We can go back to talking shit now.", user)
        ))
    }
}

fn main() {
    use std::borrow::Cow;
    use std::io::{self, BufRead};

    // I've been trying to figure out how to treat an owned string and a string slice the same way for ages.
    // Hooray for progress, right?
    let address = std::env::args().nth(1)
        .map(|s| Cow::from(s))
        .unwrap_or_else(|| Cow::from("localhost:6667"));

    let interface = Core::with_config(Default::default());
    let stdin = io::stdin();
    let handle = interface.connect(&*address, DummyResponder);

    for mut line in stdin.lock().lines().filter_map(|s| s.ok()) {
        match line.pop() {
            Some('#') => handle.send(OutgoingMessage::ChannelMessage { content: line }).unwrap(),
            Some('r') => handle.send(OutgoingMessage::Raw(line)).unwrap(),

            _ => (), // wtf who cares.
        }
    }
}
