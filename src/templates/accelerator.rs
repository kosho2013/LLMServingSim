use dam::context_tools::*;
use ndarray::{ArcArray, Array};

use super::tensor;

#[context_macro]
pub struct accelerator {
    pub accelerator_sender:Sender<tensor::tensor>,
    // accelerator_receiver:Receiver<tensor::tensor>,
}

impl accelerator {
    pub fn init(sender:Sender<tensor::tensor>) -> Self {
        let w = accelerator {
            accelerator_sender: sender,
            context_info: Default::default()
        };
        w.accelerator_sender.attach_sender(&w);
        w
    }
}

impl Context for accelerator {
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

        let _ = self.accelerator_sender.enqueue(&self.time, 
                                           ChannelElement { time: self.time.tick(), data: my_kvcache_tensor });
    
        return;
    }
}