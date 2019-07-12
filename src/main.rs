use audio_sim::oscillator;
use audio_sim::wave_simulator;
use audio_sim::wave_simulator::*;
use audio_sim::SIZE;
use audio_sim::SLEEP_TIME;

use std::sync::mpsc::*;
use std::sync::*;
use std::thread;
use std::time::Instant;

use hound;

fn main() -> std::io::Result<()> {
    let mut wave_simulator = WaveSimulator::new(SIZE).unwrap();
    wave_simulator.add_gauss(50.0, 50.0, 1.0, 1.0);

    let mem_gui = Arc::new(Mutex::new(vec![0.0; SIZE * SIZE]));
    let mem_copy = Arc::clone(&mem_gui);

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        gui(mem_copy, tx);
    });

    let mut writer = get_writer();

    let mut now = Instant::now();
    let mut mic_l_pos = [0, 0];
    let mut mic_r_pos = [0, 0];
    let tx_order_vec = wave_simulator.tx_order.clone();
    for (t, space) in wave_simulator.enumerate() {
        let mut order = None;
        if let Ok(o) = rx.try_recv() {
            match o {
                Order::Drop(x, y, f) => order = Some(Order::Drop(x, y, f)),
                Order::MoveMic(mic, pos) => match mic {
                    Mic::Left => mic_l_pos = pos,
                    Mic::Right => mic_r_pos = pos,
                },
                Order::WaveSim(ws_order) => match ws_order {
                    wave_simulator::Order::Change(Parameter::PropagationRatio(x, y, value)) => {
                        order = Some(o.clone())
                    }
                    wave_simulator::Order::Change(Parameter::DumpingRatio(x, y, value)) => {
                        order = Some(o.clone())
                    }
                    _ => tx_order_vec.iter().for_each(|tx| {
                        tx.send(ws_order.clone()).unwrap();
                    }),
                },
                Order::Quit => break,
                _ => {}
            }
        }

        let mut sp;
        {
            let mut s = space.lock().unwrap();
            sp = s.clone();

            if let Some(o) = order {
                match o {
                    Order::Drop(x, y, f) => {
                        s.space[x + y * SIZE] += f;
                    }
                    Order::WaveSim(ws_order) => match ws_order {
                        wave_simulator::Order::Change(Parameter::PropagationRatio(x, y, value)) => {
                            s.space_spec[x + y * SIZE].0 = value;
                            s.space[x + y * SIZE] = 0.0;
                        }
                        wave_simulator::Order::Change(Parameter::DumpingRatio(x, y, value)) => {
                            s.space_spec[x + y * SIZE].1 = value;
                            s.space[x + y * SIZE] = 0.0;
                        }
                    },
                    _ => (),
                }
            }
        }

        let amplitude = std::i16::MAX as f32 / 10.0;
        writer
            .write_sample(
                (sp.space[mic_l_pos[0] as usize + mic_l_pos[1] as usize * SIZE] * amplitude) as i16,
            )
            .unwrap();
        writer
            .write_sample(
                (sp.space[mic_r_pos[0] as usize + mic_r_pos[1] as usize * SIZE] * amplitude) as i16,
            )
            .unwrap();

        if now.elapsed().as_millis() > 10 {
            let mem_throw = sp
                .space
                .as_slice()
                .iter()
                .enumerate()
                .map(|(i, v)| v / 20.0 + (1.0 - sp.space_spec[i].0) + sp.space_spec[i].1 / 2.0)
                .collect::<Vec<f32>>();
            {
                let mut m = mem_gui.lock().unwrap();
                *m = mem_throw;
                m[mic_l_pos[0] as usize + mic_l_pos[1] as usize * SIZE] = 100.0;
                m[mic_r_pos[0] as usize + mic_r_pos[1] as usize * SIZE] = 100.0;
            }
            now = Instant::now();
        }

        //println!("{} ms", now.elapsed().as_millis());

        // every 100 ms
        if t % 4410 == 0 {
            println!("{} s", t as f32 / 44100.0);
        }

        thread::sleep(std::time::Duration::from_millis(SLEEP_TIME));
    }

    Ok(())
}

#[derive(Copy, Clone, Debug)]
enum Mic {
    Left,
    Right,
}

#[derive(Copy, Clone, Debug)]
enum Order {
    Drop(usize, usize, f32),
    MoveMic(Mic, [usize; 2]),
    WaveSim(wave_simulator::Order),
    Quit,
}

