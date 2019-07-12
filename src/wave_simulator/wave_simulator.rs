use crate::NUM_THREADS;

use std::fmt::*;
use std::sync::*;
use std::thread;

use super::Space;

#[derive(Clone)]
pub struct WaveSimulator {
    pub space_previous: Arc<Mutex<Space>>,
    pub space_current: Arc<Mutex<Space>>,
    pub space_next: Arc<Mutex<Space>>,
    pub space_size: usize,
    tx_update: Vec<mpsc::Sender<(usize, mpsc::Sender<usize>)>>,
    pub tx_order: Vec<mpsc::Sender<(Order)>>,
}

#[derive(Copy, Clone, Debug)]
pub enum Parameter {
    PropagationRatio(usize, usize, f32),
    DumpingRatio(usize, usize, f32),
}

#[derive(Copy, Clone, Debug)]
pub enum Order {
    Change(Parameter),
}

impl WaveSimulator {
    pub fn new(size: usize) -> std::result::Result<WaveSimulator, ()> {
        if size % NUM_THREADS != 0 {
            return Err(());
        }

        let space_current = Arc::new(Mutex::new(Space::new(size)));
        let space_previous = Arc::new(Mutex::new(Space::new(size)));
        let space_next = Arc::new(Mutex::new(Space::new(size)));

        let len_per_thread = size / NUM_THREADS;
        let mut tx_update_vec: Vec<mpsc::Sender<(usize, mpsc::Sender<usize>)>> = Vec::new();
        let mut tx_order_vec: Vec<mpsc::Sender<(Order)>> = Vec::new();

        for _ in 0..NUM_THREADS {
            let (tx_update, rx_update) = mpsc::channel();
            tx_update_vec.push(tx_update);
            let (tx_order, rx_order) = mpsc::channel();
            tx_order_vec.push(tx_order);

            let space_current = Arc::clone(&space_current);
            let space_previous = Arc::clone(&space_previous);
            let space_next = Arc::clone(&space_next);

            thread::spawn(move || {
                const dx: f32 = 0.1;
                const dt: f32 = 1.0 / 60.0;
                //let mut c: f32 = 0.2;
                //let mut coefficient: f32 = (dt * dt) / (dx * dx); // * c *  c
                //let mut k: f32 = 0.1;

                while let Ok((i, tx_update)) = rx_update.recv() {
                    if let Ok(order) = rx_order.try_recv() {
                        use Order::*;
                        use Parameter::*;
                        match order {
                            Change(param) => match param {
                                /*
                                PropagationRatio(x, y, value) => {
                                    //c = value;
                                }
                                DumpingRatio(x, y, value) => {
                                    //k = value;
                                }
                                */
                                _ => {}
                            },
                            _ => {}
                        }
                    }

                    let mut y_start = i * len_per_thread * size;
                    let mut y_end = (i + 1) * len_per_thread * size;

                    if i == 0 {
                        y_start += size
                    }

                    if i == NUM_THREADS - 1 {
                        y_end -= size
                    }
                    y_start -= size;
                    y_end += size;

                    let mut my_space_spec;
                    let mut my_current;

                    {
                        let current_space = space_current.lock().unwrap();
                        my_current = current_space.space[y_start..y_end].to_vec().clone();
                        my_space_spec = current_space.space_spec[y_start..y_end].to_vec().clone();
                    }

                    if i == 0 {
                        let tmp = my_current;
                        my_current = vec![0.0; size];
                        my_current.extend_from_slice(tmp.as_slice());
                    }
                    if i == NUM_THREADS - 1 {
                        my_current.extend_from_slice(vec![0.0; size].as_slice());
                    }

                    let mut my_previous;

                    {
                        let previous_space = space_previous.lock().unwrap();
                        my_previous = previous_space.space[y_start..y_end].to_vec().clone();
                    }

                    if i == 0 {
                        let tmp = my_previous;
                        my_previous = vec![0.0; size];
                        my_previous.extend_from_slice(tmp.as_slice());
                    }
                    if i == NUM_THREADS - 1 {
                        my_previous.extend_from_slice(vec![0.0; size].as_slice());
                    }

                    let mut my_next;

                    {
                        let next_space = space_next.lock().unwrap();
                        my_next = next_space.space[y_start..y_end].to_vec().clone();
                    }

                    if i == 0 {
                        let tmp = my_next;
                        my_next = vec![0.0; size];
                        my_next.extend_from_slice(tmp.as_slice());
                    }
                    if i == NUM_THREADS - 1 {
                        my_next.extend_from_slice(vec![0.0; size].as_slice());
                    }

                    y_start = 1;
                    y_end = len_per_thread + 1;

                    // update each cells
                    for y in y_start..y_end {
                        for x in 1..size - 1 {
                            let value_previous = my_previous[x + y * size];
                            let value_current = my_current[x + y * size];

                            let value_left = my_current[x - 1 + y * size];
                            let value_right = my_current[x + 1 + y * size];
                            let value_top = my_current[x + (y - 1) * size];
                            let value_bottom = my_current[x + (y + 1) * size];

                            let (c, k) = my_space_spec[x + y * size];
                            let coefficient = (dt * dt) * (c * c) / (dx * dx);
                            let damp = -k * dt * (value_current - value_previous);

                            my_next[x + y * size] = 2.0 * value_current - value_previous
                                + coefficient
                                    * (-4.0 * value_current
                                        + value_left
                                        + value_right
                                        + value_top
                                        + value_bottom)
                                + damp;
                        }
                    }

                    let start = i * len_per_thread * size;
                    let end = (i + 1) * len_per_thread * size;

                    let my_start = size;
                    let my_end = len_per_thread * size + size;

                    // update shared variables
                    {
                        // new next to next
                        let mut next = space_next.lock().unwrap();
                        let mut next_space = next.space.clone();

                        let mut next_b = next_space.split_off(start);
                        let mut next_b = next_b.split_off(len_per_thread * size);
                        next_space.append(&mut my_next[my_start..my_end].to_vec());
                        next_space.append(&mut next_b);
                        //assert_eq!(next.space.len(), next_space.len());
                        next.space = next_space;

                        /*
                        // old current to current
                        let mut previous = space_previous.lock().unwrap();
                        let mut previous_space = previous.space.clone();

                        let mut previous_b = previous_space.split_off(start);
                        let mut previous_b = previous_b.split_off(len_per_thread * size);
                        previous_space.append(&mut my_previous[my_start..my_end].to_vec());
                        previous_space.append(&mut previous_b);
                        //assert_eq!(current.space.len(), current_space.len());
                        previous.space = previous_space;
                        */
                    }

                    tx_update.send(i).unwrap();
                }
            });
        }

        Ok(WaveSimulator {
            space_previous: space_previous,
            space_current: space_current,
            space_next: space_next,
            space_size: size,
            tx_update: tx_update_vec,
            tx_order: tx_order_vec,
        })
    }

