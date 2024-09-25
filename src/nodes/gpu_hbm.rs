use dam::context_tools::*;
use ndarray::Array;

use crate::types::tensor;

#[context_macro]
struct GPU_HBM {
    gpu_hbm_sender:Sender<tensor::tensor>,
    gpu_hbm_receiver:Receiver<tensor::tensor>,
}

impl GPU_HBM {
    pub fn init(sender:Sender<tensor::tensor>, receiver:Receiver<tensor::tensor>) -> Self {
        let hbm = GPU_HBM {
            gpu_hbm_sender: sender,
            gpu_hbm_receiver: receiver,
            context_info: Default::default()
        };
        hbm.gpu_hbm_sender.attach_sender(&hbm);
        hbm.gpu_hbm_receiver.attach_receiver(&hbm);
        hbm
    }
}

impl Context for GPU_HBM {
    fn run (&mut self)
    {
        return;
    }
}