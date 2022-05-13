use std::io::{ErrorKind,Read,Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const LOCAL_ADDRESS: &str = "127.0.0.1:5000";
const MSG_MAX_SIZE: usize = 32;

fn main() {

    let server = TcpListener::bind(LOCAL_ADDRESS).expect("Failed to bind on address.");

    server.set_nonblocking(true).expect("Failed to initialize non-blocking server.");

    let mut clients = vec![];

    let (tx,rx) = mpsc::channel::<String>();

    loop {
        if let Ok((mut socket,addr)) = server.accept() {
            println!("Client {} connected!",addr);
            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed to clone client."));

            thread::spawn(move || loop {
                let mut buff = vec![0;MSG_MAX_SIZE];
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf-8 message");

                        println!("{}: {:?}",addr,msg);
                        tx.send(msg).expect("Failed to send message to rx.");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with {}",addr);
                        break;
                    }
                }
                hibernate();
            });
        };

        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map( |mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_MAX_SIZE,0);
                client.write_all(&buff).map( |_| client).ok()
            })
                .collect::<Vec<_>>();
        };
        hibernate();
    };
}

fn hibernate() {
    thread::sleep(Duration::from_millis(100));
}
