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

    TPad,
    Ps
}

// see https://web.archive.org/web/20210301230721/https://www.psdevwiki.com/ps4/DS4-USB
const BINARY_BUTTONS: &[(usize, &[(u8, Button)])] = &[
    (5, &[
        (0x80, Button::Triangle),
        (0x40, Button::Circle),
        (0x20, Button::X),
        (0x10, Button::Square),
    ]),
    (6, &[
        (0x80, Button::R3),
        (0x40, Button::L3),
        (0x20, Button::Options),
        (0x10, Button::Share),
        (0x08, Button::R2),
        (0x04, Button::L2),
        (0x02, Button::R1),
        (0x01, Button::L1),
    ]),
    (7, &[
        (0x02, Button::TPad),
        (0x01, Button::Ps),
    ])
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

const REPORT_SIZE: usize = 64;
type Report = [u8; REPORT_SIZE];
const EMPTY_REPORT: Report = [0; REPORT_SIZE];

struct Controller {
    device: HidDevice,
    prev_report: Cell<Report>,
}

#[inline]
fn changes_to_events(a: u8, b: u8, event_type: ButtonEventType, masks: &[(u8, Button)]) -> Vec<ButtonEvent> {
    let bits_set = (a ^ b) & b;
    get_pressed_buttons(bits_set, masks)
    .into_iter()
    .map(move |btn| ButtonEvent::new(btn, event_type)).collect()
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

    fn poll_events(&self) -> HidResult<Vec<ButtonEvent>> {
        let mut report: Report = [0; 64];
        let _ = self.device.read(&mut report)?;

        let prev_report = self.prev_report.get();

        let mut events = vec![];

        for (i, masks) in BINARY_BUTTONS.iter() {
            let curr = report[*i];
            let prev = prev_report[*i];

            if curr != prev {
                events.extend(changes_to_events(prev, curr, ButtonEventType::Pressed, &masks));
                events.extend(changes_to_events(curr, prev, ButtonEventType::Released, &masks));
            }
        }

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
