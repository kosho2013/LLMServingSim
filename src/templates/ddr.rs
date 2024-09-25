use dam::context_tools::*;
use ndarray::Array;

use super::tensor;

#[context_macro]
struct ddr {
    ddr_sender:Sender<tensor::tensor>,
    ddr_receiver:Receiver<tensor::tensor>,
}

impl ddr {
    pub fn init(sender:Sender<tensor::tensor>, receiver:Receiver<tensor::tensor>) -> Self {
        let ddr = ddr {
            ddr_sender: sender,
            ddr_receiver: receiver,
            context_info: Default::default()
        };
        ddr.ddr_sender.attach_sender(&ddr);
        ddr.ddr_receiver.attach_receiver(&ddr);
        ddr
    }
}

impl Context for ddr {
    fn run (&mut self)
    {
        return;
    }
}