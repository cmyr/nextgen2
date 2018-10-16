//! A toy bidrectional connection using unix named pipes.
//!
//! Start the client and server independently, passing the 'client' and 'server'
//! args, or start the server as a subprocess of the client by passing the 'spawn'
//! arg.

extern crate rand;
extern crate unix_named_pipe as pipe;

use std::io::{self, BufRead, BufReader, ErrorKind, Read, Write};
use std::fs::{self, File};
use std::process;
use std::path::Path;
use std::thread;

use rand::Rng;
use pipe::FileFIFOExt;

const IN_PIPE: &str = "/tmp/xi/xi_in_pipe";
const OUT_PIPE: &str = "/tmp/xi/xi_out_pipe";

// make me smaller to go fast
const SEND_DELAY_MILLIS: u32 = 100;

fn spin_read<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut reader = BufReader::new(reader);
    loop {
        match reader.read_until(b'\n', &mut buf) {
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => thread::yield_now(),
            Err(e) => return Err(e),
            Ok(0) => return Err(io::Error::new(ErrorKind::UnexpectedEof, "hmm")),
            Ok(_) => break,
        }
    }
    buf.pop();
    Ok(buf)
}

#[allow(dead_code)]
fn spin_read_no_buf<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    loop {
        match reader.read(&mut buf) {
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => thread::yield_now(),
            Err(e) => return Err(e),
            Ok(0) => return Err(io::Error::new(ErrorKind::UnexpectedEof, "hmm")),
            Ok(_) => break,
        }
    }
    buf.pop();
    Ok(buf)
}

fn spin_open_write_pipe<P: AsRef<Path>>(path: P) -> io::Result<File> {
    let path = path.as_ref();
    loop {
        match pipe::open_write(path) {
            Err(ref e) if e.kind() == ErrorKind::NotFound
                ||  e.raw_os_error() == Some(6) => thread::yield_now(),
            other => break other,
        }
    }
}

fn make_read_pipe<P: AsRef<Path>>(path: P) -> io::Result<File> {
    let path = path.as_ref();
    if !path.exists() {
        pipe::create(&path, None)?;
    }
    let file = pipe::open_read(&path)?;
    assert!(file.is_fifo().expect("is_fifo read failed"), "{:?} is not a fifo pipe", &path);
    Ok(file)
}

fn main() {
    let parent = Path::new(OUT_PIPE).parent().unwrap();
    if !parent.exists() {
        fs::create_dir_all(parent).expect("couldn't create directory");
    }

    match ::std::env::args().nth(1).as_ref().map(|s| s.as_str()) {
        Some("server") => server(),
        Some("client") => client(),
        Some("spawn") => spawn(),
        _ => eprintln!("valid arg is one of ['server', 'client', 'spawn']]"),
    }
}

fn spawn() {
    let mut child = start_server();
    client();
    child.wait().unwrap();
}

fn start_server() -> process::Child {
    process::Command::new("target/debug/examples/bidirectional")
        .arg("server")
        .spawn()
        .expect("spawn failed")
}

#[allow(deprecated)]
fn client() {
    let mut read_pipe = make_read_pipe(IN_PIPE).expect("client failed to open read pipe");
    let mut write_pipe = spin_open_write_pipe(OUT_PIPE).expect("client failed to open write pipe");

    let mut rng = rand::thread_rng();
    let op = rand::seq::sample_slice(&mut rng, &[b'+', b'-', b'*', b'/'], 1);
    write_pipe.write_all(&[op[0], b'\n']).expect("first write failed");

    loop {
        let msg: u8 = rng.gen();
        if msg == 10 { continue; } // don't sent b'\n'
        write_pipe.write_all(&[msg, b'\n']).expect("client write failed");
        let result = match spin_read(&mut read_pipe) {
        //let result = match spin_read_no_buf(&mut read_pipe) {
            Ok(v) => v,
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break println!("server closed the connection, exiting"),
            Err(other) => break println!("other client read error: {:?}", other),
        };
        println!(">> {}", String::from_utf8(result).unwrap());
        thread::sleep_ms(SEND_DELAY_MILLIS);
    }

    let _ = fs::remove_file(OUT_PIPE);
    let _ = fs::remove_file(IN_PIPE);
}


fn server() {
    let mut read_pipe = make_read_pipe(OUT_PIPE).expect("server failed to open read pipe");
    let mut write_pipe = spin_open_write_pipe(IN_PIPE).expect("server failed to open write pipe");

    let op = spin_read(&mut read_pipe).expect("server init read failed");
    //let op = spin_read_no_buf(&mut read_pipe).expect("server read failed");
    assert!(!op.is_empty(), "server initial read is empty");
    let op: char = op[0].into();
    write_pipe.write_all(format!("using {} operator\n", op).as_bytes()).expect("server init write failed");
    let mut running_total: Option<i64> = None;

    loop {
        let result = spin_read(&mut read_pipe).expect("server read failed");
        //let result = spin_read_no_buf(&mut read_pipe).expect("server read failed");
        assert_eq!(result.len(), 1, "server read incomplete: {:?}", &result);
        let number = result[0] as i8 as i64;
        if number == 0 {
            eprintln!("null byte receieved, server will exit");
            break;
        }
        if running_total.is_none() || running_total == Some(0) {
            running_total = Some(number);
            write_pipe.write_all(format!("start value = {}\n", number).as_bytes())
                .expect("server write failed");
        } else {
            let cur_val = running_total.unwrap();
            let new_val = match op {
                '+' => cur_val + number,
                '-' => cur_val - number,
                '*' => cur_val.wrapping_mul(number),
                '/' => cur_val / number,
                other => panic!("unexpected operator: '{}'", other),
            };
            running_total = Some(new_val);
            write_pipe.write_all(format!("{} {} {} = {}\n", cur_val, op, number, new_val).as_bytes())
                .expect("server write failed");
        }
    }
}
