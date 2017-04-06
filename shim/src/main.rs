extern crate eirsee;

use eirsee::interface::IrcInterface;
use eirsee::message::OutgoingMessage;
use eirsee::responder::Responder;

pub struct DummyResponder;

unsafe impl Send for DummyResponder {}

impl Responder for DummyResponder {
    fn private_message(&self, sender: String, mut content: String) -> Option<OutgoingMessage> {
        content.push_str("...... NOT!");
        Some(OutgoingMessage::PrivateMessage { recipient: sender, content })
    }

    fn channel_message(&self, sender: String, content: String) -> Option<OutgoingMessage> {
        Some(OutgoingMessage::ChannelMessage { content: format!("No, {}, not {}.", sender, content) })
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

    let interface = IrcInterface::new(DummyResponder);

    let stdin = io::stdin();
    let handle = interface.connect(&*address);

    for line in stdin.lock().lines().filter_map(|s| s.ok()) {
        // Is it just me, or does an error here seem pretty fucking unlikely?
        handle.send(OutgoingMessage::Raw(line)).expect("wtf");
    }
}
