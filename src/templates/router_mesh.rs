use std::collections::HashMap;
use std::future;
use dam::channel::PeekResult;
use dam::context_tools::*;
use dam::structures::Time;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct router_mesh<A: Clone> {
    pub in_stream: Vec<Receiver<usize>>,
    pub in_dict: HashMap<String, (usize, usize)>,
    pub in_len: usize,
    pub out_stream: Vec<Sender<usize>>,
    pub out_dict: HashMap<String, (usize, usize)>,
    pub out_len: usize,
    pub x_dim: usize,
    pub y_dim: usize,
    pub x: usize,
    pub y: usize,
    pub num_input: usize,
    pub counter: usize,
    pub dummy: A,
}

impl<A: DAMType + num::Num> router_mesh<A>
where
router_mesh<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<usize>>,
        in_dict: HashMap<String, (usize, usize)>,
        in_len: usize,
        out_stream: Vec<Sender<usize>>,
        out_dict: HashMap<String, (usize, usize)>,
        out_len: usize,
        x_dim: usize,
        y_dim: usize,
        x: usize,
        y: usize,
        num_input: usize,
        counter: usize,
        dummy: A,
    ) -> Self {
        let router_mesh = router_mesh {
            in_stream,
            in_dict,
            in_len,
            out_stream,
            out_dict,
            out_len,
            x_dim,
            y_dim,
            x,
            y,
            num_input,
            counter,
            dummy,
            context_info: Default::default(),
        };
        for i in 0..in_len
        {
            let idx: usize = i.try_into().unwrap();
            router_mesh.in_stream[idx].attach_receiver(&router_mesh);
        }
        for i in 0..out_len
        {
            let idx: usize = i.try_into().unwrap();
            router_mesh.out_stream[idx].attach_sender(&router_mesh);
        }

        router_mesh
    }
}

