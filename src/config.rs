use message::*;
use std::fmt;

pub struct Config {
    pub user: String,
    pub name: String,
    pub channel: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            user: String::from("scrbot"),
            name: String::from("scrbot"),
            channel: String::from("hello"),
        }
    }
}

impl Config {
    pub fn format<'a>(&'a self, message: &'a OutgoingMessage) -> MessageFormatter<'a> {
        MessageFormatter { config: self, message }
    }
}
