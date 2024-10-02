use std::collections::HashMap;

use dam::context_tools::*;
use ndarray::Array;

use super::tensor;

#[context_macro]
pub struct l2_compute {
    pub l2_compute_flop: Vec<usize>,
    pub l2_compute_thru: usize,
}

impl l2_compute {
    pub fn init(
        l2_compute_flop: Vec<usize>,
        l2_compute_thru: usize,
    ) -> Self {
        let l2_compute = l2_compute {
            l2_compute_flop,
            l2_compute_thru,
            context_info: Default::default()
        };
        l2_compute
    }
}

impl Context for l2_compute {
    fn run (&mut self)
    {
        for i in 0..self.l2_compute_flop.len()
        {
            for j in 0..self.l2_compute_flop[i]
            {
                if j % self.l2_compute_thru == 0
                {
                    self.time.incr_cycles(1);
                }
            }
        }
        return;
    }
}