use audio_sim::wave_simulator::*;
use audio_sim::SIZE;
use audio_sim::SLEEP_TIME;

use std::thread;
use std::sync::*;
use std::sync::mpsc::*;
use std::time::Instant;

use hound;

fn main() -> std::io::Result<()>
{
    let mut wave_simulator = WaveSimulator::new(SIZE).unwrap();
    wave_simulator.add_gauss(20.0, 25.0, 1.0, 10.0);

    
    let mem_gui = Arc::new(Mutex::new(Vec::new()));
    let mem_copy = Arc::clone(&mem_gui);

    let (tx, rx) = mpsc::channel();

    thread::spawn(move ||{gui(mem_copy, tx);});

    //let mut writer = get_writer();

    for (t, space) in wave_simulator.enumerate()
    {
        let mut order = None;
        if let Ok(o) = rx.try_recv()
        {
            match o
            {
                Order::Drop(x, y, f) => {order = Some(Order::Drop(x, y, f))},
                Order::Quit => break,
                _ => {},
            }
        }
        
        let now = Instant::now();
        let mut sp;
        {
            let mut s = space.lock().unwrap();
            sp = s.clone();
            if let Some(Order::Drop(x, y, f)) = order
            {
                s.space[x + y*SIZE] += f;
            }
        }
            //let amplitude = i16::MAX as f32 / 2  as f32 * (t as f32/44100 as f32);
            //writer.write_sample((s.space[(SIZE-1)*SIZE + SIZE-20] * amplitude) as i16).unwrap();

        {
            let mut m = mem_gui.lock().unwrap();
            *m = sp.space;
        }

        //println!("{} ms", now.elapsed().as_millis());

       /*
        if t > 44100
        {
            break;
        }
        */

        thread::sleep_ms(SLEEP_TIME);
        
    }

    Ok(())
}

enum Order
{
    Drop(usize, usize, f32),
    Quit,
}

fn gui(mem: Arc<Mutex<Vec<f32>>>, tx: Sender<Order>)
{
    use audio_sim::gui;
    use imgui::*;

    let mut drop_pos: [i32; 2] = [0, 0];
    let mut drop_f: f32 = 0.0;

    gui::run("Audio Simulator".to_owned(), mem, |ui, _, _|{
        ui_func(ui, tx.clone(), &mut drop_pos, &mut drop_f)
    });

    fn ui_func<'a>(ui: &Ui<'a>, tx: Sender<Order>, mut drop_pos: &mut [i32; 2], mut drop_f: &mut f32) -> bool
    {
        ui.window(im_str!("nanamin!!"))
            .size((300.0, 300.0), ImGuiCond::FirstUseEver)
            .build(move || {
                ui.text(im_str!("wave!"));
                ui.separator();

                ui.slider_int2(im_str!("drop x"), &mut drop_pos, 0, SIZE as i32).build();
                ui.slider_float(im_str!("drop f"), &mut drop_f, 0.0, 100.0).build();

                if ui.button(im_str!("drop!"), (80.0, 20.0))
                {
                    println!("drop!\n(x, y, f) : ({}, {}, {})", drop_pos[0], drop_pos[1], drop_f);
                    tx.send(Order::Drop(drop_pos[0] as u32 as usize, drop_pos[1] as u32 as usize, *drop_f)).unwrap();
                }

                if ui.button(im_str!("quit!"), (80.0, 20.0))
                {
                    println!("quit!");
                    tx.send(Order::Quit).unwrap();
                }

                let mouse_pos = ui.imgui().mouse_pos();
                ui.text(im_str!("{} {}", mouse_pos.0, mouse_pos.1));
            });
        true
    }
}


fn get_writer() -> hound::WavWriter<std::io::BufWriter<std::fs::File>>
{
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    hound::WavWriter::create("mic.wav", spec).unwrap()
}
