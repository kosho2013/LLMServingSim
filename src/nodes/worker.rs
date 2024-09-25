use dam::context_tools::*;
use ndarray::{ArcArray, Array};

use crate::types::tensor;

#[context_macro]
pub struct Worker {
    pub worker_sender:Sender<tensor::tensor>,
    // worker_receiver:Receiver<tensor::tensor>,
}

impl Worker {
    pub fn init(sender:Sender<tensor::tensor>) -> Self {
        let w = Worker {
            worker_sender: sender,
            context_info: Default::default()
        };
        w.worker_sender.attach_sender(&w);
        w
    }
}

impl Context for Worker {
    fn run (&mut self)
    {
        // Define the shape of the tensor
        let shape = vec![40, 1096, 100, 100];

        // Create a zero-filled tensor with the specified shape
        let kvcache_tensor = ArcArray::zeros(shape.clone());
        
        let my_kvcache_tensor = tensor::tensor {
            kvcache_shape: shape,
            kvcache_value: kvcache_tensor
        };

        let _ = self.worker_sender.enqueue(&self.time, 
                                           ChannelElement { time: self.time.tick(), data: my_kvcache_tensor });
    
        return;
    }
}