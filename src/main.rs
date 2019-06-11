use audio_sim::wave_simulator::*;
use gnuplot::*;
use gnuplot::PlotOption::*;
use hound;

use std::fs::*;
use std::i16;

fn main() -> std::io::Result<()>
{
    const SIZE: usize = 36;
    let mut wave_simulator = WaveSimulator::new(SIZE).unwrap();
    //wave_simulator.add_gauss(20.0, 25.0, 5.0, 5.0);

    let mut fg = Figure::new();

    let mut writer = get_writer();

    for (t, space) in wave_simulator.enumerate()
    {
        let mut sp;
        {
            let mut s = space.lock().unwrap();
            sp = s.clone();
            if t % 30 == 0
            {
                s.space[(SIZE/7)*SIZE + SIZE/3] += 3.0;
            }
            else if t % 15 == 0
            {
                s.space[(SIZE/2)*SIZE + SIZE/5] -= 3.0;
            }

            if (t+10) % 20 == 0
            {
                s.space[(SIZE/3)*SIZE + SIZE/4] += 3.0;
            }
            else if (t+10) % 10 == 0
            {
                s.space[(SIZE/4)*SIZE + SIZE/2] -= 3.0;
            }

            if t % 1000 == 0
            {
                println!("{}", t);
            }
            let amplitude = i16::MAX as f32 / 2 as f32;
            writer.write_sample((s.space[(SIZE-1)*SIZE + SIZE-20] * amplitude) as i16).unwrap();

        }
        /*
        fg.clear_axes();
        fg.axes3d()
            .set_z_range(Fix(-10.0), Fix(10.0))
            .set_cb_range(Fix(-5.0), Fix(5.0))
            .set_palette(HELIX)
            .set_view(30.0, 30.0)
            .surface(
                sp.space,
                SIZE,
                SIZE,
                None,
                &[Caption(t.to_string().as_str())],
                );
        fg.show();
        */
        if t > 44100
        {
            break;
        }
    }

    Ok(())
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
