use std::collections::HashMap;

use dam::context_tools::*;
use ndarray::Array;

use super::tensor;

#[context_macro]
pub struct l1_compute {
    pub l1_compute_flop: Vec<usize>,
    pub l1_compute_thru: usize,
}

impl l1_compute {
    pub fn init(
        l1_compute_flop: Vec<usize>,
        l1_compute_thru: usize,
    ) -> Self {
        let l1_compute = l1_compute {
            l1_compute_flop,
            l1_compute_thru,
            context_info: Default::default()
        };
        l1_compute
    }
}

impl Context for l1_compute {
    fn run (&mut self)
    {
        for i in 0..self.l1_compute_flop.len()
        {
            for j in 0..self.l1_compute_flop[i]
            {
                if j % self.l1_compute_thru == 0
                {
                    self.time.incr_cycles(1);
                }
            }
        }
        return;
    }
}