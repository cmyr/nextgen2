#[macro_use]
extern crate serde_json;
//#[macro_use]
//extern crate serde_derive;
extern crate serde;
extern crate mio;

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::sync::mpsc::{self, Sender};

fn main() {
println!("Hello, world!");
}

//struct XiEndpoint;
//struct ViewId;

//impl XiEndpoint {
//fn new_view(&mut self) -> ViewId {
    //ViewId
//}
//}

// Okay so: first we want to be _transport_ agnostic; our only real
// rule is that we should be communicating over a pair of channels, of some kind.
//

trait RpcSender<M>: Send {
    fn send(&mut self, msg: M) -> Result<(), ()>;
}

trait RpcReceiver<T>: Send {
    fn recv(&self) -> Result<T, ()>;
}

//type MySender<M> = Box<RpcSender<M>>;

trait MyWrite {
    fn write_all(&mut self, buf: &[u8]) -> ::std::io::Result<()>;
}

impl MyWrite for ::std::io::Stdout {
    fn write_all(&mut self, buf: &[u8]) -> ::std::io::Result<()> {
        <Self as Write>::write_all(self, buf)
    }
}

impl<W, M> RpcSender<M> for W
where
W: MyWrite + Send,
M: Serialize,
{
    fn send(&mut self, msg: M) -> Result<(), ()> {
        let mut s = serde_json::to_string(&msg).map_err(|_e| ())?;
        s.push('\n');
        self.write_all(s.as_bytes()).map_err(|_e| ())
    }
}

impl<M: Send> RpcSender<M> for Sender<M> {
    fn send(&mut self, msg: M) -> Result<(), ()> {
        Sender::<_>::send(self, msg).map_err(|_e| ())
    }
}

struct Peer<I, O> {
    send: Box<RpcSender<O>>,
    recv: Box<RpcReceiver<I>>,
}

impl<I, O> Peer<I, O> {
    fn split(self) -> (Box<RpcSender<O>>, Box<RpcReceiver<I>>) {
        let Peer { send, recv } = self;
        (send, recv)
    }
}

fn my_fun() -> impl Iterator<Item=usize> {
    (0..10).into_iter()
}

struct FakeTx;

struct FakeCore {
    send: FakeTx,
}

impl FakeCore {
    fn new(tx: FakeTx) -> Self {
        FakeCore { send: tx }
    }
}

/// Starts a runloop scoped to the current thread.
fn start_scoped(tx: FakeTx) -> FakeCore {
    crossbeam::scope(|scope| {

    });
    FakeCore::new(tx)
}

struct EventSource;
struct ViewStateEtc;

fn my_client() {
    // our runloop
    let (tx, rx) = make_fake_chan();
    let mut core = start_scoped(tx);
    let mut user_events = EventSource;
    let mut our_views = ViewStateEtc;
    loop {
        let event = select!(rx, user_events);
        our_views.handle_event(event);
        if our_views.should_exit() {
            break;
        }
    }
    // in another thread, we want to start the runloop.
    // we pass it in a channel that we will receive messages on
    // we get back an interface that we can send messages to
    //select!(input, )
        //rt
}
