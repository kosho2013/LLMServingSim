use dam::context_tools::*;
use ndarray::{ArcArray, Array};

use super::tensor;

#[context_macro]
pub struct l2 {
    pub l2_sender: Sender<tensor::element>,
    pub l2_bw: usize,
    pub initial_tensor: Vec<(usize, usize, String)>, // number of elements, bytes per element, name of tensor
    pub send_to_l1_tensor: Vec<String>,
}

impl l2 {
    pub fn init(
        l2_sender: Sender<tensor::element>,
        l2_bw: usize,
        initial_tensor: Vec<(usize, usize, String)>,
        send_to_l1_tensor: Vec<String>,  
    ) -> Self {
        let l2 = l2 {
            l2_sender,
            l2_bw,
            initial_tensor,
            send_to_l1_tensor,
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
        let mut tensors = vec![];
        for (size, datatype, name) in self.initial_tensor.clone()
        {
            let mut array = vec![];
            for i in 0..size
            {
                let element = tensor::element{data: 0, name: name.clone()};
                array.push(element);
                println!("{}", i);
            }

            let tensor = tensor::tensor {
                size: size.clone(),
                datatype: datatype.clone(),
                name: name.clone(),
                array: array,
            };
            tensors.push(tensor);
        }

        println!("done eeeeeee");


        // println!("{:?}", tensors);

        // move tensors from L2 to L1
        for name in &self.send_to_l1_tensor
        {
            for i in 0..tensors.len()
            {
                if tensors[i].name == *name
                {
                    for j in 0..tensors[i].size
                    {
                        let num_ele_per_cycle = self.l2_bw / tensors[i].datatype;
                        if j % num_ele_per_cycle == 0
                        {
                            let _ = self.l2_sender.enqueue(&self.time, ChannelElement { time: self.time.tick()+1, data: tensors[i].array[j].clone() });
                            self.time.incr_cycles(1);
                        } else
                        {
                            let _ = self.l2_sender.enqueue(&self.time, ChannelElement { time: self.time.tick(), data: tensors[i].array[j].clone() });
                        }
                    } 
                }
            }
        }

        // remove the sent tensors
        for name in &self.send_to_l1_tensor
        {
            for i in 0..tensors.len()
            {
                if tensors[i].name == *name
                {
                    tensors.remove(i);
                }
            }
        }
        
        // println!("{:?}", tensors);

        return;
    }
}