extern crate protobuf;

pub mod templates;

use std::iter::FlatMap;
use std::mem;
use frunk::labelled::chars::L;
use rand::seq::SliceRandom; // Import the SliceRandom trait
use rand::Rng;
use templates::l1_compute::l1_compute;
use templates::l2::l2;
use templates::l1::l1;
use templates::l2_compute::l2_compute;
use templates::l3::l3;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::{cmp, env};
use std::{fs, time::Instant};
use std::fs::{read_to_string, File};
use std::io::{self, BufRead, BufReader};
use dam::channel::{ChannelElement, Receiver};
use dam::templates::datastore::Behavior;
use dam::templates::pmu::{PMUReadBundle, PMUWriteBundle, PMU};
use dam::types::StaticallySized;
use dam::utility_contexts::{ConsumerContext, FunctionContext, GeneratorContext, PrinterContext};
use prost::Message;
use dam::{logging::LogEvent, simulation::*};
use dam::{shim::RunMode, simulation::{DotConvertible, InitializationOptionsBuilder, ProgramBuilder, RunOptionsBuilder}};
use serde::{Deserialize, Serialize};


fn main()
{
    let invalid = 999999;

    let args: Vec<String> = env::args().collect();
    let log_path = format!("{}", &args[1]);
	let lines = read_to_string(log_path).unwrap();
    

    let mut l2_to_l1_bw: usize = invalid;
    let mut l1_to_l2_bw: usize = invalid;
    let mut l2_to_l3_bw: usize = invalid;
    let mut l3_to_l2_bw: usize = invalid;

    let mut l1_throughput: usize = invalid;
    let mut l2_throughput: usize = invalid;
    let mut l3_throughput: usize = invalid;

    let mut l1_initialize_tensor: Vec<(String, usize)> = vec![];
    let mut l2_initialize_tensor: Vec<(String, usize)> = vec![];
    let mut l3_initialize_tensor: Vec<(String, usize)> = vec![];

    let mut l3_to_l2_tensor: Vec<(String, usize)> = vec![];
    let mut l2_to_l3_tensor: Vec<(String, usize)> = vec![];
    let mut l2_to_l1_tensor: Vec<(String, usize)> = vec![];
    let mut l1_to_l2_tensor: Vec<(String, usize)> = vec![];

    let mut l1_kernel: Vec<usize> = vec![];
    let mut l2_kernel: Vec<usize> = vec![];
    let mut l3_kernel: Vec<usize> = vec![];

    let mut l3_to_l2_receiver_counter = 0;
    let mut l2_to_l3_receiver_counter = 0;
    let mut l2_to_l1_receiver_counter = 0;
    let mut l1_to_l2_receiver_counter = 0;

    for line in lines.lines()
    {
		if line.starts_with("bandwidth 2 1") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            l2_to_l1_bw = tmp[3].parse().unwrap();
		}

        if line.starts_with("bandwidth 1 2")
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            l1_to_l2_bw = tmp[3].parse().unwrap();
		}

        if line.starts_with("bandwidth 2 3") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            l2_to_l3_bw = tmp[3].parse().unwrap();
		}

        if line.starts_with("bandwidth 3 2") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            l3_to_l2_bw = tmp[3].parse().unwrap();
		}

        if line.starts_with("throughput 1") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            l1_throughput = tmp[2].parse().unwrap();
		}

        if line.starts_with("throughput 2") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            l2_throughput = tmp[2].parse().unwrap();
		}

        if line.starts_with("throughput 3") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            l3_throughput = tmp[2].parse().unwrap();
		}

        if line.starts_with("initialize 1") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[2].parse().unwrap();
            let bbb = tmp[3].parse().unwrap();
            l1_initialize_tensor.push((aaa, bbb));
		}

        if line.starts_with("initialize 2") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[2].parse().unwrap();
            let bbb = tmp[3].parse().unwrap();
            l2_initialize_tensor.push((aaa, bbb));
		}

        if line.starts_with("initialize 3") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[2].parse().unwrap();
            let bbb = tmp[3].parse().unwrap();
            l3_initialize_tensor.push((aaa, bbb));
		}

        if line.starts_with("memory 3 2") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[3].parse().unwrap();
            let bbb = tmp[4].parse().unwrap();
            l3_to_l2_tensor.push((aaa, bbb));
            l3_to_l2_receiver_counter += bbb;
		}

        if line.starts_with("memory 2 3") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[3].parse().unwrap();
            let bbb = tmp[4].parse().unwrap();
            l2_to_l3_tensor.push((aaa, bbb));
            l2_to_l3_receiver_counter += bbb;
		}

        if line.starts_with("memory 2 1") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[3].parse().unwrap();
            let bbb = tmp[4].parse().unwrap();
            l2_to_l1_tensor.push((aaa, bbb));
            l2_to_l1_receiver_counter += bbb;
		}

        if line.starts_with("memory 1 2") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[3].parse().unwrap();
            let bbb = tmp[4].parse().unwrap();
            l1_to_l2_tensor.push((aaa, bbb));
            l1_to_l2_receiver_counter += bbb;
		}

        if line.starts_with("compute 1") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[3].parse().unwrap();
            l1_kernel.push(aaa);
		}

        if line.starts_with("compute 2") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[3].parse().unwrap();
            l2_kernel.push(aaa);
		}

        if line.starts_with("compute 3") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[3].parse().unwrap();
            l3_kernel.push(aaa);
		}
    }
    


    // println!("l1_initialize_tensor{:?}", l1_initialize_tensor);
    // println!("l2_initialize_tensor{:?}", l2_initialize_tensor);
    // println!("l3_initialize_tensor{:?}", l3_initialize_tensor);



    println!("l1_kernel:{:?}", l1_kernel);
    println!("l1_throughput:{:?}", l1_throughput);

    let mut parent = ProgramBuilder::default();
    let (l2_to_l1_sender, l2_to_l1_receiver) = parent.unbounded();
    let (l1_to_l2_sender, l1_to_l2_receiver) = parent.unbounded();
    let (l3_to_l2_sender, l3_to_l2_receiver) = parent.unbounded();
    let (l2_to_l3_sender, l2_to_l3_receiver) = parent.unbounded();

    let l1 = l1::init(l1_initialize_tensor, l1_to_l2_sender, l1_to_l2_bw, l1_to_l2_tensor, l2_to_l1_receiver, l2_to_l1_receiver_counter);
    let l2 = l2::init(l2_initialize_tensor, l2_to_l1_sender, l2_to_l1_bw, l2_to_l1_tensor, l1_to_l2_receiver, l1_to_l2_receiver_counter, l2_to_l3_sender, l2_to_l3_bw, l2_to_l3_tensor, l3_to_l2_receiver, l3_to_l2_receiver_counter);
    let l3 = l3::init(l3_initialize_tensor, l3_to_l2_sender, l3_to_l2_bw, l3_to_l2_tensor, l2_to_l3_receiver, l2_to_l3_receiver_counter);
    let l1_compute = l1_compute::init(l1_kernel, l1_throughput);
    let l2_compute = l2_compute::init(l2_kernel, l2_throughput);

    parent.add_child(l1);
    parent.add_child(l2);
    parent.add_child(l3);
    parent.add_child(l1_compute);
    parent.add_child(l2_compute);

    // run DAM
    let initialized: dam::simulation::Initialized = parent
    .initialize(
        InitializationOptionsBuilder::default()
            .run_flavor_inference(false)
            .build()
            .unwrap(),
    )
    .unwrap();
    println!("{}", initialized.to_dot_string());

    let executed = initialized.run(
        RunOptionsBuilder::default()
            .mode(RunMode::Simple)
            .build()
            .unwrap(),
    );
    println!("Elapsed us: {:?}", executed.elapsed_cycles().unwrap());
    println!("Elapsed ms: {:?}", executed.elapsed_cycles().unwrap() as f32 / 1000 as f32);
    println!("Elapsed s: {:?}", executed.elapsed_cycles().unwrap() as f32 / 1e6 as f32);

}