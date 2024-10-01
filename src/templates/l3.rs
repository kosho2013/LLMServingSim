use std::collections::HashMap;

use dam::context_tools::*;
use ndarray::{ArcArray, Array};

use super::tensor;

#[context_macro]
pub struct l3 {
    pub l3_initialize_tensor: Vec<(String, usize)>, // number of elements, name of tensor
    pub l3_to_l2_sender: Sender<tensor::element>,
    pub l3_to_l2_bw: usize,
    pub l3_to_l2_tensor: Vec<(String, usize)>, // number of elements, name of tensor
    pub l2_to_l3_receiver: Receiver<tensor::element>,
    pub l2_to_l3_receiver_counter: usize,
}

impl l3 {
    pub fn init(
        l3_initialize_tensor: Vec<(String, usize)>,
        l3_to_l2_sender: Sender<tensor::element>,
        l3_to_l2_bw: usize,
        l3_to_l2_tensor: Vec<(String, usize)>,
        l2_to_l3_receiver: Receiver<tensor::element>,
        l2_to_l3_receiver_counter: usize,
    ) -> Self {
        let l3 = l3 {
            l3_initialize_tensor,
            l3_to_l2_sender,
            l3_to_l2_bw,
            l3_to_l2_tensor,
            l2_to_l3_receiver,
            l2_to_l3_receiver_counter,
            context_info: Default::default()
        };
        l3.l3_to_l2_sender.attach_sender(&l3);
        l3.l2_to_l3_receiver.attach_receiver(&l3);
        l3
    }
}

impl Context for l3 {
    fn run (&mut self)
    {
        // initialize tensors in l3
        let mut tensors = HashMap::new();
        for (name, size) in self.l3_initialize_tensor.clone()
        {
            tensors.insert(name, size);
        }

        println!("initial tensors in l3: {:?}", tensors);


        // send l3 to l2
        for (name, size) in &self.l3_to_l2_tensor
        {
            for j in 0..*size
            {
                if j % self.l3_to_l2_bw == 0
                {
                    let element = tensor::element {
                        name: (*name.clone()).to_string(),
                    };

                    let _ = self.l3_to_l2_sender.enqueue(&self.time, ChannelElement { time: self.time.tick(), data: element });
                    self.time.incr_cycles(1);
                    // tensors.insert(name.to_string(), tensors[&(*name)]-1);
                } else
                {
                    let element = tensor::element {
                        name: (*name.clone()).to_string(),
                    };
                    
                    let _ = self.l3_to_l2_sender.enqueue(&self.time, ChannelElement { time: self.time.tick(), data: element });
                    // tensors.insert(name.to_string(), tensors[&(*name)]-1);
                }
            }

            // if tensors[&(*name)] == 0
            // {
            //     tensors.remove(&(*name));
            // }
        }

        




        // receive l2 to l3
        let mut counter = 0;
        loop
        {
            if counter == self.l2_to_l3_receiver_counter
            {
                break;
            }
            let peek_result = self.l2_to_l3_receiver.peek_next(&self.time);
            match peek_result {
                Ok(_) =>
                {
                    let element = self.l2_to_l3_receiver.dequeue(&self.time).unwrap().data;
                    
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
        
        println!("l3 tensors {:?}", tensors);
        return;
    }
}