    pub fn add_gauss(&mut self, x: f32, y: f32, sigma: f32, power: f32) {
        self.space_current
            .lock()
            .unwrap()
            .add_gauss(x, y, sigma, power);
        self.space_previous
            .lock()
            .unwrap()
            .add_gauss(x, y, sigma, power);
    }

    pub fn order(&self, order: Order) {
        self.tx_order.iter().enumerate().for_each(|(i, tx)| {
            tx.send(order.clone()).unwrap();
        });
    }
}

impl Iterator for WaveSimulator {
    type Item = Arc<Mutex<Space>>;
    fn next(&mut self) -> Option<Self::Item> {
        let (tx_r, rx_r) = mpsc::channel();

        self.tx_update.iter().enumerate().for_each(|(i, tx)| {
            tx.send((i, tx_r.clone())).unwrap();
        });

        let mut count_received = 0;

        for _ in rx_r {
            count_received += 1;
            if count_received >= NUM_THREADS {
                break;
            }
        }

        let space = self.space_current.lock().unwrap().space.clone();
        self.space_previous.lock().unwrap().space = space;

        let space = self.space_next.lock().unwrap().space.clone();
        self.space_current.lock().unwrap().space = space;

        let space = &self.space_next.lock().unwrap().space.clone();
        for (i, _) in space.as_slice().iter().enumerate() {
            self.space_next.lock().unwrap().space[i] = 0.0;
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
