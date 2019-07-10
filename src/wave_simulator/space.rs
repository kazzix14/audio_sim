use std::fmt::*;

#[derive(Clone, Debug)]
pub struct Space {
    pub space: Vec<f32>,
    pub space_spec: Vec<(f32, f32)>,
    pub size: usize,
}

impl Space {
    pub fn new(size: usize) -> Space {
        Space {
            space: vec![0.0; size * size],
            //propagation ratio, dumping ratio
            space_spec: vec![(0.2, 0.2); size * size],
            size: size,
        }
    }

    pub fn add_gauss(&mut self, cx: f32, cy: f32, sigma: f32, power: f32) {
        fn gauss(x: f32, sigma: f32) -> f32 {
            1.0 / (2.0 * std::f32::consts::PI).sqrt()
                * sigma
                * (-(x.powi(2) / (2.0 * sigma.powi(2)))).exp()
        }

        let size = self.size;

        for y in 0..size {
            for x in 0..size {
                let mut value = self.get(x, y);

                let norm = ((cx - x as f32).powi(2) + (cy - y as f32).powi(2)).sqrt();
                value += gauss(norm, sigma) * power;

                self.put(x, y, value);
            }
        }
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.space[x + y * self.size]
    }

    pub fn put(&mut self, x: usize, y: usize, value: f32) {
        self.space[x + y * self.size] = value;
    }
}

impl Display for Space {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let size = self.size;
        for y in 0..size {
            for x in 0..size {
                write!(f, "{} {} {}\n", x, y, self.get(x, y));
            }
        }
        Ok(())
    }
}
