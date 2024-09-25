use dam::context_tools::*;
use ndarray::Array;

use super::tensor;

#[context_macro]
pub struct sram {
    // sram_sender:Sender<tensor::tensor>,
    pub sram_receiver:Receiver<tensor::tensor>,
}

impl sram {
    pub fn init(receiver:Receiver<tensor::tensor>) -> Self {
        let l1 = sram {
            sram_receiver: receiver,
            context_info: Default::default()
        };
        l1.sram_receiver.attach_receiver(&l1);
        l1
    }
}

impl Context for sram {
    fn run (&mut self)
    {
        let peek_result = self.sram_receiver.peek_next(&self.time);

        match peek_result {
            Ok(channel_status) => {
                let _current_time = channel_status.time;
                let _kvcache_tensor = channel_status.data;
            },
            Err(_channel_status) => println!("Channel is closed")
        }

        let dequeue_result = self.sram_receiver.dequeue(&self.time);

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