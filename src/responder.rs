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
            Ok(IncomingMessage::Join(user)) => self.user_join(user),
            Ok(IncomingMessage::Part(user)) => self.user_part(user),
            Ok(IncomingMessage::ChannelMessage { sender, channel, content }) => self.channel_message(sender, channel, content),
            Ok(IncomingMessage::PrivateMessage { sender, content }) => self.private_message(sender, content),

            // This should almost certainly be returning some kind of an error case, because it's not kosher
            // to be spitting out log messages and swallowing weirdness this deep in the stack.
            _ => {
                println!("UNKNOWN: {}", message);
                None
            },
        }
    }

    fn channel_message(&self, sender: String, channel: String, content: String) -> Option<OutgoingMessage>;
    fn private_message(&self, sender: String, content: String) -> Option<OutgoingMessage>;
    fn user_join(&self, user: String) -> Option<OutgoingMessage>;
    fn user_part(&self, user: String) -> Option<OutgoingMessage>;
}
