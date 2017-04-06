pub trait Responder: Send {
    /// Retrieve a response for the received message.
    ///
    /// Note that not all messages require a response. In fact, I would guess that the vast majority of channel
    /// messages will pass by without any reply from the average bot.
    fn respond(&self, message: &str) -> Option<String> {
        // Here we need to determine the type of the message and then dispatch to other,
        // unimplemented functions defined in the trait.
        unimplemented!()
    }
}

pub struct DummyResponder;

unsafe impl Send for DummyResponder {}

impl Responder for DummyResponder {
    fn respond(&self, _message: &str) -> Option<String> {
        None
        // Some(String::from("Hello!"))    
    }
}
