use dam::context_tools::*;
use ndarray::Array;

use super::tensor;

#[context_macro]
pub struct l1 {
    pub l1_receiver:Receiver<tensor::element>,
}

impl l1 {
    pub fn init(
        receiver:Receiver<tensor::element>
    ) -> Self {
        let l1 = l1 {
            l1_receiver: receiver,
            context_info: Default::default()
        };
        l1.l1_receiver.attach_receiver(&l1);
        l1
    }
}

impl Context for l1 {
    fn run (&mut self)
    {
        // let mut tensors = vec![];

        loop
        {
            let peek_result = self.l1_receiver.peek_next(&self.time);

            match peek_result {
                Ok(channel_status) =>
                {
                    let element = self.l1_receiver.dequeue(&self.time).unwrap().data;
                    
                    // if tensors.len() == 0
                    // {

                    // }
                    // for i in 0..tensors.len()
                    // {

                    // }
                },
                Err(channel_status) =>
                {
                    return;
                }
            }
        }
        

    }
}