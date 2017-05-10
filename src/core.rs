use config::Config;
use message::OutgoingMessage;
use responder::Responder;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str;
use std::sync::mpsc;
use std::thread;

// Here, the static life time does not refer to the whole life of the program as I would have expected,
// but rather only to the life of this struct. I was able to demonstrate this by borrowing a DummyResponder
// and attempting to use it in this position. That didn't work. However, an owend DummyResponder works
// fine.
pub struct Core {
    config: Config
}

impl Core {
    pub fn with_config(config: Config) -> Core {
        Core { config }
    }

    // As you can see, this method consumes the Core and returns only a handle to the sender thread
    // spawned by this function. This is intended to provide something of a workaround for the (very annoying)
    // problems I had with hiirc: to wit, the fact that it was a practical impossibility to initiate any
    // kind of communication from the client side. Communication initiated from the server side is fine for
    // a bot that reacts to stimuli in the channel or from ... You know, whatever ... but it's not so good for
    // what I had in mind, which was, for instance, the ability to send arbitrary commands from the client
    // side, or to carry out certain functions on a scheduled basis, etc.
    pub fn connect<T: Responder + 'static>(self, address: &str, responder: T) -> mpsc::Sender<OutgoingMessage> {
        let Core { config } = self;

        println!("Connecting...");
        let tx_stream = TcpStream::connect(address).unwrap();
        let rx_stream = tx_stream.try_clone().unwrap();

        // let (tx_stream, rx_stream) = if false {
        //     println!("Securing connection...");
        //     (
        //         connector.connect(address, tx_stream).unwrap(),
        //         connector.connect(address, rx_stream).unwrap(),
        //     )
        // } else {
        //     (tx_stream, rx_stream)
        // };

        // Oddly enough, these type annotations are required in order to get this to compile.
        let (responder_tx, responder_rx) = mpsc::channel::<String>();
        let (network_writer_tx, network_writer_rx) = mpsc::channel::<OutgoingMessage>();

        // Responder thread
        //
        // I'm not entirely convinced that this needs to be separate from the reader thread, but I don't want to
        // lock myself into any dumb decisions at this point and the cost of sending a message from that thread
        // to this thread and then on to the writer thread is probably so high that it can't be measured in
        // millionths of a cent, so...
        let write_handle = network_writer_tx.clone();
        thread::spawn(move || {
            for message in responder_rx.iter() {
                if let Some(response) = responder.respond(message) {
                    network_writer_tx.send(response).unwrap();
                }
            }
        });

        // Reader thread
        thread::spawn(move || {
            let rx_stream = BufReader::new(rx_stream);
            for message in rx_stream.lines().filter_map(|s| s.ok()) {
                // println!("rx: {}", message);
                responder_tx.send(message).unwrap();
            }
        });

        // Writer thread
        thread::spawn(move || {
            let mut tx_stream = tx_stream;
            for message in network_writer_rx.iter() {
                // println!("tx: {}", config.format(&message));
                write!(tx_stream, "{}\r\n", config.format(&message)).expect("could not send response");
            }
        });

        // Here I make the following assumptions: each bot has one name, one username, and plans to join only one
        // channel. If you want the bot to handle multiple channels that's fine, but you're going to need to use
        // separate instances of the bot in order to pull that off. At least for right now.
        write_handle.send(OutgoingMessage::Nick(None)).unwrap();
        write_handle.send(OutgoingMessage::User).unwrap();
        write_handle.send(OutgoingMessage::Join).unwrap();
        write_handle
    }
}
