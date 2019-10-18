pub use winit::{Event, EventsLoop, KeyboardInput, VirtualKeyCode, WindowEvent};
use std::time::Instant;

// handles all the events.
// you can clone it around as much as you want cause it's small
pub struct EventHandler {
    pub frame_info: FrameInfo,
    events_loop: EventsLoop,
    last_frame_time: Instant,
    start_time: Instant,
    frames_drawn: u32,
}

// information about the current frame
#[derive(Clone, Debug)]
pub struct FrameInfo {
    pub all_events: Vec<Event>,
    pub keydowns: Vec<VirtualKeyCode>,
    pub keyups: Vec<VirtualKeyCode>,
    pub keys_down: KeysDown,
    pub mouse_movement: [f32; 2],
    pub delta: f32,
    pub dimensions: [u32; 2],
}

impl EventHandler {
    pub fn new(events_loop: EventsLoop) -> Self {
        Self {
            frame_info: FrameInfo::empty(),
            events_loop,
            last_frame_time: Instant::now(),
            start_time: Instant::now(),
            frames_drawn: 0,
        }
    }

    pub fn update(&mut self, dimensions: [u32; 2]) -> bool {
        // call this before drawing every frame
        self.frame_info.delta = get_elapsed(self.last_frame_time);
        self.last_frame_time = Instant::now();
        self.frames_drawn += 1;

        // if this is our first frame, reset the start time so that loading
        // times don't affect FPS calculations
        if self.frames_drawn == 1 {
            self.start_time = Instant::now();
        }

        self.frame_info.dimensions = dimensions;
        self.collect_events()
    }

    pub fn get_fps(&self) -> f32 {
        (self.frames_drawn as f32) / get_elapsed(self.start_time)
    }

    pub fn collect_events(&mut self) -> bool {
        // returns whether the program should exit or not
        // clobbers all input from the last frame, mind
        // also assumes the mouse was at the center of the screen last frame

        // TODO: try and replace these variables with pointers to members of
        // self
        let mut done = false;
        let mut keydowns = vec![];
        let mut keyups = vec![];
        let mut all_events = vec![];
        let mut cursor_pos = None;

        self.events_loop.poll_events(|ev| {
            match ev.clone() {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => done = true,
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position: p, .. },
                    ..
                } => {
                    cursor_pos = Some(p);
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { .. },
                    ..
                } => {
                    if let Some(keyboard_input) = winit_event_to_keycode(&ev) {
                        match keyboard_input {
                            KeyboardInput {
                                virtual_keycode: Some(key),
                                state: winit::ElementState::Pressed,
                                ..
                            } => keydowns.push(key),
                            KeyboardInput {
                                virtual_keycode: Some(key),
                                state: winit::ElementState::Released,
                                ..
                            } => keyups.push(key),
                            _ => {}
                        }
                    }
                }
                _ => {}
            };
            all_events.push(ev.clone());
        });

        // for avoiding problems with borrow checker
        // append all new keydown events to the list, as well as updating keys_down
        keydowns.iter().for_each(|&keycode| {
            // yeah, this sucks
            match keycode {
                VirtualKeyCode::A => self.frame_info.keys_down.a = true,
                VirtualKeyCode::B => self.frame_info.keys_down.b = true,
                VirtualKeyCode::C => self.frame_info.keys_down.c = true,
                VirtualKeyCode::D => self.frame_info.keys_down.d = true,
                VirtualKeyCode::E => self.frame_info.keys_down.e = true,
                VirtualKeyCode::F => self.frame_info.keys_down.f = true,
                VirtualKeyCode::G => self.frame_info.keys_down.g = true,
                VirtualKeyCode::H => self.frame_info.keys_down.h = true,
                VirtualKeyCode::I => self.frame_info.keys_down.i = true,
                VirtualKeyCode::J => self.frame_info.keys_down.j = true,
                VirtualKeyCode::K => self.frame_info.keys_down.k = true,
                VirtualKeyCode::L => self.frame_info.keys_down.l = true,
                VirtualKeyCode::M => self.frame_info.keys_down.m = true,
                VirtualKeyCode::N => self.frame_info.keys_down.n = true,
                VirtualKeyCode::O => self.frame_info.keys_down.o = true,
                VirtualKeyCode::P => self.frame_info.keys_down.p = true,
                VirtualKeyCode::Q => self.frame_info.keys_down.q = true,
                VirtualKeyCode::R => self.frame_info.keys_down.r = true,
                VirtualKeyCode::S => self.frame_info.keys_down.s = true,
                VirtualKeyCode::T => self.frame_info.keys_down.t = true,
                VirtualKeyCode::U => self.frame_info.keys_down.u = true,
                VirtualKeyCode::V => self.frame_info.keys_down.v = true,
                VirtualKeyCode::W => self.frame_info.keys_down.w = true,
                VirtualKeyCode::X => self.frame_info.keys_down.x = true,
                VirtualKeyCode::Y => self.frame_info.keys_down.y = true,
                VirtualKeyCode::Z => self.frame_info.keys_down.z = true,
                _ => {}
            }
        });
        keyups.iter().for_each(|&keycode| {
            // yeah, this sucks
            // a possible solution: make keys_down a list of VirtualKeyCodes instead
            match keycode {
                VirtualKeyCode::A => self.frame_info.keys_down.a = false,
                VirtualKeyCode::B => self.frame_info.keys_down.b = false,
                VirtualKeyCode::C => self.frame_info.keys_down.c = false,
                VirtualKeyCode::D => self.frame_info.keys_down.d = false,
                VirtualKeyCode::E => self.frame_info.keys_down.e = false,
                VirtualKeyCode::F => self.frame_info.keys_down.f = false,
                VirtualKeyCode::G => self.frame_info.keys_down.g = false,
                VirtualKeyCode::H => self.frame_info.keys_down.h = false,
                VirtualKeyCode::I => self.frame_info.keys_down.i = false,
                VirtualKeyCode::J => self.frame_info.keys_down.j = false,
                VirtualKeyCode::K => self.frame_info.keys_down.k = false,
                VirtualKeyCode::L => self.frame_info.keys_down.l = false,
                VirtualKeyCode::M => self.frame_info.keys_down.m = false,
                VirtualKeyCode::N => self.frame_info.keys_down.n = false,
                VirtualKeyCode::O => self.frame_info.keys_down.o = false,
                VirtualKeyCode::P => self.frame_info.keys_down.p = false,
                VirtualKeyCode::Q => self.frame_info.keys_down.q = false,
                VirtualKeyCode::R => self.frame_info.keys_down.r = false,
                VirtualKeyCode::S => self.frame_info.keys_down.s = false,
                VirtualKeyCode::T => self.frame_info.keys_down.t = false,
                VirtualKeyCode::U => self.frame_info.keys_down.u = false,
                VirtualKeyCode::V => self.frame_info.keys_down.v = false,
                VirtualKeyCode::W => self.frame_info.keys_down.w = false,
                VirtualKeyCode::X => self.frame_info.keys_down.x = false,
                VirtualKeyCode::Y => self.frame_info.keys_down.y = false,
                VirtualKeyCode::Z => self.frame_info.keys_down.z = false,
                _ => {}
            }
        });

        self.frame_info.keydowns = keydowns;
        self.frame_info.keyups = keyups;

        // calculate mouse movement, assuming it used to be at the center of the screen
        if let Some(pos) = cursor_pos {
            let x_diff = pos.x - ((self.frame_info.dimensions[0] / 2) as f64);
            let y_diff = pos.y - ((self.frame_info.dimensions[1] / 2) as f64);

            self.frame_info.mouse_movement = [x_diff as f32, y_diff as f32];
        }

        self.frame_info.all_events = all_events;

        done
    }
}

