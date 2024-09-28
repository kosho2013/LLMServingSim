use dam::context_tools::*;
use ndarray::{ArcArray, Array, IxDyn};

#[derive(Clone, Debug, Default)]
pub struct tensor {
    pub size: usize, // number of KBs in the tensor
    pub name: String, // name of tensor
}

#[derive(Clone, Debug, Default)]
pub struct element { // each element represents one KB
    pub name: String,
}

impl DAMType for tensor {
    fn dam_size(&self) -> usize {
        1
    }
}

impl DAMType for element {
    fn dam_size(&self) -> usize {
        1
    }
}

