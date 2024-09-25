use dam::context_tools::*;
use ndarray::{ArcArray, Array, IxDyn};

#[derive(Clone, Debug, Default)]
pub struct tensor {
    pub kvcache_shape:Vec<usize>,
    pub kvcache_value:ArcArray<f32, IxDyn>,
}

impl DAMType for tensor {
    fn dam_size(&self) -> usize {
        self.kvcache_shape.iter().copied().reduce(|a, b| a * b).unwrap()
    }
}