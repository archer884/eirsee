use config::Config;
use std::fmt;
use std::str;
use regex::Regex;

lazy_static! {
    static ref PRIVMSG: Regex = Regex::new(r#":(\S+)!~.+ PRIVMSG (\S+) :(.*)"#).unwrap();
    static ref CHANMSG: Regex = Regex::new(r#":(\S+)!~.+ PRIVMSG (#\S+) :(.*)"#).unwrap();
    static ref JOIN: Regex = Regex::new(r#":(\S+)!~.+ JOIN :(#\S+)"#).unwrap();
    static ref PART: Regex = Regex::new(r#":(\S+)!~.+ PART (#\S+)"#).unwrap();
}

pub enum IncomingMessage {
    Join(String),
    Part(String),
    Ping(String),
    // Channel messages do not include the channel they were received from because we only join one damn channel.
    ChannelMessage { sender: String, channel: String, content: String },
    PrivateMessage { sender: String, content: String },
}

// Why can I still not use TryFrom for this?!
impl str::FromStr for IncomingMessage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("PING") {
            return Ok(IncomingMessage::Ping(String::from(&s[s.find(':').unwrap_or(0)..])));
        }

        if let Some(captures) = CHANMSG.captures(s) {
            return Ok(IncomingMessage::ChannelMessage {
                sender: captures.get(1).unwrap().as_str().to_string(),
                channel: captures.get(2).unwrap().as_str().to_string(),
                content: captures.get(3).unwrap().as_str().to_string(),
            })
        }

        if let Some(captures) = PRIVMSG.captures(s) {
            return Ok(IncomingMessage::PrivateMessage {
                sender: captures.get(1).unwrap().as_str().to_string(),
                content: captures.get(3).unwrap().as_str().to_string(),
            })
        }

        if let Some(captures) = JOIN.captures(s) {
            return Ok(IncomingMessage::Join(captures.get(1).unwrap().as_str().to_string()))
        }

        if let Some(captures) = PART.captures(s) {
            return Ok(IncomingMessage::Part(captures.get(1).unwrap().as_str().to_string()))
        }

        Err(format!("Unrecognized: {}", s))
    }
}

// I don't know if there is any programmatic difference between a "channel message" and a "private message"
// for the purposes of outgoing messages. There *kind of* is for incoming messages (you can target either the
// channel or the person who sent you the message, right? I dunno...)
//
// Also, I'm not certain that an outgoing message should not just reference a user via an Rc or something, but
// I am not clear on how to pass that across thread boundaries. That's *probably* something I can revisit later.
pub enum OutgoingMessage {
    Nick(Option<String>),
    User,
    Join,
    Topic(String),
    Pong(String),
    ChannelMessage { content: String },
    PrivateMessage { recipient: String, content: String },
    Raw(String),
}

impl OutgoingMessage {
    pub fn to_channel(content: String) -> OutgoingMessage {
        OutgoingMessage::ChannelMessage { content }
    }

    pub fn to_private(recipient: String, content: String) -> OutgoingMessage {
        OutgoingMessage::PrivateMessage { recipient, content }
    }
}

pub struct MessageFormatter<'a> {
    pub config: &'a Config,
    pub message: &'a OutgoingMessage,
}

impl<'a> fmt::Display for MessageFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::OutgoingMessage::*;

        match *self.message {
            Nick(Some(ref nick)) => write!(f, "NICK {}", nick),
            Nick(None) => write!(f, "NICK {}", self.config.user),
            User => write!(f, "USER {} 0 * :{}", self.config.user, self.config.name),
            Join => write!(f, ":{} join :#{}", self.config.user, self.config.channel),
            Topic(ref topic) => write!(f, ":{} TOPIC #{} :{}", self.config.user, self.config.channel, topic),

            // Pong formatter looks different from the rest because the message we copy from the incoming ping
            // already contains the : expected in the pong. Rather than do the extra work to remove that, we're
            // just shunting it back out the door as part of the response.
            Pong(ref message) => write!(f, "PONG {}", message),

            PrivateMessage { ref recipient, ref content } => write!(f, ":{} PRIVMSG {} :{}", self.config.user, recipient, content),
            ChannelMessage { ref content } => write!(f, ":{} PRIVMSG #{} :{}", self.config.user, self.config.channel, content),
            Raw(ref message) => write!(f, "{}", message),
        }
    }
}
