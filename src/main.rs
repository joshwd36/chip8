mod chip8;

use std::{
    fs::File,
    io::{BufReader, Read},
    thread,
};

use chip8::{display::DisplayInstruction, settings::Settings, Chip8};
use crossbeam_channel::{unbounded, Receiver};
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

    let file = File::open("roms/3-corax+.ch8").unwrap();
    let mut reader = BufReader::new(file);
    let mut program = Vec::new();

    reader.read_to_end(&mut program).unwrap();

    let (display_sender, display_receiver) = unbounded();

    thread::spawn(move || {
        let settings = Settings::default();
        let mut chip8 = Chip8::new(settings, &program, display_sender);
        chip8.run()
    });

    eframe::run_native(
        "Chip8 Emulator",
        options,
        Box::new(|_cc| Box::new(MyApp::new(display_receiver))),
    )
}

struct MyApp {
    display_buffer: Box<[bool]>,
    display_receiver: Receiver<DisplayInstruction>,
}

impl MyApp {
    fn new(display_receiver: Receiver<DisplayInstruction>) -> Self {
        let display_buffer = vec![false; 2048].into_boxed_slice();
        Self {
            display_buffer,
            display_receiver,
        }
    }
}

const RECT_SIZE: usize = 12;

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
