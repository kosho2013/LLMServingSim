use std::mem;

use dam::context_tools::*;
use dam::types::StaticallySized;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct to_simd_pcu<A: Clone> {
    pub in_stream: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_stream: Sender<usize>,
    pub num_input: usize,
    pub counter: usize,
    pub dummy: A,
}

impl<A: DAMType> to_simd_pcu<A>
where
to_simd_pcu<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<usize>>,
        in_len: usize,
        out_stream: Sender<usize>,
        num_input: usize,
        counter: usize,
        dummy: A,
    ) -> Self {
        let to_simd_pcu = to_simd_pcu {
            in_stream,
            in_len,
            out_stream,
            num_input,
            counter,
            dummy,
            context_info: Default::default(),
        };
        for i in 0..in_len
        {
            let idx: usize = i.try_into().unwrap();
            to_simd_pcu.in_stream[idx].attach_receiver(&to_simd_pcu);
        }
        to_simd_pcu.out_stream.attach_sender(&to_simd_pcu);

        to_simd_pcu
    }
}

impl<A: DAMType + num::Num> Context for to_simd_pcu<A> {
    fn run(&mut self)
    {
        for _ in 0..(self.counter * self.num_input)
        {
            for j in 0..self.in_len
            {
                let _ = self.in_stream[j].dequeue(&self.time);
            }

            self.out_stream.enqueue(&self.time, ChannelElement::new(self.time.tick(), 0)).unwrap();
        }
        self.time.incr_cycles(1);
    }
}






















#[context_macro]
pub struct from_simd_pcu<A: Clone> {
    pub in_stream: Receiver<usize>,
    pub out_stream: Vec<Sender<usize>>,
    pub out_len: usize,
    pub out_dst: Vec<usize>,
    pub num_input: usize,
    pub counter: usize,
    pub dummy: A,
}

impl<A: DAMType> from_simd_pcu<A>
where
from_simd_pcu<A>: Context,
{
    pub fn new(
        in_stream: Receiver<usize>,
        out_stream: Vec<Sender<usize>>,
        out_len: usize,
        out_dst: Vec<usize>,
        num_input: usize,
        counter: usize,
        dummy: A,
    ) -> Self {
        let from_simd_pcu = from_simd_pcu {
            in_stream,
            out_stream,
            out_len,
            out_dst,
            num_input,
            counter,
            dummy,
            context_info: Default::default(),
        };
        from_simd_pcu.in_stream.attach_receiver(&from_simd_pcu);
        for i in 0..out_len
        {
            let idx: usize = i.try_into().unwrap();
            from_simd_pcu.out_stream[idx].attach_sender(&from_simd_pcu);
        }

        from_simd_pcu
    }
}

impl<A: DAMType + num::Num> Context for from_simd_pcu<A> {
    fn run(&mut self)
    {
        for _ in 0..(self.counter * self.num_input)
        {
            let _ = self.in_stream.dequeue(&self.time).unwrap().data;

            for j in 0..self.out_len
            {
                self.out_stream[j].enqueue(&self.time, ChannelElement::new(self.time.tick(), self.out_dst[j])).unwrap();
            }
        }
        self.time.incr_cycles(1); 
    }
}














#[context_macro]
pub struct to_systolic_pcu<A: Clone> {
    pub in_stream: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_stream_lane: Sender<usize>,
    pub out_stream_stage: Sender<usize>,
    pub num_input: usize,
    pub counter: usize,
    pub lane_dim: usize,
    pub stage_dim: usize,
    pub dummy: A,
}

impl<A: DAMType> to_systolic_pcu<A>
where
to_systolic_pcu<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<usize>>,
        in_len: usize,
        out_stream_lane: Sender<usize>,
        out_stream_stage: Sender<usize>,
        num_input: usize,
        counter: usize,
        lane_dim: usize,
        stage_dim: usize,
        dummy: A,
    ) -> Self {
        let to_systolic_pcu = to_systolic_pcu {
            in_stream,
            in_len,
            out_stream_lane,
            out_stream_stage,
            num_input,
            counter,
            lane_dim,
            stage_dim,
            dummy,
            context_info: Default::default(),
        };
        for i in 0..in_len
        {
            let idx: usize = i.try_into().unwrap();
            to_systolic_pcu.in_stream[idx].attach_receiver(&to_systolic_pcu);
        }
        to_systolic_pcu.out_stream_lane.attach_sender(&to_systolic_pcu);
        to_systolic_pcu.out_stream_stage.attach_sender(&to_systolic_pcu);

        to_systolic_pcu
    }
}

impl<A: DAMType + num::Num> Context for to_systolic_pcu<A> {
    fn run(&mut self)
    {
        for _ in 0..(self.counter * self.num_input)
        {
            for j in 0..self.in_len
            {
                let _ = self.in_stream[j].dequeue(&self.time);
            }

            self.out_stream_lane.enqueue(&self.time, ChannelElement::new(self.time.tick(), 0)).unwrap();
            self.out_stream_stage.enqueue(&self.time, ChannelElement::new(self.time.tick(), 0)).unwrap();
        }
        self.time.incr_cycles(1);
    }
}








#[context_macro]
pub struct from_systolic_pcu<A: Clone> {
    pub in_stream_lane: Receiver<usize>,
    pub in_stream_stage: Receiver<usize>,
    pub out_stream: Vec<Sender<usize>>,
    pub out_len: usize,
    pub out_dst: Vec<usize>,
    pub num_input: usize,
    pub counter: usize,
    pub lane_dim: usize,
    pub stage_dim: usize,
    pub dummy: A,
}

impl<A: DAMType> from_systolic_pcu<A>
where
from_systolic_pcu<A>: Context,
{
    pub fn new(
        in_stream_lane: Receiver<usize>,
        in_stream_stage: Receiver<usize>,
        out_stream: Vec<Sender<usize>>,
        out_len: usize,
        out_dst: Vec<usize>,
        num_input: usize,
        counter: usize,
        lane_dim: usize,
        stage_dim: usize,
        dummy: A,
    ) -> Self {
        let from_systolic_pcu = from_systolic_pcu {
            in_stream_lane,
            in_stream_stage,
            out_stream,
            out_len,
            out_dst,
            num_input,
            counter,
            lane_dim,
            stage_dim,
            dummy,
            context_info: Default::default(),
        };
        from_systolic_pcu.in_stream_lane.attach_receiver(&from_systolic_pcu);
        from_systolic_pcu.in_stream_stage.attach_receiver(&from_systolic_pcu);
        for i in 0..out_len
        {
            let idx: usize = i.try_into().unwrap();
            from_systolic_pcu.out_stream[idx].attach_sender(&from_systolic_pcu);
        }

        from_systolic_pcu
    }
}

impl<A: DAMType + num::Num> Context for from_systolic_pcu<A> {
    fn run(&mut self)
    {
        for i in 0..(self.counter * self.num_input)
        {
            let _ = self.in_stream_lane.dequeue(&self.time);
            let _ = self.in_stream_stage.dequeue(&self.time);

            for j in 0..self.out_len
            {
                self.out_stream[j].enqueue(&self.time, ChannelElement::new(self.time.tick(), self.out_dst[j])).unwrap();
            }
        }
        self.time.incr_cycles(1); 
    }
}