//use mio::

// what do we need, to get an event?
// we need some source of raw data, and some way of turning that data into
// some event we know how to handle.

struct State {
    peers: Vec<()>,
}

enum EventSource {
    Client,
    Plugin,
    Idle,
    Timer,
    FileSystem,
}

type Token = usize;

const IDLE_TOKEN: TOKEN = usize::max_value();
const CLIENT_TOKEN: TOKEN = 1;
const NO_TIMEOUT: Duration = Duration::from_micros(1);

struct EventStream {
    sources: HashMap<Token, EventSource>,
    event_buffer: Events,
    ready_events: BinaryHeap<Token>,
    has_idle: bool,
}

impl EventStream {
    /// Handle newly ready events, determining priority etc.
    fn process_new_events(&mut self) {
        for event in &self.event_buffer {
            if event.is_readable() {
                let token = event.get_token();
                if token == IDLE_TOKEN {
                    self.has_idle = true;
                } else {
                    ready_events.push(event.get_token());
                }
            }
        }
    }
}

impl Iterator for EventStream {
    type Item = ();
    fn next(&mut self) -> Option<()> {
        let timeout: Option<Duration> = match self.ready_events.is_empty() {
            false => Some(NO_TIMEOUT),
            true => None,
        };
        poll.poll(&mut self.events, Some(NO_TIMEOUT));
        self.process_new_events();
        debug_assert!(!self.ready_events.is_empty());
        self.ready_events.pop()
    }
}

fn loop(state: &mut State) {
    let mut events = Events::with_capacity(1024);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in &events {

        }
    }
}
