//extern crate unix_named_pipe as pipe;
extern crate mio;
extern crate nextgen2;

use std::io::{ErrorKind, Read, Write};
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

    let mut in_pipe = make_read_pipe(&in_pipe_path).expect("client read pipe failed");
    let mut out_pipe = spin_open_write_pipe(&out_pipe_path).expect("client write pipe failed");

    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);
    poll.register(&EventedFd(&in_pipe.as_raw_fd()), CLIENT,
                  Ready::readable(), PollOpt::edge())
        .expect("server register failed");

    out_pipe.write_all(b"ready\n").unwrap();
    loop {
        poll.poll(&mut events, Some(Duration::from_millis(5000))).unwrap();
        if events.is_empty() {
            out_pipe.write_all(b"ping\n").unwrap();
            continue;
        }
        for event in events.iter() {
            match event.token() {
                CLIENT => {
                    loop {
                    match spin_read(&mut in_pipe) {
                        Ok(ref n) => break handle_client_msg(::std::str::from_utf8(n).unwrap(), &mut out_pipe),
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => (), // continue
                        Err(e) => break eprintln!("server read err {:?}", e),
                    }
                    }
                }
                _other => panic!("ahhhhhh"),
            }
        }
        events.clear();
    }
}


fn handle_client_msg<W: Write>(msg: &str, client: &mut W) {
    match msg.trim() {
        "ping" => client.write_all(b"ping\n").unwrap(),
        "hi" => client.write_all(b"hello there!\n").unwrap(),
        n if n.parse::<isize>().is_ok() => {
            let num = n.parse::<isize>().unwrap();
            let numplus = num + 1;
            client.write_all(format!("{}\n", numplus).as_bytes()).unwrap();
        }
        _other => client.write_all(b"huh?\n").unwrap(),
    }
}
