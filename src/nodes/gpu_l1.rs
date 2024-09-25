use dam::context_tools::*;
use ndarray::Array;

use crate::types::tensor;

#[context_macro]
pub struct GPU_L1 {
    // gpu_l1_sender:Sender<tensor::tensor>,
    pub gpu_l1_receiver:Receiver<tensor::tensor>,
}

impl GPU_L1 {
    pub fn init(receiver:Receiver<tensor::tensor>) -> Self {
        let l1 = GPU_L1 {
            gpu_l1_receiver: receiver,
            context_info: Default::default()
        };
        l1.gpu_l1_receiver.attach_receiver(&l1);
        l1
    }
}

impl Context for GPU_L1 {
    fn run (&mut self)
    {
        let peek_result = self.gpu_l1_receiver.peek_next(&self.time);

        match peek_result {
            Ok(channel_status) => {
                let _current_time = channel_status.time;
                let _kvcache_tensor = channel_status.data;
            },
            Err(_channel_status) => println!("Channel is closed")
        }

        let dequeue_result = self.gpu_l1_receiver.dequeue(&self.time);

        match dequeue_result {
            Ok(channel_status) => {
                let _current_time = channel_status.time;
                let _kvcache_tensor = channel_status.data;
            },
            Err(_channel_status) => println!("Channel is closed")
        }

        return;
    }
}