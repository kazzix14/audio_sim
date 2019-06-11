use std::thread;
use std::sync::*;
use std::fmt::*;

use super::Space;

const NUM_THREADS: usize = 6;

#[derive(Clone)]
pub struct WaveSimulator
{
    pub space_previous: Arc<Mutex<Space>>,
    pub space_current: Arc<Mutex<Space>>,
    pub space_size: usize,
    tx: Vec<mpsc::Sender<(usize, mpsc::Sender<usize>)>>,
}

impl WaveSimulator
{
    pub fn new(size: usize) -> std::result::Result<WaveSimulator, ()>
    {
        if size % NUM_THREADS != 0
        {
            return Err(())
        }

        let space_current = Arc::new(Mutex::new(Space::new(size)));
        let space_previous = Arc::new(Mutex::new(Space::new(size)));

        let len_per_thread = size / NUM_THREADS;
        let mut tx_vec: Vec<mpsc::Sender<(usize, mpsc::Sender<usize>)>> = Vec::new();

        for _ in 0..NUM_THREADS
        {
            let (tx, rx) = mpsc::channel();
            tx_vec.push(tx);
            let space_current = Arc::clone(&space_current);
            let space_previous = Arc::clone(&space_previous);

            thread::spawn(
                    move ||
                    {   
                        const dx : f32 = 0.1;
                        const dt : f32 = 1.0/60.0;
                        const c : f32 = 1.0;
                        const coefficient :f32 = (dt*dt) * (c*c) / (dx*dx);
                        const k : f32 = 0.2;

                        while let Ok((i, tx)) = rx.recv()
                        {

                            let mut y_start = i*len_per_thread*size - size;
                            let mut y_end = (i+1)*len_per_thread*size + size;

                            let limit = NUM_THREADS-1;
                            
                            if i == 0
                            {
                                y_start += size
                            }

                            if i == NUM_THREADS-1
                            {
                                y_end -= size
                            }

                            let mut my_current;
                            
                            {
                                let mut current_space = space_current.lock().unwrap();
                                my_current = current_space.space[y_start..y_end].to_vec().clone();
                            }
                            

                            if i == 0
                            {
                                let tmp = my_current;
                                my_current = vec![0.0; size];
                                my_current.extend_from_slice(tmp.as_slice());
                            }
                            if i == NUM_THREADS-1
                            {
                                my_current.extend_from_slice(vec![0.0; size].as_slice());
                            }

                            let mut my_previous;
                            {
                                let mut previous_space = space_previous.lock().unwrap();
                                my_previous = previous_space.space[y_start..y_end].to_vec().clone();
                            }

                            if i == 0
                            {
                                let tmp = my_previous;
                                my_previous = vec![0.0; size];
                                my_previous.extend_from_slice(tmp.as_slice());
                            }
                            if i == NUM_THREADS-1
                            {
                                my_previous.extend_from_slice(vec![0.0; size].as_slice());
                            }

                            y_start = 1;
                            y_end = len_per_thread+1;

                            for y in y_start..y_end
                            {
                                for x in 1..size-1
                                {
                                    let value_previous = my_previous[x + y*size];
                                    let value_current = my_current[x + y*size];
                                    
                                    let value_left   = my_previous[x-1 + y*size];
                                    let value_right  = my_current[x+1 + y*size]; 
                                    let value_top    = my_previous[x + (y-1)*size];
                                    let value_bottom = my_current[x + (y+1)*size];

                                    let damp = -k * dt * (value_current - value_previous);
                                    
                                    my_previous[x + y*size] = value_current;
                                    my_current[x + y*size] = 2.0*value_current - value_previous + coefficient*(-4.0 * value_current + value_left + value_right + value_top + value_bottom) + damp;
                                }
                            }
                            
                            let start = i*len_per_thread*size;
                            let end = (i+1)*len_per_thread*size;

                            let my_start = size;
                            let my_end = len_per_thread*size+size;

                            {
                                    
                                let mut current = space_current.lock().unwrap();
                                let mut previous = space_previous.lock().unwrap();
                                let mut current_space = current.space.clone();
                                let mut previous_space = previous.space.clone();

                                let mut current_b = current_space.split_off(start);
                                let mut current_b = current_b.split_off(len_per_thread*size);
                                current_space.append(&mut my_current[my_start..my_end].to_vec());
                                current_space.append(&mut current_b);
                                current.space = current_space;

                                let mut previous_b = previous_space.split_off(start);
                                let mut previous_b = previous_b.split_off(len_per_thread*size);
                                previous_space.append(&mut my_previous[my_start..my_end].to_vec());
                                previous_space.append(&mut previous_b);
                                previous.space = previous_space;
                            }

                            tx.send(i);
                        }
                    }
                );
        }

        Ok(WaveSimulator{
            space_previous: space_previous,
            space_current: space_current,
            space_size: size,
            tx: tx_vec,
        })
    }

    pub fn add_gauss(&mut self, x: f32, y: f32, sigma: f32, power: f32)
    {
        self.space_current.lock().unwrap().add_gauss(x, y, sigma, power);
        self.space_previous.lock().unwrap().add_gauss(x, y, sigma, power);
    }
}

impl Iterator for WaveSimulator
{
    type Item = Arc<Mutex<Space>>;
    fn next(&mut self) -> Option<Self::Item>
    {
        const dx : f32 = 0.1;
        const dt : f32 = 1.0/60.0;
        const c : f32 = 1.0;
        const coefficient :f32 = (dt*dt) * (c*c) / (dx*dx);

        let len_per_thread = self.space_size / NUM_THREADS;
        let size_pow2 = self.space_size.pow(2);

        let (tx_r, rx_r) = mpsc::channel();
        self.tx.iter().enumerate().for_each(
                |(i, tx)|
                {
                    tx.send((i, tx_r.clone()));
                }
            );
        let mut count_received = 0;
        for received in rx_r
        {
            count_received += 1;
            if count_received >= NUM_THREADS
            {
                break;
            }
        }

        /*
        let space_size = self.space_size;
        for y in 1..space_size-1
        {
            for x in 1..space_size-1
            {
                let value_previous = self.space_previous.get(x, y);
                let value_current = self.space_current.get(x, y);

                let value_left   = self.space_previous.get(x-1, y);
                let value_right  = self.space_current.get(x+1, y); 
                let value_top    = self.space_previous.get(x, y-1);
                let value_bottom = self.space_current.get(x, y+1);

                self.space_previous.put(x, y, value_current);
                self.space_current.put(x, y, 2.0*value_current - value_previous + coefficient*(-4.0 * value_current + value_left + value_right + value_top + value_bottom));
            }
        }
        */

        Some(self.space_current.clone())

    }
}

