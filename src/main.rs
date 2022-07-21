use hidapi::{HidApi, HidResult, HidDevice};
use std::time::Instant;
use std::cell::Cell;

#[derive(Debug, Clone, Copy)]
enum Button {
    // picture buttons
    Triangle,
    Circle,
    X,
    Square,
    // d-pad buttons
    NorthWest,
    West,
    SouthWest,
    South,
    SouthEast,
    East,
    NorthEast,
    North,

    // Other buttons
    R3,
    L3,
    Options,
    Share,
    R2,
    L2,
    R1,
    L1,
}

// see https://web.archive.org/web/20210301230721/https://www.psdevwiki.com/ps4/DS4-USB
const PICTURE_BUTTONS: [(u8, Button); 4] = [
    (0x80, Button::Triangle),
    (0x40, Button::Circle),
    (0x20, Button::X),
    (0x10, Button::Square),
];

const OTHER_BUTTONS: [(u8, Button); 8] = [
    (0x80, Button::R3),
    (0x40, Button::L3),
    (0x20, Button::Options),
    (0x10, Button::Share),
    (0x08, Button::R2),
    (0x04, Button::L2),
    (0x02, Button::R1),
    (0x01, Button::L1),
];

const D_PAD_MAP: [(u8, Button); 8]= [
    (0x07, Button::NorthWest),
    (0x06, Button::West),
    (0x05, Button::SouthWest),
    (0x04, Button::South),
    (0x03, Button::SouthEast),
    (0x02, Button::East),
    (0x01, Button::NorthEast),
    (0x00, Button::North),
];

fn get_pressed_buttons(b: u8, masks: &[(u8, Button)]) -> Vec<Button> {
    masks
        .iter()
        .filter_map(|(mask, button)| (b & mask > 0).then(|| *button))
        .collect()
}

#[derive(Debug, Clone, Copy)]
enum ButtonEventType {
    Pressed,
    Released,
}

#[derive(Debug)]
struct ButtonEvent {
    button: Button,
    event_type: ButtonEventType,
    instant: Instant,
}

impl ButtonEvent {
    fn new(button: Button, event_type: ButtonEventType) -> Self {
        ButtonEvent {
            button,
            event_type,
            instant: Instant::now(),
        }
    }
}

#[derive(Debug)]
struct ByteTracker {
    curr: u8,
    prev: u8,
}

fn get_events(curr: u8, prev: u8, masks: &[(u8, Button)]) -> Vec<ButtonEvent> {
            // Find which buttons changed their state using bitwise math.
            // changes = curr ^ prev
            // pressed (0->1) = curr & changes
            // released (1->0) = prev & changes
            // -> also equivalent to !pressed & changes
            let changes = curr ^ prev;
            let pressed = curr & changes;
            let released = prev & changes;

            let mut events = Vec::with_capacity(16);

            let to_events = |b, event_type| {
                get_pressed_buttons(b, masks).into_iter().map(move |b| ButtonEvent::new(b, event_type))
            };

            events.extend(to_events(pressed, ButtonEventType::Pressed));
            events.extend(to_events(released, ButtonEventType::Released));

            events
}

const REPORT_SIZE: usize = 64;
type Report = [u8; REPORT_SIZE];
const EMPTY_REPORT: Report = [0; REPORT_SIZE];

struct Controller {
    device: HidDevice,
    prev_report: Cell<Report>,
}

impl Controller {
    fn new(device: HidDevice) -> Controller {
        Controller {
            device,
            prev_report: Cell::new(EMPTY_REPORT),
        }
    }

    fn open(api: &HidApi) -> HidResult<Controller> {
        api.open(1356, 2508).map(|device| Controller::new(device))
    }

    fn next_report<F: FnOnce(&Report, &Report)>(&mut self, f: F) {

    }

    fn poll_events(&self) -> HidResult<Vec<ButtonEvent>> {
        let mut report: Report = [0; 64];
        let _ = self.device.read(&mut report)?;

        let prev_report = self.prev_report.get();

        let mut events = vec![];

        let mut events_if_different = |i, masks: &[(u8, Button)]| {
            if report[i] != prev_report[i] {
                events.extend(get_events(report[i], prev_report[i], masks));
            }
        };

        // picture buttons and d-pad
        events_if_different(5, &PICTURE_BUTTONS);
        events_if_different(6, &OTHER_BUTTONS);

        // save the current report state for tracking changes.
        self.prev_report.set(report);

        Ok(events)
    }
}

fn main() {
    let api = HidApi::new().unwrap();

    let controller = Controller::open(&api).expect("Coudln't open controller");

    loop {
        match controller.poll_events() {
            Ok(events) => {
                for event in events {
                    match (event.button, event.event_type) {
                        (Button::L1, ButtonEventType::Pressed) => {
                            println!("Left!");
                        },
                        (Button::R1, ButtonEventType::Pressed) => {
                            println!("Right");
                        },
                        (Button::Triangle, ButtonEventType::Pressed) => {
                            println!("Jump");
                        },
                        (Button::X, ButtonEventType::Pressed) => {
                            println!("Crouch");
                        },
                        (Button::Circle, ButtonEventType::Pressed) => {
                            println!("Block");
                        }
                        (Button::Square, ButtonEventType::Pressed) => {
                            println!("Attack");
                        },
                        _ => {},
                    }
                }
            },
            Err(e) => eprintln!("error: {}", e),
        }
    }
}
