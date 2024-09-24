use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct to_pmu<A: Clone> {
    pub in_stream: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_stream_wr_addr: Sender<usize>,
    pub out_stream_wr_data: Sender<usize>,
    pub num_input: usize,
    pub counter: usize,
    pub dummy: A,
}

impl<A: DAMType> to_pmu<A>
where
to_pmu<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<usize>>,
        in_len: usize,
        out_stream_wr_addr: Sender<usize>,
        out_stream_wr_data: Sender<usize>,
        num_input: usize,
        counter: usize,
        dummy: A,
    ) -> Self {
        let to_pmu = to_pmu {
            in_stream,
            in_len,
            out_stream_wr_addr,
            out_stream_wr_data,
            num_input,
            counter,
            dummy,
            context_info: Default::default(),
        };
        for i in 0..in_len
        {
            let idx: usize = i.try_into().unwrap();
            to_pmu.in_stream[idx].attach_receiver(&to_pmu);
        }
        to_pmu.out_stream_wr_addr.attach_sender(&to_pmu);
        to_pmu.out_stream_wr_data.attach_sender(&to_pmu);

        to_pmu
    }
}

impl<A: DAMType + num::Num> Context for to_pmu<A> {
    fn run(&mut self) {
        for _ in 0..(self.counter * self.num_input)
        {
            for j in 0..self.in_len
            {
                let _ = self.in_stream[j].dequeue(&self.time);
            }

            self.out_stream_wr_addr.enqueue(&self.time, ChannelElement::new(self.time.tick(), 0)).unwrap();
            self.out_stream_wr_data.enqueue(&self.time, ChannelElement::new(self.time.tick(), 0)).unwrap();
        }
        self.time.incr_cycles(1);
    }
}








#[context_macro]
pub struct from_pmu<A: Clone> {
    pub in_stream: Receiver<usize>,
    pub out_stream: Vec<Sender<usize>>,
    pub out_len: usize,
    pub out_dst: Vec<usize>,
    pub num_input: usize,
    pub counter: usize,
    pub dummy: A,
}

impl<A: DAMType> from_pmu<A>
where
from_pmu<A>: Context,
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
        let from_pmu = from_pmu {
            in_stream,
            out_stream,
            out_len,
            out_dst,
            num_input,
            counter,
            dummy,
            context_info: Default::default(),
        };
        from_pmu.in_stream.attach_receiver(&from_pmu);
        for i in 0..out_len
        {
            let idx: usize = i.try_into().unwrap();
            from_pmu.out_stream[idx].attach_sender(&from_pmu);
        }

        from_pmu
    }
}

impl<A: DAMType + num::Num> Context for from_pmu<A> {
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