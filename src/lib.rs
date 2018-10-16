extern crate unix_named_pipe as pipe;
extern crate libc;
extern crate mio;
extern crate crossbeam;

pub mod mio_chan;

use std::io::{self, ErrorKind, Read, Write};
use std::fs::{self, OpenOptions, File};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

pub struct ReadPipe {
    path: PathBuf,
    pub file: File,
    buf: Vec<u8>,
}

pub struct WritePipe {
    path: PathBuf,
    file: File,
}

impl ReadPipe {
   pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref().to_owned();
        if !path.exists() {
            eprintln!("reader creating {:?}", &path);
            pipe::create(&path, None)?;
        }
        let file = pipe::open_read(&path)?;
        let buf = Vec::new();
        Ok(ReadPipe { path, file, buf })
    }

   //pub fn reopen(&mut self) -> io::Result<()> {
       //let file = OpenOptions::new()
        //.read(true)
        //.open(&self.path)?;
       //self.file = file;
       //Ok(())
   //}

    //pub fn make_writer(&self) -> io::Result<WritePipe> {
        //let file = OpenOptions::new()
        //.write(true)
        //.append(true)
        //.open(&self.path)?;
        ////let file = pipe::open_write(&self.path)?;
        //Ok(WritePipe { path: self.path.clone(), file })
    //}

    pub fn read(&mut self) -> Option<&[u8]> {
        let result = self.file.read(&mut self.buf);
        match result {
            Err(e) => {
                eprintln!("READERR {:?}", &e);
                None
            }
            Ok(n) => Some(&self.buf[..n])
        }
    }
}

impl Drop for ReadPipe {
    fn drop(&mut self) {
        fs::remove_file(&self.path);
    }
}

impl WritePipe {
   pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref().to_owned();
        if !path.exists() {
            eprintln!("writer creating {:?}", &path);
            pipe::create(&path, None)?;
        }
        let file = pipe::open_write(&path)?;
        Ok(WritePipe { path, file })
    }


    pub fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        let mut data = data;
        while !data.is_empty() {
            match self.file.write(data) {
                Ok(0) => return Err(io::Error::new(ErrorKind::WriteZero,
                                               "failed to write whole data")),
                Ok(n) => data = &data[n..],
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub fn flush(&mut self) {
        self.file.flush().unwrap();
    }
}

pub struct LazyWritePipe {
    path: PathBuf,
    inner: Option<WritePipe>,
}

impl LazyWritePipe {
    pub fn new<P: AsRef<Path>>(path: P) -> LazyWritePipe {
        let path = path.as_ref().to_owned();
        LazyWritePipe { path, inner: None }
    }

    pub fn write_all<B: AsRef<[u8]>>(&mut self, msg: B) -> io::Result<()> {
        let needs_inner = self.inner.is_none();
        if needs_inner {
            let inner = WritePipe::new(&self.path).unwrap();
            self.inner = Some(inner);
            eprintln!("creating writer thing");
        }
        self.inner.as_mut().unwrap()
            .write_all(msg.as_ref())
    }
}