impl FrameInfo {
    pub fn empty() -> Self {
        Self {
            all_events: vec![],
            keydowns: vec![],
            keyups: vec![],
            keys_down: KeysDown::all_false(),
            mouse_movement: [0.0, 0.0],
            delta: 0.0,
            dimensions: [0, 0],
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeysDown {
    pub a: bool,
    pub b: bool,
    pub c: bool,
    pub d: bool,
    pub e: bool,
    pub f: bool,
    pub g: bool,
    pub h: bool,
    pub i: bool,
    pub j: bool,
    pub k: bool,
    pub l: bool,
    pub m: bool,
    pub n: bool,
    pub o: bool,
    pub p: bool,
    pub q: bool,
    pub r: bool,
    pub s: bool,
    pub t: bool,
    pub u: bool,
    pub v: bool,
    pub w: bool,
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl KeysDown {
    fn all_false() -> Self {
        KeysDown {
            a: false,
            b: false,
            c: false,
            d: false,
            e: false,
            f: false,
            g: false,
            h: false,
            i: false,
            j: false,
            k: false,
            l: false,
            m: false,
            n: false,
            o: false,
            p: false,
            q: false,
            r: false,
            s: false,
            t: false,
            u: false,
            v: false,
            w: false,
            x: false,
            y: false,
            z: false,
        }
    }
}

pub fn get_elapsed(start: std::time::Instant) -> f32 {
    start.elapsed().as_secs() as f32 + start.elapsed().subsec_nanos() as f32 / 1_000_000_000.0
}

fn winit_event_to_keycode(event: &Event) -> Option<winit::KeyboardInput> {
    // only matches key press/release events
    if let Event::WindowEvent {
        event: WindowEvent::KeyboardInput { input, .. },
        ..
    } = event
    {
        if input.virtual_keycode.is_some() {
            Some(*input)
        } else {
            None
        }
    } else {
        None
    }
}
