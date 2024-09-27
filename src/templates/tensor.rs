use dam::context_tools::*;
use ndarray::{ArcArray, Array, IxDyn};

#[derive(Clone, Debug, Default)]
pub struct tensor {
    pub size: usize, // number of elements
    pub datatype: usize, // number of bytes per element
    pub name: String, // name of tensor
    pub array: Vec<element>, // actual data
}

#[derive(Clone, Debug, Default)]
pub struct element {
    pub data: usize,
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

