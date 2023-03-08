use borsh::BorshDeserialize;
use lazy_static::lazy_static;
use std::io;
use tokio::net::{TcpListener, TcpStream};
use xor_mailer::{Envelope, Mailer, MailerConfig};

lazy_static! {
    static ref MAILER_CONFIG: MailerConfig = MailerConfig::init();
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6363").await.unwrap();

    println!("Running on socket 127.0.0.1:6363");

    loop {
        let (socket, socket_addr) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process_socket(socket).await.unwrap();
        });
    }
}

async fn process_socket(stream: TcpStream) -> anyhow::Result<()> {
    let mut stream_data = Vec::<u8>::new();

    loop {
        // Wait for the socket to be readable
        stream.readable().await?;

        // Creating the buffer **after** the `await` prevents it from
        // being stored in the async task.
        let mut buffer = [0u8; 4096];

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_read(&mut buffer) {
            Ok(0) => break,
            Ok(byte_read_length) => {
                stream_data.append(&mut buffer[..byte_read_length].to_owned());

                break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    let envelope = Envelope::try_from_slice(&stream_data)?;

    Mailer::new(&*MAILER_CONFIG)
        .add_envelope(envelope)
        .send()
        .await?;

    Ok(())
}
