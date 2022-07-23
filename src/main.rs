use hidapi::{HidApi, HidDevice, HidResult};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DPad {
    Released,
    NorthWest,
    West,
    SouthWest,
    South,
    SouthEast,
    East,
    NorthEast,
    North,
}

impl Default for DPad {
    fn default() -> Self {
        DPad::Released
    }
}

impl DPad {
    fn from_byte(b: u8) -> Self {
        match b & 0x0f {
            0x08 => DPad::Released,
            0x07 => DPad::NorthWest,
            0x06 => DPad::West,
            0x05 => DPad::SouthWest,
            0x04 => DPad::South,
            0x03 => DPad::SouthEast,
            0x02 => DPad::East,
            0x01 => DPad::NorthEast,
            0x00 => DPad::North,
            _ => panic!("invalid dpad value: 0b{:04b}", b),
        }
    }
}

struct Controls {
    triangle: Button<bool>,
    circle: Button<bool>,
    x: Button<bool>,
    square: Button<bool>,
    dpad: Button<DPad>,
    r3: Button<bool>,
    l3: Button<bool>,
    options: Button<bool>,
    share: Button<bool>,
    r2: Button<bool>,
    l2: Button<bool>,
    r1: Button<bool>,
    l1: Button<bool>,
    tpad: Button<bool>,
    ps: Button<bool>,
}

impl Controls {
    fn new() -> Self {
        Controls {
            triangle: Button::default(),
            circle: Button::default(),
            x: Button::default(),
            square: Button::default(),
            dpad: Button::default(),
            r3: Button::default(),
            l3: Button::default(),
            options: Button::default(),
            share: Button::default(),
            r2: Button::default(),
            l2: Button::default(),
            r1: Button::default(),
            l1: Button::default(),
            tpad: Button::default(),
            ps: Button::default(),
        }
    }

    fn update(&mut self, report: &[u8]) {
        self.triangle.update(report[5] & 0x80 > 0);
        self.circle.update(report[5] & 0x40 > 0);
        self.x.update(report[5] & 0x20 > 0);
        self.square.update(report[5] & 0x10 > 0);
        self.dpad.update(DPad::from_byte(report[5]));
        self.r3.update(report[6] & 0x80 > 0);
        self.l3.update(report[6] & 0x40 > 0);
        self.options.update(report[6] & 0x20 > 0);
        self.share.update(report[6] & 0x10 > 0);
        self.r2.update(report[6] & 0x08 > 0);
        self.l2.update(report[6] & 0x04 > 0);
        self.r1.update(report[6] & 0x02 > 0);
        self.l1.update(report[6] & 0x01 > 0);
        self.tpad.update(report[7] & 0x02 > 0);
        self.ps.update(report[7] & 0x01 > 0);
    }
}

struct Controller {
    device: HidDevice,
    controls: Controls,
}

impl Controller {
    fn new(device: HidDevice) -> Controller {
        Controller {
            device,
            controls: Controls::new(),
        }
    }

    fn open(api: &HidApi) -> HidResult<Controller> {
        api.open(1356, 2508).map(|device| Controller::new(device))
    }

    fn update(&mut self) -> HidResult<()> {
        let mut report = [0u8; 64];
        let _ = self.device.read(&mut report)?;

        self.controls.update(&report);

        Ok(())
    }
}

struct Button<T> {
    state: T,
    handler: Option<ButtonHandler<T>>,
}

type ButtonHandler<T> = fn(T, T);

impl<T: Default + Eq + Copy> Button<T> {
    fn new(state: T) -> Self {
        Button {
            state,
            handler: None,
        }
    }

    fn default() -> Self {
        Button::new(T::default())
    }

    fn set_handler(&mut self, handler: ButtonHandler<T>) {
        self.handler = Some(handler);
    }

    fn update(&mut self, new_state: T) {
        if self.state != new_state {
            let old_state = self.state;
            self.state = new_state;
            if let Some(handler) = self.handler.as_ref() {
                handler(old_state, new_state);
            }
        }
    }
}

struct RateLimiter {
    interval: Duration,
    last_iter: Instant,
}

impl RateLimiter {
    fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_iter: Instant::now() - interval,
        }
    }

    fn wait(&mut self) {
        let last_iter_duration = Instant::now() - self.last_iter;
        if last_iter_duration < self.interval {
            let delay = self.interval - last_iter_duration;
            thread::sleep(delay);
        }

        self.last_iter = Instant::now();
    }
}

fn main() {
    let api = HidApi::new().unwrap();

    let mut controller = Controller::open(&api).expect("Coudln't open controller");

    controller
        .controls
        .square
        .set_handler(|old_state, new_state| {
            if !old_state && new_state {
                println!("SQUARE PRESSED");
            }
        });

    controller
        .controls
        .triangle
        .set_handler(|old_state, new_state| {
            if !old_state && new_state {
                println!("TRIANGLE PRESSED");
            }
        });

    controller
        .controls
        .dpad
        .set_handler(|old_state, new_state| {
            println!("dpad: {:?} => {:?}", old_state, new_state);
        });

    const TARGET_FPS: u64 = 60;
    let mut rl = RateLimiter::new(Duration::from_millis(1000 / TARGET_FPS));

    loop {
        rl.wait();
        controller.update().expect("failed to update controller");
    }
}
