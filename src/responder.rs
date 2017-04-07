use message::*;

pub trait Responder: Send {
    /// Retrieve a response for the received message.
    ///
    /// Note that not all messages require a response. In fact, I would guess that the vast majority of channel
    /// messages will pass by without any reply from the average bot.
    fn respond(&self, message: String) -> Option<OutgoingMessage> {
        // Here we need to determine the type of the message and then dispatch to other,
        // unimplemented functions defined in the trait.
        match message.parse() {
            // First off, just fire back pongs immediately because I can't be arsed to even consider configuring this.
            Ok(IncomingMessage::Ping(message)) => Some(OutgoingMessage::Pong(message)),
            Ok(IncomingMessage::ChannelMessage { sender, content }) => self.channel_message(sender, content),
            Ok(IncomingMessage::PrivateMessage { sender, content }) => self.private_message(sender, content),

            _ => None,
        }
    }

    fn channel_message(&self, sender: String, content: String) -> Option<OutgoingMessage>;
    fn private_message(&self, sender: String, content: String) -> Option<OutgoingMessage>;
}
