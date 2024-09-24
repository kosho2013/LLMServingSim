use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct to_router<A: Clone> {
    pub in_stream: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_stream: Sender<usize>,
    pub num_input: usize,
    pub counter: usize,
    pub dummy: A,
}

impl<A: DAMType> to_router<A>
where
to_router<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<usize>>,
        in_len: usize,
        out_stream: Sender<usize>,
        num_input: usize,
        counter: usize,
        dummy: A,
    ) -> Self {
        let to_router = to_router {
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
            to_router.in_stream[idx].attach_receiver(&to_router);
        }
        to_router.out_stream.attach_sender(&to_router);

        to_router
    }
}

impl<A: DAMType + num::Num> Context for to_router<A> {
    fn run(&mut self)
    {
        for _ in 0..(self.num_input * self.counter)
        {
            for j in 0..self.in_len
            {
                let in_data = self.in_stream[j].dequeue(&self.time).unwrap().data;
                self.out_stream.enqueue(&self.time, ChannelElement::new(self.time.tick(), in_data.clone())).unwrap();
            }
        }
        self.time.incr_cycles(1);
    }
}








#[context_macro]
pub struct from_router<A: Clone> {
    pub in_stream: Receiver<usize>,
    pub out_stream: Vec<Sender<usize>>,
    pub out_len: usize,
    pub num_input: usize,
    pub counter: usize,
    pub dummy: A,
}

impl<A: DAMType> from_router<A>
where
from_router<A>: Context,
{
    pub fn new(
        in_stream: Receiver<usize>,
        out_stream: Vec<Sender<usize>>,
        out_len: usize,
        num_input: usize,
        counter: usize,
        dummy: A,
    ) -> Self {
        let from_router = from_router {
            in_stream,
            out_stream,
            out_len,
            num_input,
            counter,
            dummy,
            context_info: Default::default(),
        };
        from_router.in_stream.attach_receiver(&from_router);
        for i in 0..out_len
        {
            let idx: usize = i.try_into().unwrap();
            from_router.out_stream[idx].attach_sender(&from_router);
        }

        from_router
    }
}

impl<A: DAMType + num::Num> Context for from_router<A> {
    fn run(&mut self) {
        for _ in 0..(self.num_input * self.counter)
        {
            for j in 0..self.out_len
            {
                let in_data = self.in_stream.dequeue(&self.time).unwrap().data;
                self.out_stream[j].enqueue(&self.time, ChannelElement::new(self.time.tick(), in_data.clone())).unwrap();
            }
        }
        self.time.incr_cycles(1);
    }
}