fn gui(mem: Arc<Mutex<Vec<f32>>>, tx: Sender<Order>) {
    use audio_sim::gui;
    use imgui::*;

    let mut drop_pos: [i32; 2] = [0, 0];
    let mut drop_f: f32 = 0.0;
    let mut mic_l_pos: [i32; 2] = [0, 0];
    let mut mic_r_pos: [i32; 2] = [0, 0];
    let mut oscillate: bool = false;
    let mut propagration_ratio: f32 = 0.2;
    let mut dumping_ratio: f32 = 0.1;
    let mut mode: i32 = 0;

    gui::run("Audio Simulator".to_owned(), mem, |mut run, mut ui| {
        ui_func(
            &mut ui,
            &mut run,
            tx.clone(),
            &mut drop_pos,
            &mut drop_f,
            &mut mic_l_pos,
            &mut mic_r_pos,
            &mut oscillate,
            &mut propagration_ratio,
            &mut dumping_ratio,
            &mut mode,
        )
    });

    fn ui_func<'a>(
        ui: &mut Ui<'a>,
        run: &mut bool,
        tx: Sender<Order>,
        mut drop_pos: &mut [i32; 2],
        mut drop_f: &mut f32,
        mut mic_l_pos: &mut [i32; 2],
        mut mic_r_pos: &mut [i32; 2],
        mut oscillate: &mut bool,
        mut propagration_ratio: &mut f32,
        mut dumping_ratio: &mut f32,
        mut mode: &mut i32,
    ) -> bool {
        ui.window(im_str!("nanamin!!"))
            .size([500.0, 300.0], Condition::FirstUseEver)
            .build(move || {
                ui.text(im_str!("wave!"));
                ui.separator();
                /*
                ui.slider_int2(im_str!("drop pos"), &mut drop_pos, 0, SIZE as i32)
                    .build();
                    */

                ui.slider_float(im_str!("drop f"), &mut drop_f, -10.0, 10.0)
                    .build();

                *drop_f += ui.imgui().mouse_wheel() / 10.0;

                /*
                if ui.button(im_str!("drop!"), (80.0, 20.0)) {
                    println!(
                        "drop!\n(x, y, f) : ({}, {}, {})",
                        drop_pos.0], drop_pos.1], drop_f
                    );
                    tx.send(Order::Drop(
                        drop_pos.0] as u32 as usize,
                        drop_pos.1] as u32 as usize,
                        *drop_f,
                    ))
                    .unwrap();
                }
                */

                if ui
                    .slider_float(
                        im_str!("propagration ratio"),
                        &mut propagration_ratio,
                        0.0,
                        1.0,
                    )
                    .build()
                {
                    /*
                    tx.send(Order::WaveSim(wave_simulator::Order::Change(
                        wave_simulator::Parameter::PropagationRatio(10, 10, *propagration_ratio),
                    )))
                    .unwrap();
                    */
                }

                if ui
                    .slider_float(im_str!("dumping ratio"), &mut dumping_ratio, 0.0, 2.0)
                    .build()
                {
                    /*
                    tx.send(Order::WaveSim(wave_simulator::Order::Change(
                        wave_simulator::Parameter::DumpingRatio(10, 10, *dumping_ratio),
                    )))
                    .unwrap();
                    */
                }
                if ui.button(im_str!("fill spec!"), [80.0, 20.0]) {
                    for x in 0..SIZE {
                        for y in 0..SIZE {
                            tx.send(Order::WaveSim(wave_simulator::Order::Change(
                                wave_simulator::Parameter::DumpingRatio(x, y, *dumping_ratio),
                            )))
                            .unwrap();
                            tx.send(Order::WaveSim(wave_simulator::Order::Change(
                                wave_simulator::Parameter::PropagationRatio(
                                    x,
                                    y,
                                    *propagration_ratio,
                                ),
                            )))
                            .unwrap();
                        }
                    }
                }

                if ui.imgui().is_mouse_down(MouseButton::Left)
                    && *mode == 1
                    && !ui.is_window_focused()
                {
                    let mouse_pos = ui.imgui().mouse_pos();
                    let frame_size = ui.io().display_size;
                    let x = (mouse_pos.0 / frame_size[0] as f32 * SIZE as f32) as u32 as usize;
                    let y =
                        ((1.0 - mouse_pos.1 / frame_size[1] as f32) * SIZE as f32) as u32 as usize;
                    if x < SIZE && y < SIZE {
                        tx.send(Order::WaveSim(wave_simulator::Order::Change(
                            wave_simulator::Parameter::DumpingRatio(x, y, *dumping_ratio),
                        )))
                        .unwrap()
                    }
                }

                if ui.imgui().is_mouse_down(MouseButton::Left)
                    && *mode == 1
                    && !ui.is_window_focused()
                {
                    let mouse_pos = ui.imgui().mouse_pos();
                    let frame_size = ui.io().display_size;
                    let x = (mouse_pos.0 / frame_size[0] as f32 * SIZE as f32) as u32 as usize;
                    let y =
                        ((1.0 - mouse_pos.1 / frame_size[1] as f32) * SIZE as f32) as u32 as usize;
                    if x < SIZE && y < SIZE {
                        tx.send(Order::WaveSim(wave_simulator::Order::Change(
                            wave_simulator::Parameter::PropagationRatio(x, y, *propagration_ratio),
                        )))
                        .unwrap();
                    }
                }

                if ui
                    .slider_int2(im_str!("mic l pos"), &mut mic_l_pos, 0, SIZE as i32)
                    .build()
                {
                    tx.send(Order::MoveMic(
                        Mic::Left,
                        [mic_l_pos[0] as u32 as usize, mic_l_pos[1] as u32 as usize],
                    ))
                    .unwrap();
                }

                if ui.imgui().is_mouse_down(MouseButton::Left)
                    && !ui.is_window_focused()
                    && *mode == 0
                {
                    let mouse_pos = ui.imgui().mouse_pos();
                    let frame_size = ui.io().display_size;
                    let x = (mouse_pos.0 / frame_size[0] as f32 * SIZE as f32) as u32 as usize;
                    let y =
                        ((1.0 - mouse_pos.1 / frame_size[1] as f32) * SIZE as f32) as u32 as usize;
                    if x < SIZE && y < SIZE {
                        mic_l_pos[0] = x as i32;
                        mic_l_pos[1] = y as i32;
                        tx.send(Order::MoveMic(
                            Mic::Left,
                            [mic_l_pos[0] as u32 as usize, mic_l_pos[1] as u32 as usize],
                        ))
                        .unwrap();
                    }
                }
                if ui
                    .slider_int2(im_str!("mic r pos"), &mut mic_r_pos, 0, SIZE as i32)
                    .build()
                {
                    tx.send(Order::MoveMic(
                        Mic::Right,
                        [mic_r_pos[0] as u32 as usize, mic_r_pos[1] as u32 as usize],
                    ))
                    .unwrap();
                }

                if ui.imgui().is_mouse_down(MouseButton::Right)
                    && !ui.is_window_focused()
                    && *mode == 0
                {
                    let mouse_pos = ui.imgui().mouse_pos();
                    let frame_size = ui.io().display_size;
                    let x = (mouse_pos.0 / frame_size[0] as f32 * SIZE as f32) as u32 as usize;
                    let y =
                        ((1.0 - mouse_pos.1 / frame_size[1] as f32) * SIZE as f32) as u32 as usize;
                    if x < SIZE && y < SIZE {
                        mic_r_pos[0] = x as i32;
                        mic_r_pos[1] = y as i32;
                        tx.send(Order::MoveMic(
                            Mic::Right,
                            [mic_r_pos[0] as u32 as usize, mic_r_pos[1] as u32 as usize],
                        ))
                        .unwrap();
                    }
                }

                ui.radio_button(im_str!("normal mode!"), mode, 0);
                ui.radio_button(im_str!("spec mode!"), mode, 1);

                if ui.button(im_str!("quit!"), [80.0, 20.0]) {
                    println!("quit!");
                    *run = false;
                    tx.send(Order::Quit).unwrap();
                }
                if ui.imgui().is_mouse_clicked(MouseButton::Middle) {
                    *oscillate = !oscillate.clone();
                }

                if *oscillate {
                    let mouse_pos = ui.imgui().mouse_pos();
                    let frame_size = ui.io().display_size;
                    let x = (mouse_pos.0 / frame_size[0] as f32 * SIZE as f32) as u32 as usize;
                    let y =
                        ((1.0 - mouse_pos.1 / frame_size[1] as f32) * SIZE as f32) as u32 as usize;
                    if x < SIZE && y < SIZE {
                        tx.send(Order::Drop(x, y, *drop_f)).unwrap();
                    }
                }

                let mouse_pos = ui.imgui().mouse_pos();
                ui.text(im_str!("{} {}", mouse_pos.0, mouse_pos.1));
            });
        true
    }
}

fn get_writer() -> hound::WavWriter<std::io::BufWriter<std::fs::File>> {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    hound::WavWriter::create("mic.wav", spec).unwrap()
}
