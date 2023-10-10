use std::{process::Command, sync::Mutex, thread};

use indexmap::IndexMap;
use signal_hook::consts::SIGINT;
use signal_hook::iterator::Signals;

use once_cell::sync::OnceCell;

use crate::data::TestResource;

static SIGINT_HANDLER: OnceCell<Mutex<SigintHandler>> = OnceCell::new();

pub struct SigintHandler {
    resources: IndexMap<usize, Command>,
    next_index: usize,
}

impl SigintHandler {
    fn new() -> Self {
        SigintHandler {
            resources: IndexMap::new(),
            next_index: 0,
        }
    }
    /* Get the global singleton SignalHandler instance */
    pub fn get_instance() -> &'static Mutex<SigintHandler> {
        SIGINT_HANDLER.get_or_init(|| {
            Self::start_thread();
            Mutex::new(SigintHandler::new())
        })
    }

    /* Initializes thread that listens for SIGINT. Should be called once at the start of the program */
    fn start_thread() {
        thread::spawn(move || {
            let mut signals = Signals::new([SIGINT]).unwrap();
            signals.forever().next();
            Self::get_instance().lock().unwrap().handle_sigint();
        });
    }

    fn handle_sigint(&mut self) {
        for (_, cmd) in self.resources.iter_mut().rev() {
            let _result = cmd.spawn().and_then(|mut h| h.wait());
        }
    }

    /* Registers a resource for deletion when SIGINT is received. Resources are deleted in LIFO order. */
    pub fn register_resource<R: TestResource>(&mut self, resource: &R) -> SigintResourceHandle {
        self.resources
            .insert(self.next_index, resource.delete_cmd());
        let handle = SigintResourceHandle {
            index: self.next_index,
        };
        self.next_index += 1;
        handle
    }
}

pub struct SigintResourceHandle {
    index: usize,
}

impl Drop for SigintResourceHandle {
    fn drop(&mut self) {
        SigintHandler::get_instance()
            .lock()
            .unwrap()
            .resources
            .remove(&self.index);
    }
}
