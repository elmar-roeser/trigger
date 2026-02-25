use rdev::{listen, simulate, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    let active = Arc::new(AtomicBool::new(false));
    let running = Arc::new(AtomicBool::new(true));

    // Ctrl+C handler: release key if active, then exit
    {
        let active = active.clone();
        let running = running.clone();
        ctrlc::set_handler(move || {
            if active.load(Ordering::SeqCst) {
                let _ = simulate(&EventType::KeyRelease(Key::KeyR));
            }
            running.store(false, Ordering::SeqCst);
            println!("\nBeendet.");
            std::process::exit(0);
        })
        .expect("Ctrl+C Handler konnte nicht gesetzt werden");
    }

    // Simulator thread: sends repeated KeyPress while active
    {
        let active = active.clone();
        let running = running.clone();
        thread::spawn(move || {
            let mut was_active = false;
            while running.load(Ordering::SeqCst) {
                if active.load(Ordering::SeqCst) {
                    if !was_active {
                        // Initial delay so user can release Ctrl+Shift
                        thread::sleep(Duration::from_millis(200));
                        was_active = true;
                        // Re-check in case it was toggled off during delay
                        if !active.load(Ordering::SeqCst) {
                            was_active = false;
                            continue;
                        }
                    }
                    let _ = simulate(&EventType::KeyPress(Key::KeyR));
                    thread::sleep(Duration::from_millis(50));
                } else {
                    if was_active {
                        let _ = simulate(&EventType::KeyRelease(Key::KeyR));
                        was_active = false;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
        });
    }

    println!("Trigger bereit. Ctrl+Shift+R zum Togglen. Ctrl+C zum Beenden.");

    // Listener: track modifier state and detect Ctrl+Shift+R
    let ctrl_pressed = Arc::new(AtomicBool::new(false));
    let shift_pressed = Arc::new(AtomicBool::new(false));

    let ctrl = ctrl_pressed.clone();
    let shift = shift_pressed.clone();

    listen(move |event| {
        match event.event_type {
            EventType::KeyPress(Key::ControlLeft | Key::ControlRight) => {
                ctrl.store(true, Ordering::SeqCst);
            }
            EventType::KeyRelease(Key::ControlLeft | Key::ControlRight) => {
                ctrl.store(false, Ordering::SeqCst);
            }
            EventType::KeyPress(Key::ShiftLeft | Key::ShiftRight) => {
                shift.store(true, Ordering::SeqCst);
            }
            EventType::KeyRelease(Key::ShiftLeft | Key::ShiftRight) => {
                shift.store(false, Ordering::SeqCst);
            }
            EventType::KeyPress(Key::KeyR) => {
                if ctrl.load(Ordering::SeqCst) && shift.load(Ordering::SeqCst) {
                    let prev = active.load(Ordering::SeqCst);
                    active.store(!prev, Ordering::SeqCst);
                    if !prev {
                        println!("R wird gehalten...");
                    } else {
                        println!("Gestoppt.");
                    }
                }
            }
            _ => {}
        }
    })
    .expect("Listener konnte nicht gestartet werden");
}
