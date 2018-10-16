//extern crate unix_named_pipe as pipe;
extern crate mio;
extern crate nextgen2;

use std::io::{ErrorKind, Read};
use std::os::unix::io::AsRawFd;
use std::thread;
use std::time::Duration;

use mio::*;
use mio::unix::EventedFd;

use nextgen2::*;

const CLIENT: Token = Token(0);

fn main() {
    let mut args = ::std::env::args().skip(1);
    let in_pipe_path = args.next().expect("missing inpipe arg");
    let out_pipe_path = args.next().expect("missing outpipe arg");

    let mut in_pipe = nextgen2::ReadPipe::new(&in_pipe_path).expect("server in open failed");
    //let mut out_pipe = nextgen2::WritePipe::new(&out_pipe_path).expect("server out open failed");
    let mut out_pipe = LazyWritePipe::new(&out_pipe_path);
    eprintln!("server opened '{}'", &in_pipe_path);


    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);
    poll.register(&EventedFd(&in_pipe.file.as_raw_fd()), CLIENT,
                  Ready::readable(), PollOpt::edge())
        .expect("server register failed");

    let mut read_buf = String::new();
    out_pipe.write_all(b"ready").unwrap();
    loop {
        poll.poll(&mut events, Some(Duration::from_millis(2000))).unwrap();
        if events.is_empty() {
            out_pipe.write_all(b"ping").unwrap();
            continue;
        }
        for event in events.iter() {
            match event.token() {
                CLIENT => {
                    loop {
                    match in_pipe.file.read_to_string(&mut read_buf) {
                        Ok(0) => break eprintln!("server read 0 bytes"),
                        Ok(n) => break handle_client_msg(&read_buf, &mut out_pipe),
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => (), // continue
                        Err(e) => break eprintln!("server read err {:?}", e),
                    }
                    }
                }
                _other => panic!("ahhhhhh"),
            }
        }
        events.clear();
        //write.write_all("ping".as_bytes()).unwrap();
        //thread::sleep(Duration::from_millis(1000));
    }
}


fn handle_client_msg(msg: &str, client: &mut LazyWritePipe) {
    match msg.trim() {
        "ping" => client.write_all("ping".as_bytes()).unwrap(),
        "hi" => client.write_all("hello there!".as_bytes()).unwrap(),
        n if n.parse::<isize>().is_ok() => {
            let num = n.parse::<isize>().unwrap();
            let numplus = num + 1;
            client.write_all(format!("{}", numplus).as_bytes()).unwrap();
        }
        other => client.write_all("huh?".as_bytes()).unwrap(),
    }
}
