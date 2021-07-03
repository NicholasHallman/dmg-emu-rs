
use dmg_emu;
use pixels::{ Pixels };
use winit::{dpi::PhysicalSize, event::{Event, VirtualKeyCode}, event_loop::ControlFlow, window::Window};
use winit_input_helper::WinitInputHelper;
use std::{env, sync::mpsc::{self, TryRecvError, Receiver, Sender}, thread};

enum EmuStatus {
    Draw,
    Close
}

enum StatusTypes {
    PhysicalSize(PhysicalSize<u32>),
    Buttons(u8),
    None
}

impl From<PhysicalSize<u32>> for StatusTypes {
    fn from(size: PhysicalSize<u32>) -> Self {
        StatusTypes::PhysicalSize(size)
    }
}

impl From<u8> for StatusTypes {
    fn from(buttons: u8) -> Self {
        StatusTypes::Buttons(buttons)
    }
}


enum WindowStatus {
    Close,
    Resize,
    Draw,
    Buttons
}

fn main() {

    let mut rom_name: Option<String> = None;
        // get the rom name
    for (i, argument) in env::args().enumerate() {
        println!("argument: {}\n", argument);
        if i == 1 {
            rom_name = Some(argument);
        } else {
            rom_name = Some("".to_string());
        }
    }

    // initialize window
    let (event_loop, pixels, window) = dmg_emu::screen::init(rom_name.unwrap()).unwrap();
    let mut input = WinitInputHelper::new();

    // initialize thread communications
    let (emu_sender, emu_receiver) = mpsc::channel();
    let (window_sender, window_receiver) = mpsc::channel();

    // start loops
    start_emulator_thread(pixels, window_receiver, emu_sender);

    event_loop.run(move |event, _, control_flow|{

        if let Event::RedrawRequested(_) = event {
            window_sender.send((WindowStatus::Draw, StatusTypes::None)).unwrap();
        }
        
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                window_sender.send((WindowStatus::Close, StatusTypes::None))
                    .expect("Failed to send to thread");
                return;
            } else if input.key_pressed(VirtualKeyCode::Up) || 
                input.key_pressed(VirtualKeyCode::Down) ||
                input.key_pressed(VirtualKeyCode::Left) ||
                input.key_pressed(VirtualKeyCode::Right) ||
                input.key_pressed(VirtualKeyCode::Z) ||
                input.key_pressed(VirtualKeyCode::X) ||
                input.key_pressed(VirtualKeyCode::Return) ||
                input.key_pressed(VirtualKeyCode::LShift) 
            {
                let value = input.key_pressed(VirtualKeyCode::Up) as u8 |
                    (input.key_pressed(VirtualKeyCode::Down) as u8) << 1 |
                    (input.key_pressed(VirtualKeyCode::Left) as u8) << 2 |
                    (input.key_pressed(VirtualKeyCode::Right) as u8) << 3 |
                    (input.key_pressed(VirtualKeyCode::Z) as u8) << 4 |
                    (input.key_pressed(VirtualKeyCode::X) as u8) << 5 |
                    (input.key_pressed(VirtualKeyCode::Return) as u8) << 6 |
                    (input.key_pressed(VirtualKeyCode::LShift) as u8) << 7;

                window_sender.send((WindowStatus::Buttons, StatusTypes::Buttons(value))).expect("Failed to send buttons to thread");
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                window_sender.send((WindowStatus::Resize, StatusTypes::PhysicalSize(size))).unwrap();
            }

            // start an emu thread

            // if we have a handle then
            match emu_receiver.try_recv() {
                Ok(EmuStatus::Draw) => {
                    window.request_redraw();
                },
                Ok(EmuStatus::Close) => {
                    *control_flow = ControlFlow::Exit;
                },
                Err(mpsc::TryRecvError::Disconnected) => *control_flow = ControlFlow::Exit,
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
    });

    fn start_emulator_thread(mut pixels: Pixels<Window>, window_receiver: Receiver<(WindowStatus, StatusTypes)>, emu_sender: Sender<EmuStatus>) {
        
        let mut rom_name: Option<String> = None;
        // get the rom name
        for (i, argument) in env::args().enumerate() {
            println!("argument: {}\n", argument);
            if i == 1 {
                rom_name = Some(argument);
            }
        }

        let mut emu = dmg_emu::Emu::new(true) ;

        match rom_name {
            Some(name) => {
                emu.load_rom(name);
                emu.cpu.PC = 0x0100;
                emu.cpu.SP = 0xFFFE;
                emu.cpu.AF = 0x1180;

                emu.debug.add_pc_breakpoint(0x0100);
                //emu.debug.add_pc_breakpoint(0xC505);
                //emu.debug.add_pc_breakpoint(0xC35C);
            },
            None => {
                emu.load_boot_rom();
                //emu.debug.add_pc_breakpoint(0x0000);
                // emu.debug.add_breakpoint(6);
                // emu.debug.add_breakpoint(28);
                // emu.debug.add_breakpoint(0x3F);
                // emu.debug.add_breakpoint(0x45);
                // emu.debug.add_breakpoint(0x47);
            } 
        }

        thread::spawn(move || loop {

            emu.ppu.ready = false;
            let mut should_die = false;
    
            while !emu.ppu.ready {
                // listen to window events and use pixel if need
                let win_received: Result<(WindowStatus, StatusTypes), TryRecvError> = window_receiver.try_recv();
                match win_received {
                    Ok((WindowStatus::Draw, _)) => {
                        emu.ppu.draw(pixels.get_frame());
                        if pixels
                            .render()
                            .map_err(|e| println!("Error printing {}", e.to_string()))
                            .is_err()
                        {
                            emu_sender.send(EmuStatus::Close).unwrap();
                            return;
                        }
                    },
                    Ok((WindowStatus::Resize, size)) => if let StatusTypes::PhysicalSize(given) = size {
                        pixels.resize(given.width, given.height);  
                    },
                    Ok((WindowStatus::Close, _)) => {
                        should_die = true;
                        break;
                    },
                    Ok((WindowStatus::Buttons, status)) => if let StatusTypes::Buttons(buttons) = status {
                        emu.button_states(buttons);
                    }
                    Err(mpsc::TryRecvError::Disconnected) => emu_sender.send(EmuStatus::Close).unwrap(),
                    Err(mpsc::TryRecvError::Empty) => {},
                };
    
                // then tick the emulator
                emu.tick();
            }
    
            // let the window thread know that we need to redraw
            emu_sender.send(EmuStatus::Draw).unwrap();
            if should_die {break;}
        });

    }
    
}
