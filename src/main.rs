mod chip8;

use std::{
    fs::File,
    io::{BufReader, Read},
    thread,
};

use chip8::{display::DisplayInstruction, keypad::Event, settings::Settings, Chip8};
use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::{
    egui::{self, Sense},
    epaint::{Color32, Pos2, Rect, Rounding, Vec2},
};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 800.0)),
        ..Default::default()
    };

    let file = File::open("roms/6-keypad.ch8").unwrap();
    let mut reader = BufReader::new(file);
    let mut program = Vec::new();

    reader.read_to_end(&mut program).unwrap();

    let (display_sender, display_receiver) = unbounded();
    let (event_sender, event_receiver) = unbounded();

    thread::spawn(move || {
        let settings = Settings::default();
        let mut chip8 = Chip8::new(settings, &program, display_sender, event_receiver);
        chip8.run()
    });

    eframe::run_native(
        "Chip8 Emulator",
        options,
        Box::new(|_cc| Box::new(MyApp::new(display_receiver, event_sender))),
    )
}

struct MyApp {
    display_buffer: Box<[bool]>,
    display_receiver: Receiver<DisplayInstruction>,
    event_sender: Sender<Event>,
}

impl MyApp {
    fn new(display_receiver: Receiver<DisplayInstruction>, event_sender: Sender<Event>) -> Self {
        let display_buffer = vec![false; 2048].into_boxed_slice();
        Self {
            display_buffer,
            display_receiver,
            event_sender,
        }
    }
}

const RECT_SIZE: usize = 12;

static KEY_MAP: &'static [(egui::Key, chip8::keypad::Key)] = &[
    (egui::Key::Num1, chip8::keypad::Key::Key1),
    (egui::Key::Num2, chip8::keypad::Key::Key2),
    (egui::Key::Num3, chip8::keypad::Key::Key3),
    (egui::Key::Num4, chip8::keypad::Key::Key4),
    (egui::Key::Q, chip8::keypad::Key::KeyQ),
    (egui::Key::W, chip8::keypad::Key::KeyW),
    (egui::Key::E, chip8::keypad::Key::KeyE),
    (egui::Key::R, chip8::keypad::Key::KeyR),
    (egui::Key::A, chip8::keypad::Key::KeyA),
    (egui::Key::S, chip8::keypad::Key::KeyS),
    (egui::Key::D, chip8::keypad::Key::KeyD),
    (egui::Key::F, chip8::keypad::Key::KeyF),
    (egui::Key::Z, chip8::keypad::Key::KeyZ),
    (egui::Key::X, chip8::keypad::Key::KeyX),
    (egui::Key::C, chip8::keypad::Key::KeyC),
    (egui::Key::V, chip8::keypad::Key::KeyV),
];

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(
                Vec2 {
                    x: (RECT_SIZE * 64) as f32,
                    y: (RECT_SIZE * 32) as f32,
                },
                Sense::hover(),
            );

            for (egui_key, chip8_key) in KEY_MAP {
                if ui.input(|i| i.key_down(*egui_key)) {
                    self.event_sender.send(Event::KeyDown(*chip8_key)).unwrap();
                }
                if ui.input(|i| i.key_released(*egui_key)) {
                    self.event_sender.send(Event::KeyUp(*chip8_key)).unwrap();
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.event_sender.send(Event::Stop).unwrap();
            }

            let x_offset = response.rect.left();
            let y_offset = response.rect.top();

            while let Ok(instruction) = self.display_receiver.try_recv() {
                match instruction {
                    DisplayInstruction::Set { value, index } => self.display_buffer[index] = value,
                    DisplayInstruction::Clear => self.display_buffer.fill(false),
                }
            }

            for x in 0..64 {
                for y in 0..32 {
                    let index = x + y * 64;
                    let set = self.display_buffer[index];
                    let colour = if set { Color32::WHITE } else { Color32::BLACK };
                    let rect = Rect {
                        min: Pos2 {
                            x: (x * RECT_SIZE) as f32 + x_offset,
                            y: (y * RECT_SIZE) as f32 + y_offset,
                        },
                        max: Pos2 {
                            x: ((x + 1) * RECT_SIZE) as f32 + x_offset,
                            y: ((y + 1) * RECT_SIZE) as f32 + y_offset,
                        },
                    };
                    painter.rect_filled(rect, Rounding::none(), colour)
                }
            }
            ui.ctx().request_repaint()
        });
    }
}
