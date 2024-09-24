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
pub struct router_dragonfly<A: Clone> {
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

impl<A: DAMType + num::Num> router_dragonfly<A>
where
router_dragonfly<A>: Context,
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
        let router_dragonfly = router_dragonfly {
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
            router_dragonfly.in_stream[idx].attach_receiver(&router_dragonfly);
        }
        for i in 0..out_len
        {
            let idx: usize = i.try_into().unwrap();
            router_dragonfly.out_stream[idx].attach_sender(&router_dragonfly);
        }

        router_dragonfly
    }
}

impl<A: DAMType + num::Num> Context for router_dragonfly<A> {
    fn run(&mut self)
    {
        let radix_intra = self.x_dim-1;
        let radix_inter = self.y_dim-1;
        let num_ports = radix_intra + radix_inter + 1;




        let mut inter_link_dict = HashMap::new();
        for m in 0..self.x_dim
        {
            for n in 0..self.y_dim
            {
                let mut tmp = vec![];
                inter_link_dict.insert((m, n), tmp);
            }
        }

        let mut num_connection = vec![];
        let quotient = (self.y_dim-1) / self.x_dim;
        let remainer = (self.y_dim-1) - quotient * self.x_dim;
        for m in 0..self.x_dim
        {						
            num_connection.push(quotient);
        }
        for m in 0..remainer
        {
            num_connection[m] += 1;
        }
        

        for m in 0..self.y_dim
        {
            let dst_x = m % self.x_dim;
            let mut cnt = m+1;

            for n in 0..num_connection.len()
            {
                let mut aaa = 0;
                while aaa < num_connection[n]
                {
                    let dst_y = cnt % self.y_dim;
                    if inter_link_dict[&(n, m)].len() < num_connection[n] && inter_link_dict[&(dst_x, dst_y)].len() < num_connection[dst_x]
                    {
                        let mut tmp = inter_link_dict[&(n, m)].clone();
                        tmp.push((dst_x, dst_y));
                        inter_link_dict.insert((n, m), tmp);

                        let mut tmp = inter_link_dict[&(dst_x, dst_y)].clone();
                        tmp.push((n, m));
                        inter_link_dict.insert((dst_x, dst_y), tmp);

                        cnt += 1;
                    }
                    aaa += 1;
                }

            }

        }






        let invalid = usize::MAX;




        // intra_0, intra_1, ..., inter_0, inter_1, ..., L_in
        let mut in_idx_vec = vec![]; // num_ports
        let mut in_invalid = vec![]; // num_ports
        let mut in_num_received_limit = vec![]; // num_ports
        let mut in_num_recieved = vec![]; // num_ports
        for _ in 0..num_ports
        {
            in_num_recieved.push(0);
        }


        for _ in 0..num_ports
        {
            in_idx_vec.push(invalid);
            in_invalid.push(false);
            in_num_received_limit.push(invalid);
        }

        for i in 0..radix_intra
        {
            let name = &("intra_".to_owned()+&(i).to_string()+"_in");
            if self.in_dict.contains_key(name)
            {
                in_idx_vec[i] = self.in_dict[name].0;
                in_num_received_limit[i] = self.in_dict[name].1;
            } else {
                in_invalid[i] = true;
            }
        }

        for i in radix_intra..radix_intra+radix_inter
        {
            let name = &("inter_".to_owned()+&(i).to_string()+"_in");
            if self.in_dict.contains_key(name)
            {
                in_idx_vec[i] = self.in_dict[name].0;
                in_num_received_limit[i] = self.in_dict[name].1;
            } else {
                in_invalid[i] = true;
            }
        }

        if self.in_dict.contains_key("L_in")
        {
            in_idx_vec[num_ports-1] = self.in_dict["L_in"].0;
            in_num_received_limit[num_ports-1] = self.in_dict["L_in"].1;
        } else {
            in_invalid[num_ports-1] = true;
        }




        let mut out_idx_vec = vec![]; // num_ports
        let mut out_invalid = vec![]; // num_ports
        let mut out_num_sent_limit = vec![]; // num_ports
        let mut out_num_sent = vec![]; // num_ports
        for _ in 0..num_ports
        {
            out_num_sent.push(0);
        }



        for _ in 0..num_ports
        {
            out_idx_vec.push(invalid);
            out_invalid.push(false);
            out_num_sent_limit.push(invalid);
        }



        for i in 0..radix_intra
        {
            let name = &("intra_".to_owned()+&i.to_string()+"_out");
            if self.out_dict.contains_key(name)
            {
                out_idx_vec[i] = self.out_dict[name].0;
                out_num_sent_limit[i] = self.out_dict[name].1;
            } else {
                out_invalid[i] = true;
            }
        }

        for i in radix_intra..radix_intra+radix_inter
        {
            let name = &("inter_".to_owned()+&i.to_string()+"_out");
            if self.out_dict.contains_key(name)
            {
                out_idx_vec[i] = self.out_dict[name].0;
                out_num_sent_limit[i] = self.out_dict[name].1;
            } else {
                out_invalid[i] = true;
            }
        }



        if self.out_dict.contains_key("L_out")
        {
            out_idx_vec[num_ports-1] = self.out_dict["L_out"].0;
            out_num_sent_limit[num_ports-1] = self.out_dict["L_out"].1;
        } else {
            out_invalid[num_ports-1] = true;
        }


    


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
            let mut data_vec = vec![]; // num_ports
            let mut dst_x_vec = vec![]; // num_ports
            let mut dst_y_vec = vec![]; // num_ports
            let mut future_time = vec![]; // num_ports
            let mut future_time_tmp = vec![]; // num_ports


            for i in 0..num_ports // num_ports
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
            let mut lowest_idx = invalid;
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
                        if out_idx_vec[num_ports-1] == invalid
                        {
                            panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                        } else {
                            self.out_stream[out_idx_vec[num_ports-1]].enqueue(&self.time, ChannelElement::new(self.time.tick(), data_vec[lowest_idx].clone())).unwrap();
                            flag = false;

                            out_num_sent[num_ports-1] += 1;
                            prev_output_port = num_ports-1;
                        }

                    } else if dst_x_vec[lowest_idx] != self.x && dst_y_vec[lowest_idx] == self.y // third hop, intra
                    {
                        let mut idx = 0;
                        while (self.x+idx+1) % self.x_dim != dst_x_vec[lowest_idx]
                        {
                            idx += 1;
                        }

                        if out_idx_vec[idx] == invalid
                        {
                            panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                        } else {
                            if (lowest_idx == num_ports-1) || (lowest_idx != prev_lowest_idx && future_time[lowest_idx] == prev_time && idx != prev_output_port)
                            {
                                self.out_stream[out_idx_vec[idx]].enqueue(&self.time, ChannelElement::new(self.time.tick(), data_vec[lowest_idx].clone())).unwrap();
                                flag = false;
                            } else
                            {
                                self.out_stream[out_idx_vec[idx]].enqueue(&self.time, ChannelElement::new(self.time.tick()+1, data_vec[lowest_idx].clone())).unwrap();
                                flag = true;
                            }
                            out_num_sent[idx] += 1;
                            prev_output_port = idx;
                        }
                        
                    } else
                    {
                        let mut dst_x_tmp = invalid;
                        let mut curr_x_tmp = invalid;
                        let mut curr_out_port_idx = invalid;
                        for (key, value) in &inter_link_dict
                        {
                            if key.1 == self.y
                            {
                                for m in 0..value.len()
                                {
                                    if value[m].1 == dst_y_vec[lowest_idx]
                                    {
                                        curr_x_tmp = key.0;
                                        dst_x_tmp = value[m].0;
                                        curr_out_port_idx = m;
                                        break;
                                    }
                                }
                            }
                        }


                        if self.x == curr_x_tmp // second hop, inter
						{
                            if out_idx_vec[radix_intra+curr_out_port_idx] == invalid
                            {
                                panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                            } else {
                                if (lowest_idx == num_ports-1) || (lowest_idx != prev_lowest_idx && future_time[lowest_idx] == prev_time && radix_intra+curr_out_port_idx != prev_output_port)
                                {
                                    self.out_stream[out_idx_vec[radix_intra+curr_out_port_idx]].enqueue(&self.time, ChannelElement::new(self.time.tick(), data_vec[lowest_idx].clone())).unwrap();
                                    flag = false;
                                } else
                                {
                                    self.out_stream[out_idx_vec[radix_intra+curr_out_port_idx]].enqueue(&self.time, ChannelElement::new(self.time.tick()+1, data_vec[lowest_idx].clone())).unwrap();
                                    flag = true;
                                }
                                out_num_sent[radix_intra+curr_out_port_idx] += 1;
                                prev_output_port = radix_intra+curr_out_port_idx;
                            }

                        } else
                        {
                            let mut idx = 0;
                            while (self.x+idx+1) % self.x_dim != curr_x_tmp
                            {
                                idx += 1;
                            }

                            if out_idx_vec[idx] == invalid
                            {
                                panic!("x:{}, y:{}, Wrong!", self.x, self.y);
                            } else
                            {
                                if (lowest_idx == num_ports-1) || (lowest_idx != prev_lowest_idx && future_time[lowest_idx] == prev_time && idx != prev_output_port)
                                {
                                    self.out_stream[out_idx_vec[idx]].enqueue(&self.time, ChannelElement::new(self.time.tick(), data_vec[lowest_idx].clone())).unwrap();
                                    flag = false;
                                } else
                                {
                                    self.out_stream[out_idx_vec[idx]].enqueue(&self.time, ChannelElement::new(self.time.tick()+1, data_vec[lowest_idx].clone())).unwrap();
                                    flag = true;
                                }
                                out_num_sent[idx] += 1;
                                prev_output_port = idx;
                            }

                        }

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

