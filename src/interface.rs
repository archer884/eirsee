use config::Config;
use message::OutgoingMessage;
use responder::Responder;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::str;
use std::sync::mpsc;
use std::thread;

// Here, the static life time does not refer to the whole life of the program as I would have expected,
// but rather only to the life of this struct. I was able to demonstrate this by borrowing a DummyResponder
// and attempting to use it in this position. That didn't work. However, an owend DummyResponder works
// fine.
pub struct Core<T: 'static> {
    config: Config,
    responder: T
}

impl<T: Responder> Core<T> {
    pub fn new(responder: T) -> Core<T> {
        Core {
            config: Config::default(),
            responder: responder,
        }
    }

    // As you can see, this method consumes the Core and returns only a handle to the sender thread
    // spawned by this function. This is intended to provide something of a workaround for the (very annoying)
    // problems I had with hiirc: to wit, the fact that it was a practical impossibility to initiate any
    // kind of communication from the client side. Communication initiated from the server side is fine for
    // a bot that reacts to stimuli in the channel or from ... You know, whatever ... but it's not so good for
    // what I had in mind, which was, for instance, the ability to send arbitrary commands from the client
    // side, or to carry out certain functions on a scheduled basis, etc.
    pub fn run<A: ToSocketAddrs>(self, address: A) -> mpsc::Sender<OutgoingMessage> {
        let Core { config, responder } = self;
        let mut tx_stream = TcpStream::connect(address).expect("unable to connect");
        let mut rx_stream = BufReader::new(tx_stream.try_clone().expect("unable to clone stream"));

        // Oddly enough, these type annotations are required in order to get this to compile.
        let (reader_to_responder_tx, reader_to_responder_rx) = mpsc::channel::<String>();
        let (responder_to_writer_tx, responder_to_writer_rx) = mpsc::channel::<OutgoingMessage>();

        // Responder thread
        //
        // I'm not entirely convinced that this needs to be separate from the reader thread, but I don't want to
        // lock myself into any dumb decisions at this point and the cost of sending a message from that thread
        // to this thread and then on to the writer thread is probably so high that it can't be measured in
        // millionths of a cent, so...
        let write_handle = responder_to_writer_tx.clone();
        thread::spawn(move || {
            for message in reader_to_responder_rx.iter() {
                if let Some(response) = responder.respond(message) {
                    responder_to_writer_tx.send(response);
                }
            }
        });

        // Reader thread
        thread::spawn(move || {
            for message in rx_stream.lines().filter_map(|s| s.ok()) {
                println!("rx: {}", message);
                reader_to_responder_tx.send(message);
            }
        });

        // Writer thread
        thread::spawn(move || {
            for message in responder_to_writer_rx.iter() {
                println!("tx: {}", config.format(&message));
                write!(tx_stream, "{}\r\n", config.format(&message))
                    .expect("could not send response");
            }
        });

        write_handle.send(OutgoingMessage::Nick);
        write_handle.send(OutgoingMessage::User);
        write_handle.send(OutgoingMessage::Join);

        write_handle
    }
}
