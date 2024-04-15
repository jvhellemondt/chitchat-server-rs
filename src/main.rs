use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const LOCAL_IP: &str = "127.0.0.1";
const PORT: &str = "6000";
const MSG_SIZE: usize = 32;

fn main() {
    let address = format!("{}:{}", LOCAL_IP.to_owned(), PORT.to_owned());
    let server: TcpListener = match TcpListener::bind(address.as_str()) {
        Ok(listener) => {
            println!("Server listening on {}", address);
            listener
        },
        Err(err) => panic!("Server failed to start, {}", err),
    };

    match server.set_nonblocking(true) {
        Ok(_) => println!("Server set to non-blocking"),
        Err(err) => panic!("Failed to set server to non-blocking, {}", err),
    }

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();

    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);
            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed to clone client"));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter()
                            .take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message received");

                        println!("{}: {:?}", addr, msg);
                        tx.send(msg).expect("Failed to send message to rx")
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Error: closing connection with {}", addr);
                        break;
                    }
                }

                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).map(|_| client).ok()
            })
                .collect::<Vec<_>>();
        }

        sleep();
    }
}

fn sleep() {
    thread::sleep(Duration::from_millis(100))
}
