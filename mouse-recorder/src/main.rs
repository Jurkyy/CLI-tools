use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use device_query::{DeviceQuery, DeviceState};
use rdev::{simulate, Button, EventType};
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
enum MouseAction {
    Move,
    LeftClick,
    LeftRelease,
    RightClick,
    RightRelease,
}

#[derive(Clone, Debug, PartialEq)]
struct MouseEvent {
    x: i32,
    y: i32,
    action: MouseAction,
    timestamp: Duration,
}

struct MouseRecorder {
    events: Vec<MouseEvent>,
    device_state: DeviceState,
}

impl MouseRecorder {
    fn new() -> Self {
        MouseRecorder {
            events: Vec::new(),
            device_state: DeviceState::new(),
        }
    }

    fn start_recording(&mut self) {
        println!("Recording started. Press ESC to stop recording...");
        self.events.clear();

        let start_time = Instant::now();
        let mut last_pos = None;
        let mut last_left_state = false;
        let mut last_right_state = false;

        enable_raw_mode().unwrap();

        loop {
            // Check for ESC key
            if event::poll(Duration::from_millis(1)).unwrap() {
                if let Event::Key(key_event) = event::read().unwrap() {
                    if key_event.code == KeyCode::Esc {
                        break;
                    }
                }
            }

            // Get current mouse state
            let mouse_state = self.device_state.get_mouse();
            let current_pos = (mouse_state.coords.0 as i32, mouse_state.coords.1 as i32);
            let left_pressed = mouse_state.button_pressed[1];
            let right_pressed = mouse_state.button_pressed[2];

            // Record position changes
            if last_pos != Some(current_pos) {
                self.events.push(MouseEvent {
                    x: current_pos.0,
                    y: current_pos.1,
                    action: MouseAction::Move,
                    timestamp: start_time.elapsed(),
                });
                last_pos = Some(current_pos);
            }

            // Record left button state changes
            if left_pressed != last_left_state {
                self.events.push(MouseEvent {
                    x: current_pos.0,
                    y: current_pos.1,
                    action: if left_pressed {
                        MouseAction::LeftClick
                    } else {
                        MouseAction::LeftRelease
                    },
                    timestamp: start_time.elapsed(),
                });
                last_left_state = left_pressed;
            }

            // Record right button state changes
            if right_pressed != last_right_state {
                self.events.push(MouseEvent {
                    x: current_pos.0,
                    y: current_pos.1,
                    action: if right_pressed {
                        MouseAction::RightClick
                    } else {
                        MouseAction::RightRelease
                    },
                    timestamp: start_time.elapsed(),
                });
                last_right_state = right_pressed;
            }

            thread::sleep(Duration::from_millis(1)); // 1000Hz sampling
        }

        disable_raw_mode().unwrap();

        println!(
            "\nRecording stopped. Recorded {} events.",
            self.events.len()
        );
        thread::sleep(Duration::from_millis(100));
    }

    fn play_recording(&self, loop_playback: bool) {
        if self.events.is_empty() {
            println!("No recording to play!");
            return;
        }

        println!("Playing recording... Press ESC to stop.");

        enable_raw_mode().unwrap();

        'playback: loop {
            for (i, event) in self.events.iter().enumerate() {
                // Check for ESC key
                if event::poll(Duration::from_millis(1)).unwrap() {
                    if let Event::Key(key_event) = event::read().unwrap() {
                        if key_event.code == KeyCode::Esc {
                            break 'playback;
                        }
                    }
                }

                // Simulate the event
                let event_type = match event.action {
                    MouseAction::Move => EventType::MouseMove {
                        x: event.x as f64,
                        y: event.y as f64,
                    },
                    MouseAction::LeftClick => EventType::ButtonPress(Button::Left),
                    MouseAction::LeftRelease => EventType::ButtonRelease(Button::Left),
                    MouseAction::RightClick => EventType::ButtonPress(Button::Right),
                    MouseAction::RightRelease => EventType::ButtonRelease(Button::Right),
                };

                if let Err(e) = simulate(&event_type) {
                    eprintln!("Error simulating mouse event: {}", e);
                }

                // Wait until it's time for the next event
                if let Some(next_event) = self.events.get(i + 1) {
                    let sleep_duration = next_event.timestamp - event.timestamp;
                    thread::sleep(sleep_duration);
                }
            }

            if !loop_playback {
                break;
            }

            thread::sleep(Duration::from_millis(500));
        }

        disable_raw_mode().unwrap();

        println!("\nPlayback stopped.");
        thread::sleep(Duration::from_millis(100));
    }

    fn save_to_file(&self, filename: &str) -> io::Result<()> {
        let content = self
            .events
            .iter()
            .map(|event| {
                let action_str = match event.action {
                    MouseAction::Move => "move",
                    MouseAction::LeftClick => "leftdown",
                    MouseAction::LeftRelease => "leftup",
                    MouseAction::RightClick => "rightdown",
                    MouseAction::RightRelease => "rightup",
                };
                format!(
                    "{},{},{},{}",
                    event.x,
                    event.y,
                    action_str,
                    event.timestamp.as_millis()
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        std::fs::write(filename, content)
    }

    fn load_from_file(&mut self, filename: &str) -> io::Result<()> {
        let content = std::fs::read_to_string(filename)?;
        self.events = content
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() == 4 {
                    let action = match parts[2] {
                        "move" => MouseAction::Move,
                        "leftdown" => MouseAction::LeftClick,
                        "leftup" => MouseAction::LeftRelease,
                        "rightdown" => MouseAction::RightClick,
                        "rightup" => MouseAction::RightRelease,
                        _ => return None,
                    };
                    Some(MouseEvent {
                        x: parts[0].parse().ok()?,
                        y: parts[1].parse().ok()?,
                        action,
                        timestamp: Duration::from_millis(parts[3].parse().ok()?),
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok(())
    }
}

fn main() {
    let mut recorder = MouseRecorder::new();

    loop {
        print!(
            "\nMouse Recorder Menu:\n\
                1. Start Recording\n\
                2. Play Recording Once\n\
                3. Play Recording on Loop\n\
                4. Save Recording\n\
                5. Load Recording\n\
                6. Exit\n\
                Enter your choice (1-6): "
        );

        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => recorder.start_recording(),
            "2" => recorder.play_recording(false),
            "3" => recorder.play_recording(true),
            "4" => {
                print!("Enter filename to save: ");
                io::stdout().flush().unwrap();
                let mut filename = String::new();
                io::stdin().read_line(&mut filename).unwrap();
                if let Err(e) = recorder.save_to_file(filename.trim()) {
                    println!("Error saving file: {}", e);
                } else {
                    println!("Recording saved successfully!");
                }
            }
            "5" => {
                print!("Enter filename to load: ");
                io::stdout().flush().unwrap();
                let mut filename = String::new();
                io::stdin().read_line(&mut filename).unwrap();
                if let Err(e) = recorder.load_from_file(filename.trim()) {
                    println!("Error loading file: {}", e);
                } else {
                    println!("Recording loaded successfully!");
                }
            }
            "6" => {
                println!("Exiting...");
                break;
            }
            _ => println!("Invalid choice. Please try again."),
        }
    }
}

