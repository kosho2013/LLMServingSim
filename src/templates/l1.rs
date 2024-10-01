use std::collections::HashMap;

use dam::context_tools::*;
use ndarray::Array;

use super::tensor;

#[context_macro]
pub struct l1 {
    pub l1_initialize_tensor: Vec<(String, usize)>, // number of elements, name of tensor
    pub l1_to_l2_sender: Sender<tensor::element>,
    pub l1_to_l2_bw: usize,
    pub l1_to_l2_tensor: Vec<(String, usize)>, // number of elements, name of tensor
    pub l2_to_l1_receiver: Receiver<tensor::element>,
    pub l2_to_l1_receiver_counter: usize,
}

impl l1 {
    pub fn init(
        l1_initialize_tensor: Vec<(String, usize)>, // number of elements, name of tensor
        l1_to_l2_sender: Sender<tensor::element>,
        l1_to_l2_bw: usize,
        l1_to_l2_tensor: Vec<(String, usize)>, // number of elements, name of tensor
        l2_to_l1_receiver: Receiver<tensor::element>,
        l2_to_l1_receiver_counter: usize,
    ) -> Self {
        let l1 = l1 {
            l1_initialize_tensor,
            l1_to_l2_sender,
            l1_to_l2_bw,
            l1_to_l2_tensor,
            l2_to_l1_receiver,
            l2_to_l1_receiver_counter,
            context_info: Default::default()
        };
        l1.l1_to_l2_sender.attach_sender(&l1);
        l1.l2_to_l1_receiver.attach_receiver(&l1);
        l1
    }
}

impl Context for l1 {
    fn run (&mut self)
    {
        // initialize tensors in L1
        let mut tensors = HashMap::new();
        for (name, size) in self.l1_initialize_tensor.clone()
        {
            tensors.insert(name, size);
        }

        println!("initial tensors in l1: {:?}", tensors);


        // send L1 to L2
        for (name, size) in &self.l1_to_l2_tensor
        {
            for j in 0..*size
            {
                if j % self.l1_to_l2_bw == 0
                {
                    let element = tensor::element {
                        name: (*name.clone()).to_string(),
                    };

                    let _ = self.l1_to_l2_sender.enqueue(&self.time, ChannelElement { time: self.time.tick(), data: element });
                    self.time.incr_cycles(1);
                    // tensors.insert(name.to_string(), tensors[&(*name)]-1);
                } else
                {
                    let element = tensor::element {
                        name: (*name.clone()).to_string(),
                    };
                    
                    let _ = self.l1_to_l2_sender.enqueue(&self.time, ChannelElement { time: self.time.tick(), data: element });
                    // tensors.insert(name.to_string(), tensors[&(*name)]-1);
                }
            }

            // if tensors[&(*name)] == 0
            // {
            //     tensors.remove(&(*name));
            // }
        }




        
        let mut counter = 0;
        loop
        {
            if counter == self.l2_to_l1_receiver_counter
            {
                break;
            }
            let peek_result = self.l2_to_l1_receiver.peek_next(&self.time);
            match peek_result {
                Ok(_) =>
                {
                    let element = self.l2_to_l1_receiver.dequeue(&self.time).unwrap().data;
                    
                    if tensors.contains_key(&element.name.clone())
                    {
                        tensors.insert(element.name.clone(), tensors[&element.name.clone()]+1);
                    } else
                    {
                        tensors.insert(element.name.clone(), 1);
                    }
                    counter += 1;
                },
                Err(_) =>
                {
                    panic!("Wrong!");
                }
            }
        }
        
        println!("l1 tensors {:?}", tensors);
        return;
    }
}