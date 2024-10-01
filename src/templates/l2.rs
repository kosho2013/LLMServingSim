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
    pub l1_to_l2_receiver_counter: usize,
    pub l2_to_l3_sender: Sender<tensor::element>,
    pub l2_to_l3_bw: usize,
    pub l2_to_l3_tensor: Vec<(String, usize)>, // number of elements, name of tensor
    pub l3_to_l2_receiver: Receiver<tensor::element>,
    pub l3_to_l2_receiver_counter: usize,
}

impl l2 {
    pub fn init(
        l2_initialize_tensor: Vec<(String, usize)>,
        l2_to_l1_sender: Sender<tensor::element>,
        l2_to_l1_bw: usize,
        l2_to_l1_tensor: Vec<(String, usize)>,
        l1_to_l2_receiver: Receiver<tensor::element>,
        l1_to_l2_receiver_counter: usize,
        l2_to_l3_sender: Sender<tensor::element>,
        l2_to_l3_bw: usize,
        l2_to_l3_tensor: Vec<(String, usize)>,
        l3_to_l2_receiver: Receiver<tensor::element>,
        l3_to_l2_receiver_counter: usize,
    ) -> Self {
        let l2 = l2 {
            l2_initialize_tensor,
            l2_to_l1_sender,
            l2_to_l1_bw,
            l2_to_l1_tensor,
            l1_to_l2_receiver,
            l1_to_l2_receiver_counter,
            l2_to_l3_sender,
            l2_to_l3_bw,
            l2_to_l3_tensor,
            l3_to_l2_receiver,
            l3_to_l2_receiver_counter,
            context_info: Default::default()
        };
        l2.l2_to_l1_sender.attach_sender(&l2);
        l2.l1_to_l2_receiver.attach_receiver(&l2);
        l2.l2_to_l3_sender.attach_sender(&l2);
        l2.l3_to_l2_receiver.attach_receiver(&l2);
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
        let mut l2_to_l1_send_element = vec![];
        let mut l2_to_l3_send_element = vec![];

        for (name, size) in &self.l2_to_l1_tensor
        {
            for _ in 0..*size
            {
                let element = tensor::element {
                    name: (*name.clone()).to_string(),
                };
                l2_to_l1_send_element.push(element);
            }
        }

        for (name, size) in &self.l2_to_l3_tensor
        {
            for _ in 0..*size
            {
                let element = tensor::element {
                    name: (*name.clone()).to_string(),
                };
                l2_to_l3_send_element.push(element);
            }
        }


        let mut l2_to_l1_counter = 0;
        let mut l2_to_l3_counter = 0;
        let mut l2_to_l1_flag = false;
        let mut l2_to_l3_flag = false;

        loop
        {
            for _ in 0..self.l2_to_l1_bw
            {
                if l2_to_l1_counter >= l2_to_l1_send_element.len()
                {
                    l2_to_l1_flag = true;
                    break;
                }
                let mut element = l2_to_l1_send_element[l2_to_l1_counter].clone();
                let _ = self.l2_to_l1_sender.enqueue(&self.time, ChannelElement { time: self.time.tick(), data: element });
                l2_to_l1_counter += 1;
            }

            for _ in 0..self.l2_to_l3_bw
            {
                if l2_to_l3_counter >= l2_to_l3_send_element.len()
                {
                    l2_to_l3_flag = true;
                    break;
                }
                let mut element = l2_to_l3_send_element[l2_to_l3_counter].clone();
                let _ = self.l2_to_l3_sender.enqueue(&self.time, ChannelElement { time: self.time.tick(), data: element });
                l2_to_l3_counter += 1;
            }

            if l2_to_l1_flag && l2_to_l3_flag
            {
                break;
            }

            self.time.incr_cycles(1);
        }

        




        // receive l1 to l2
        let mut counter = 0;
        loop
        {
            if counter == self.l1_to_l2_receiver_counter
            {
                break;
            }
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
                    counter += 1;
                },
                Err(_) =>
                {
                    panic!("Wrong!");
                }
            }
        }

        




        // receive l3 to l2
        let mut counter = 0;
        loop
        {
            if counter == self.l3_to_l2_receiver_counter
            {
                break;
            }
            let peek_result = self.l3_to_l2_receiver.peek_next(&self.time);
            match peek_result {
                Ok(_) =>
                {
                    let element = self.l3_to_l2_receiver.dequeue(&self.time).unwrap().data;
                    
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




        
        println!("l2 tensors {:?}", tensors);
        return;
    }
}