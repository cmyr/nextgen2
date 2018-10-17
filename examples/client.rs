
extern crate libc;
extern crate unix_named_pipe as pipe;
extern crate mio;
extern crate crossbeam;
extern crate nextgen2;

use std::io::{self, BufRead, ErrorKind, Read, Write};
use std::os::unix::io::AsRawFd;
//use std::ffi::CString;
use std::fs::{self, OpenOptions, File};
use std::process::Command;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use mio::*;
use mio::unix::EventedFd;

use nextgen2::*;

const IN_PIPE: &str = "/tmp/xi/xi_in_pipe";
const OUT_PIPE: &str = "/tmp/xi/xi_out_pipe";

const STDIN: Token = Token(0);
const SERVER: Token = Token(1);

fn main() {

    //fs::create_dir_all(Path::new(OUT_PIPE).parent().unwrap());
    let _ = fs::remove_file(OUT_PIPE);
    let _ = fs::remove_file(IN_PIPE);

    crossbeam::scope(|scope| {
        let mut child = Command::new("target/debug/examples/server")
            .arg(OUT_PIPE)
            .arg(IN_PIPE)
            .spawn()
            .expect("spawn failed");

        eprintln!("got child {:?}", &child);
        let (send, recv) = mio_chan::evented_channel();

        thread::sleep(Duration::from_millis(1000));
        // start the stdin reading thread:
        scope.spawn(move || {
            let stdin = io::stdin();
            loop {
                let mut stdin = stdin.lock();
                let mut buf = String::new();
                stdin.read_line(&mut buf).expect("stdin read failed??");
                send.send(buf);
            }
        });

        let mut in_pipe = make_read_pipe(IN_PIPE).expect("client read pipe failed");
        let mut out_pipe = spin_open_write_pipe(OUT_PIPE).expect("client write pipe failed");

        let poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(1024);

        poll.register(&recv, STDIN, Ready::readable(), PollOpt::edge())
            .expect("stdin register failed");
        poll.register(&EventedFd(&in_pipe.as_raw_fd()), SERVER,
                      Ready::readable(), PollOpt::edge())
            .expect("register failed");


        loop {
            poll.poll(&mut events, None).unwrap();
            for event in events.iter() {
                match event.token() {
                    STDIN => {
                        let msg = recv.try_recv().expect("no message after ready?");
                        out_pipe.write_all(msg.as_bytes()).unwrap();
                        out_pipe.write_all("\n".as_bytes()).unwrap();
                        //eprint!(">>{}", msg);
                    }
                    SERVER => {
                        let result = spin_read(&mut in_pipe)
                            .map(|b| String::from_utf8(b).expect("client: invalid utf8"))
                            .expect("client read failed");
                        eprintln!("<<'{}'", result);
                    }
                    other => panic!("whoops"),
                }
            }
        }
        child.wait().unwrap();
    });
}

