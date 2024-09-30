use std::collections::HashMap;

use dam::context_tools::*;
use ndarray::{ArcArray, Array};

use super::tensor;

#[context_macro]
pub struct l2 {
    pub l2_sender: Sender<tensor::element>,
    pub l2_to_l1_bw: usize,
    pub l2_initialize: Vec<(String, usize)>, // number of elements, name of tensor
    pub l2_to_l1: Vec<(String, usize)>,
}

impl l2 {
    pub fn init(
        l2_sender: Sender<tensor::element>,
        l2_to_l1_bw: usize,
        l2_initialize: Vec<(String, usize)>,
        l2_to_l1: Vec<(String, usize)>,  
    ) -> Self {
        let l2 = l2 {
            l2_sender,
            l2_to_l1_bw,
            l2_initialize,
            l2_to_l1,
            context_info: Default::default()
        };
        l2.l2_sender.attach_sender(&l2);
        l2
    }
}

impl Context for l2 {
    fn run (&mut self)
    {
        // initialize tensors in L2
        let mut tensors = HashMap::new();
        for (name, size) in self.l2_initialize.clone()
        {
            tensors.insert(name, size);
        }

        println!("{:?}", tensors);

        // move tensors from L2 to L1
        for (name, _) in &self.l2_to_l1
        {
            if tensors.contains_key(&(*name))
            {
                for j in 0..tensors[&(*name)]
                {
                    if j % self.l2_to_l1_bw == 0
                    {
                        let element = tensor::element {
                            name: (*name.clone()).to_string(),
                        };

                        let _ = self.l2_sender.enqueue(&self.time, ChannelElement { time: self.time.tick()+1, data: element });
                        self.time.incr_cycles(1);
                    } else
                    {
                        let element = tensor::element {
                            name: (*name.clone()).to_string(),
                        };
                        
                        let _ = self.l2_sender.enqueue(&self.time, ChannelElement { time: self.time.tick(), data: element });
                    }
                }
            }
        }

        // remove the sent tensors
        for (name, _) in &self.l2_to_l1
        {
            if tensors.contains_key(&(*name))
            {
                tensors.remove(&(*name));
            }
        }
        
        println!("{:?}", tensors);

        return;
    }
}