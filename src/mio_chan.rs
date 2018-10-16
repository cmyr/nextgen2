use std::io;
use mio::{Ready, Registration, Poll, PollOpt, Token, SetReadiness};
use mio::event::Evented;
use crossbeam::channel;

pub struct EventReceiver<T> {
    receiver: channel::Receiver<T>,
    registration: Registration,
}

pub struct EventSender<T> {
    sender: channel::Sender<T>,
    set_readiness: SetReadiness,
}

pub fn evented_channel<T>() -> (EventSender<T>, EventReceiver<T>) {
    let (registration, set_readiness) = Registration::new2();
    let (sender, receiver) = channel::unbounded();
    (EventSender { sender, set_readiness }, EventReceiver { receiver, registration })
}

impl<T> EventSender<T> {
    pub fn send(&self, msg: T) {
        self.sender.send(msg);
        self.set_readiness.set_readiness(Ready::readable());
    }
}

impl<T> EventReceiver<T> {
    pub fn try_recv(&self) -> Option<T> {
        self.receiver.try_recv()
    }
}


impl<T> Evented for EventReceiver<T> {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt)
        -> io::Result<()>
    {
        self.registration.register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt)
        -> io::Result<()>
    {
        self.registration.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.registration.deregister(poll)
    }
}
