use audio_sim::wave_simulator::*;
use gnuplot::*;
use gnuplot::PlotOption::*;

use std::fs::*;

fn main() -> std::io::Result<()>
{
    const SIZE: usize = 60;
    let mut wave_simulator = WaveSimulator::new(SIZE).unwrap();
    //wave_simulator.add_gauss(20.0, 25.0, 5.0, 5.0);
    //wave_simulator.add_gauss(30.0, 40.0, 1.0, 25.0);
    //wave_simulator.add_gauss(20.0, 19.0, 2.0, 20.0);
    //wave_simulator.add_gauss(30.0, 30.0, 5.0, 10.0);

    let mut fg = Figure::new();

    for (t, space) in wave_simulator.enumerate()
    {
        let mut sp;
        {
            let s = space.lock().unwrap();
            sp = s.clone();
        }
        
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
        
    }

    Ok(())
}

