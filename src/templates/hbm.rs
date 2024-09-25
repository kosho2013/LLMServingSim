use dam::context_tools::*;
use ndarray::Array;

use super::tensor;

#[context_macro]
struct hbm {
    hbm_sender:Sender<tensor::tensor>,
    hbm_receiver:Receiver<tensor::tensor>,
}

impl hbm {
    pub fn init(sender:Sender<tensor::tensor>, receiver:Receiver<tensor::tensor>) -> Self {
        let hbm = hbm {
            hbm_sender: sender,
            hbm_receiver: receiver,
            context_info: Default::default()
        };
        hbm.hbm_sender.attach_sender(&hbm);
        hbm.hbm_receiver.attach_receiver(&hbm);
        hbm
    }
}

impl Context for hbm {
    fn run (&mut self)
    {
        return;
    }
}