impl<A: DAMType + num::Num> Context for router_mesh<A> {
    fn run(&mut self) {

        let invalid = usize::MAX;
        let num_ports = 5;

        let mut in_idx_vec = vec![]; // NSEWL
        let mut in_invalid = vec![]; // NSEWL
        let mut in_num_received_limit = vec![]; // NSEWL
        let mut in_num_recieved = [0, 0, 0, 0, 0]; // NSEWL

        for _ in 0..num_ports
        {
            in_idx_vec.push(invalid);
            in_invalid.push(false);
            in_num_received_limit.push(invalid);
        }

        if self.in_dict.contains_key("N_in")
        {
            in_idx_vec[0] = self.in_dict["N_in"].0;
            in_num_received_limit[0] = self.in_dict["N_in"].1;
        } else {
            in_invalid[0] = true;
        }
        if self.in_dict.contains_key("S_in")
        {
            in_idx_vec[1] = self.in_dict["S_in"].0;
            in_num_received_limit[1] = self.in_dict["S_in"].1;
        } else {
            in_invalid[1] = true;
        }
        if self.in_dict.contains_key("E_in")
        {
            in_idx_vec[2] = self.in_dict["E_in"].0;
            in_num_received_limit[2] = self.in_dict["E_in"].1;
        } else {
            in_invalid[2] = true;
        }
        if self.in_dict.contains_key("W_in")
        {
            in_idx_vec[3] = self.in_dict["W_in"].0;
            in_num_received_limit[3] = self.in_dict["W_in"].1;
        } else {
            in_invalid[3] = true;
        }
        if self.in_dict.contains_key("L_in")
        {
            in_idx_vec[4] = self.in_dict["L_in"].0;
            in_num_received_limit[4] = self.in_dict["L_in"].1;
        } else {
            in_invalid[4] = true;
        }




        let mut out_idx_vec = vec![]; // NSEWL
        let mut out_invalid = vec![]; // NSEWL
        let mut out_num_sent_limit = vec![]; // NSEWL
        let mut out_num_sent = [0, 0, 0, 0, 0]; // NSEWL

        for _ in 0..num_ports
        {
            out_idx_vec.push(invalid);
            out_invalid.push(false);
            out_num_sent_limit.push(invalid);
        }

        if self.out_dict.contains_key("N_out")
        {
            out_idx_vec[0] = self.out_dict["N_out"].0;
            out_num_sent_limit[0] = self.out_dict["N_out"].1;
        } else {
            out_invalid[0] = true;
        }
        if self.out_dict.contains_key("S_out")
        {
            out_idx_vec[1] = self.out_dict["S_out"].0;
            out_num_sent_limit[1] = self.out_dict["S_out"].1;
        } else {
            out_invalid[1] = true;
        }
        if self.out_dict.contains_key("E_out")
        {
            out_idx_vec[2] = self.out_dict["E_out"].0;
            out_num_sent_limit[2] = self.out_dict["E_out"].1;
        } else {
            out_invalid[2] = true;
        }
        if self.out_dict.contains_key("W_out")
        {
            out_idx_vec[3] = self.out_dict["W_out"].0;
            out_num_sent_limit[3] = self.out_dict["W_out"].1;
        } else {
            out_invalid[3] = true;
        }
        if self.out_dict.contains_key("L_out")
        {
            out_idx_vec[4] = self.out_dict["L_out"].0;
            out_num_sent_limit[4] = self.out_dict["L_out"].1;
        } else {
            out_invalid[4] = true;
        }


        
        

        // println!("x:{}, y:{}, in_idx_vec{:?}, in_num_received_limit{:?}, out_idx_vec{:?}, out_num_sent_limit{:?}   ", self.x, self.y, in_idx_vec, in_num_received_limit, out_idx_vec, out_num_sent_limit);




        let mut in_closed = vec![];
        for i in 0..num_ports
        {
            in_closed.push(in_invalid[i].clone());
        }


        let mut prev_lowest_idx = invalid;
        let mut prev_time = invalid;
        let mut prev_output_port = invalid;

        loop
        {
            // peek from all input ports
            let mut data_vec = vec![]; // NSEWL
            let mut dst_x_vec = vec![]; // NSEWL
            let mut dst_y_vec = vec![]; // NSEWL
            let mut future_time = vec![]; // NSEWL
            let mut future_time_tmp = vec![]; // NSEWL


            for i in 0..num_ports // NSEWL
            {
                if in_closed[i]
                {
                    future_time.push(invalid);
                    future_time_tmp.push(Time::new(invalid as u64));

                    data_vec.push(invalid);
                    dst_x_vec.push(invalid);
                    dst_y_vec.push(invalid);
                } else
                {
                    let peek_result = self.in_stream[in_idx_vec[i]].peek();
                    match peek_result {
                        PeekResult::Something(data) =>
                        {
                            future_time.push(data.time.time() as usize);
                            future_time_tmp.push(data.time);

                            let dst_x = data.data / self.y_dim;
                            let dst_y = data.data % self.y_dim;
                            data_vec.push(data.data);
                            dst_x_vec.push(dst_x);
                            dst_y_vec.push(dst_y);
                        },
                        PeekResult::Nothing(time) => 
                        {
                            future_time.push((time.time()+1) as usize);
                            future_time_tmp.push(Time::new(time.time()+1 as u64));

                            data_vec.push(invalid);
                            dst_x_vec.push(invalid);
                            dst_y_vec.push(invalid);
                        },
                        PeekResult::Closed =>
                        {
                            future_time.push(invalid);
                            future_time_tmp.push(Time::new(invalid as u64));

                            data_vec.push(invalid);
                            dst_x_vec.push(invalid);
                            dst_y_vec.push(invalid);

                            in_closed[i] = true;
                        },
                    }
                }
            }
            

            // deal with lowest time input
            let mut lowest_idx = invalid; // NSEWL
            let mut lowest_future_time = usize::MAX;
            for i in 0..num_ports
            {
                if future_time[i] != invalid
                {
                    if future_time[i] < lowest_future_time
                    {
                        lowest_idx = i;
                        lowest_future_time = future_time[i];
                    }
                }
            }


            

            if lowest_idx != invalid
            {
                if data_vec[lowest_idx] == invalid // Nothing
                {
                    self.time.advance(future_time_tmp[lowest_idx]);
                } else // Something
                {

                    let mut flag;


                    let data = self.in_stream[in_idx_vec[lowest_idx]].dequeue(&self.time).unwrap().data;
                    if data != data_vec[lowest_idx]
                    {
                        panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                    }
                    in_num_recieved[lowest_idx] += 1;

                    // send the data
                    if dst_x_vec[lowest_idx] == self.x && dst_y_vec[lowest_idx] == self.y // exit local port
                    {
                        if out_idx_vec[4] == invalid
                        {
                            panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                        } else {
                            self.out_stream[out_idx_vec[4]].enqueue(&self.time, ChannelElement::new(self.time.tick(), data_vec[lowest_idx].clone())).unwrap();
                            flag = false;
                            
                            out_num_sent[4] += 1;
                            prev_output_port = 4;
                        }

                    } else if dst_x_vec[lowest_idx] == self.x && dst_y_vec[lowest_idx] < self.y // exit W port
                    {
                        if out_idx_vec[3] == invalid
                        {
                            panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                        } else {
                            if (lowest_idx == num_ports-1) || (lowest_idx != prev_lowest_idx && future_time[lowest_idx] == prev_time && 3 != prev_output_port)
                            {
                                self.out_stream[out_idx_vec[3]].enqueue(&self.time, ChannelElement::new(self.time.tick(), data_vec[lowest_idx].clone())).unwrap();
                                flag = false;
                            } else
                            {
                                self.out_stream[out_idx_vec[3]].enqueue(&self.time, ChannelElement::new(self.time.tick()+1, data_vec[lowest_idx].clone())).unwrap();
                                flag = true;
                            }
                            out_num_sent[3] += 1;
                            prev_output_port = 3;
                        }

                    } else if dst_x_vec[lowest_idx] < self.x && dst_y_vec[lowest_idx] < self.y // exit N port
                    {
                        if out_idx_vec[0] == invalid
                        {
                            panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                        } else {
                            if (lowest_idx == num_ports-1) || (lowest_idx != prev_lowest_idx && future_time[lowest_idx] == prev_time && 0 != prev_output_port)
                            {
                                self.out_stream[out_idx_vec[0]].enqueue(&self.time, ChannelElement::new(self.time.tick(), data_vec[lowest_idx].clone())).unwrap();
                                flag = false;
                            } else
                            {
                                self.out_stream[out_idx_vec[0]].enqueue(&self.time, ChannelElement::new(self.time.tick()+1, data_vec[lowest_idx].clone())).unwrap();
                                flag = true;
                            }
                            out_num_sent[0] += 1;
                            prev_output_port = 0;
                        }

                    } else if dst_x_vec[lowest_idx] < self.x && dst_y_vec[lowest_idx] == self.y // exit N port
                    {
                        if out_idx_vec[0] == invalid
                        {
                            panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                        } else {
                            if (lowest_idx == num_ports-1) || (lowest_idx != prev_lowest_idx && future_time[lowest_idx] == prev_time && 0 != prev_output_port)
                            {
                                self.out_stream[out_idx_vec[0]].enqueue(&self.time, ChannelElement::new(self.time.tick(), data_vec[lowest_idx].clone())).unwrap();
                                flag = false;
                            } else
                            {
                                self.out_stream[out_idx_vec[0]].enqueue(&self.time, ChannelElement::new(self.time.tick()+1, data_vec[lowest_idx].clone())).unwrap();
                                flag = true;
                            }
                            out_num_sent[0] += 1;
                            prev_output_port = 0;
                        }

                    } else if dst_x_vec[lowest_idx] < self.x && dst_y_vec[lowest_idx] > self.y // exit N port
                    {
                        if out_idx_vec[0] == invalid
                        {
                            panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                        } else {
                            if (lowest_idx == num_ports-1) || (lowest_idx != prev_lowest_idx && future_time[lowest_idx] == prev_time && 0 != prev_output_port)
                            {
                                self.out_stream[out_idx_vec[0]].enqueue(&self.time, ChannelElement::new(self.time.tick(), data_vec[lowest_idx].clone())).unwrap();
                                flag = false;
                            } else
                            {
                                self.out_stream[out_idx_vec[0]].enqueue(&self.time, ChannelElement::new(self.time.tick()+1, data_vec[lowest_idx].clone())).unwrap();
                                flag = true;
                            }
                            out_num_sent[0] += 1;
                            prev_output_port = 0;
                        }

                    } else if dst_x_vec[lowest_idx] == self.x && dst_y_vec[lowest_idx] > self.y // exit E port
                    {
                        if out_idx_vec[2] == invalid
                        {
                            panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                        } else {
                            if (lowest_idx == num_ports-1) || (lowest_idx != prev_lowest_idx && future_time[lowest_idx] == prev_time && 2 != prev_output_port)
                            {
                                self.out_stream[out_idx_vec[2]].enqueue(&self.time, ChannelElement::new(self.time.tick(), data_vec[lowest_idx].clone())).unwrap();
                                flag = false;
                            } else
                            {
                                self.out_stream[out_idx_vec[2]].enqueue(&self.time, ChannelElement::new(self.time.tick()+1, data_vec[lowest_idx].clone())).unwrap();
                                flag = true;
                            }
                            out_num_sent[2] += 1;
                            prev_output_port = 2;
                        }

                    } else if dst_x_vec[lowest_idx] > self.x && dst_y_vec[lowest_idx] > self.y // exit S port
                    {
                        if out_idx_vec[1] == invalid
                        {
                            panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                        } else {
                            if (lowest_idx == num_ports-1) || (lowest_idx != prev_lowest_idx && future_time[lowest_idx] == prev_time && 1 != prev_output_port)
                            {
                                self.out_stream[out_idx_vec[1]].enqueue(&self.time, ChannelElement::new(self.time.tick(), data_vec[lowest_idx].clone())).unwrap();
                                flag = false;
                            } else
                            {
                                self.out_stream[out_idx_vec[1]].enqueue(&self.time, ChannelElement::new(self.time.tick()+1, data_vec[lowest_idx].clone())).unwrap();
                                flag = true;
                            }
                            out_num_sent[1] += 1;
                            prev_output_port = 1;
                        }

                    } else if dst_x_vec[lowest_idx] > self.x && dst_y_vec[lowest_idx] == self.y // exit S port
                    {
                        if out_idx_vec[1] == invalid
                        {
                            panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                        } else {
                            if (lowest_idx == num_ports-1) || (lowest_idx != prev_lowest_idx && future_time[lowest_idx] == prev_time && 1 != prev_output_port)
                            {
                                self.out_stream[out_idx_vec[1]].enqueue(&self.time, ChannelElement::new(self.time.tick(), data_vec[lowest_idx].clone())).unwrap();
                                flag = false;
                            } else
                            {
                                self.out_stream[out_idx_vec[1]].enqueue(&self.time, ChannelElement::new(self.time.tick()+1, data_vec[lowest_idx].clone())).unwrap();
                                flag = true;
                            }
                            out_num_sent[1] += 1;
                            prev_output_port = 1;
                        }

                    } else if dst_x_vec[lowest_idx] > self.x && dst_y_vec[lowest_idx] < self.y // exit S port
                    {
                        if out_idx_vec[1] == invalid
                        {
                            panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                        } else {
                            if (lowest_idx == num_ports-1) || (lowest_idx != prev_lowest_idx && future_time[lowest_idx] == prev_time && 1 != prev_output_port)
                            {
                                self.out_stream[out_idx_vec[1]].enqueue(&self.time, ChannelElement::new(self.time.tick(), data_vec[lowest_idx].clone())).unwrap();
                                flag = false;
                            } else
                            {
                                self.out_stream[out_idx_vec[1]].enqueue(&self.time, ChannelElement::new(self.time.tick()+1, data_vec[lowest_idx].clone())).unwrap();
                                flag = true;
                            }
                            out_num_sent[1] += 1;
                            prev_output_port = 1;
                        }
                        
                    } else
                    {
                        panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                    }

                   
                    if flag == true
                    {
                        self.time.incr_cycles(1);   
                    }
                    
                    prev_lowest_idx = lowest_idx;
                    prev_time = future_time[lowest_idx];
                }
            }


            // return when all input ports have received all packets and all output ports have sent all packets
            let mut cnt1 = 0;
            for i in 0..num_ports
            {
                if in_invalid[i]
                {
                    cnt1 += 1;
                } else {
                    if in_num_recieved[i] == in_num_received_limit[i] * self.num_input * self.counter
                    {
                        cnt1 += 1;
                    }
                }
            }

            let mut cnt2 = 0;
            for i in 0..num_ports
            {
                if out_invalid[i]
                {
                    cnt2 += 1;
                } else {
                    if out_num_sent[i] == out_num_sent_limit[i] * self.num_input * self.counter
                    {
                        cnt2 += 1;
                    }
                }
            }

            if cnt1 == num_ports && cnt2 == num_ports
            {
                println!("finished!!!!!!!!!!!!!!!!!!!!!!! {}, {}", self.x, self.y);
                return;
            }

        }




    }
}