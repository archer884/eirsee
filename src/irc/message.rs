pub struct Message {
    from: Sender,
    kind: MessageKind,
    content: String,
}

pub enum Sender {
    Server,
    User(User),
}

pub struct User;

enum MessageKind {
    ChannelMessage,
    Ping,
    PrivateMessage,
}
