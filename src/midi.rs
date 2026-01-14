use core::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use midly::live::LiveEvent;
use midly::num::u4;
use midly::MidiMessage;

// using this type alias because LiveEvents need lifetimes and I don't
// want that to pollute my other types
pub type MidiEvent = (u4, MidiMessage);

#[derive(Clone)]
pub struct MidiEventSender {
    event_queue: Rc<RefCell<VecDeque<MidiEvent>>>,
}

impl MidiEventSender {
    pub fn send(&self, event: MidiEvent) {
        self.event_queue.borrow_mut().push_back(event);
    }
}

pub struct MidiConfig {
    pub midi_out: MidiOutput,
    pub ports: Vec<(String, MidiOutputPort, bool)>,
    pub connection: Option<MidiOutputConnection>,

    event_queue: Rc<RefCell<VecDeque<MidiEvent>>>,
}

impl MidiConfig {
    pub fn new() -> Self {
        let midi_out = MidiOutput::new("graf").unwrap();
        let mut midi_cfg = MidiConfig {
            midi_out,
            ports: vec![],
            connection: None,

            event_queue: Rc::new(RefCell::new(VecDeque::new())),
        };

        midi_cfg.refresh_ports();

        midi_cfg
    }

    pub fn refresh_ports(&mut self) {
        self.ports.clear();
        for port in self.midi_out.ports() {
            self.ports
                .push((self.midi_out.port_name(&port).unwrap(), port, false));
        }
    }

    pub fn connect_to_port(&mut self, port: &MidiOutputPort) {
        let midi_conn_out = MidiOutput::new("graf-connection-output").unwrap();
        self.connection = Some(midi_conn_out.connect(port, "graf-midi").unwrap());

        for (_name, p, connected) in self.ports.iter_mut() {
            *connected = port == p;
        }
    }

    pub fn process_events(&mut self) {
        if let Some(conn) = &mut self.connection {
            for (channel, message) in self.event_queue.borrow_mut().drain(..) {
                let mut buf = Vec::new();
                let event = LiveEvent::Midi { channel, message };
                event.write(&mut buf).unwrap();
                conn.send(&buf).unwrap();
            }
        } else {
            // dump any events in queue if there is no active connection
            self.event_queue.borrow_mut().clear();
        }
    }

    pub fn get_event_sender(&self) -> MidiEventSender {
        MidiEventSender {
            event_queue: self.event_queue.clone(),
        }
    }
}
