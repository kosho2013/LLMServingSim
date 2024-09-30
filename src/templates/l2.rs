use std::collections::HashMap;

use dam::context_tools::*;
use ndarray::{ArcArray, Array};

use super::tensor;

#[context_macro]
pub struct l2 {
    pub l2_initialize_tensor: Vec<(String, usize)>, // number of elements, name of tensor
    pub l2_to_l1_sender: Sender<tensor::element>,
    pub l2_to_l1_bw: usize,
    pub l2_to_l1_tensor: Vec<(String, usize)>, // number of elements, name of tensor
    pub l1_to_l2_receiver: Receiver<tensor::element>,
}

impl l2 {
    pub fn init(
        l2_initialize_tensor: Vec<(String, usize)>,
        l2_to_l1_sender: Sender<tensor::element>,
        l2_to_l1_bw: usize,
        l2_to_l1_tensor: Vec<(String, usize)>,
        l1_to_l2_receiver: Receiver<tensor::element>,
    ) -> Self {
        let l2 = l2 {
            l2_initialize_tensor,
            l2_to_l1_sender,
            l2_to_l1_bw,
            l2_to_l1_tensor,
            l1_to_l2_receiver,
            context_info: Default::default()
        };
        l2.l2_to_l1_sender.attach_sender(&l2);
        l2.l1_to_l2_receiver.attach_receiver(&l2);
        l2
    }
}

impl Context for l2 {
    fn run (&mut self)
    {
        // initialize tensors in L2
        let mut tensors = HashMap::new();
        for (name, size) in self.l2_initialize_tensor.clone()
        {
            tensors.insert(name, size);
        }

        println!("initial tensors in l2: {:?}", tensors);


        // send L2 to L1
        for (name, size) in &self.l2_to_l1_tensor
        {
            if tensors.contains_key(&(*name))
            {
                for j in 0..*size
                {
                    if j % self.l2_to_l1_bw == 0
                    {
                        let element = tensor::element {
                            name: (*name.clone()).to_string(),
                        };

                        let _ = self.l2_to_l1_sender.enqueue(&self.time, ChannelElement { time: self.time.tick()+1, data: element });
                        self.time.incr_cycles(1);
                        tensors.insert(name.to_string(), tensors[&(*name)]-1);
                    } else
                    {
                        let element = tensor::element {
                            name: (*name.clone()).to_string(),
                        };
                        
                        let _ = self.l2_to_l1_sender.enqueue(&self.time, ChannelElement { time: self.time.tick(), data: element });
                        tensors.insert(name.to_string(), tensors[&(*name)]-1);
                    }
                }

                if tensors[&(*name)] == 0
                {
                    tensors.remove(&(*name));
                }
            }
        }

        




        // receive l1 to l2
        loop
        {
            let peek_result = self.l1_to_l2_receiver.peek_next(&self.time);

            match peek_result {
                Ok(_) =>
                {
                    let element = self.l1_to_l2_receiver.dequeue(&self.time).unwrap().data;
                    
                    if tensors.contains_key(&element.name.clone())
                    {
                        tensors.insert(element.name.clone(), tensors[&element.name.clone()]+1);
                    } else
                    {
                        tensors.insert(element.name.clone(), 1);
                    }
                    println!("dddd");
                },
                Err(_) =>
                {
                    println!("l2 tensors {:?}", tensors);
                    return;
                }
            }
        }
        
        return;
    }
}