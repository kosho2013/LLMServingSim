extern crate protobuf;

pub mod templates;

use std::iter::FlatMap;
use std::mem;
use frunk::labelled::chars::L;
use rand::seq::SliceRandom; // Import the SliceRandom trait
use rand::Rng;
use templates::l2::l2;
use templates::l1::l1;
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
    let log_path = format!("{}{}", &args[1], "/log.txt");
	let lines = read_to_string(log_path).unwrap();
    

    let mut l2_to_l1_bw: usize = invalid;
    let mut l1_to_l2_bw: usize = invalid;
    let mut l2_to_l3_bw: usize = invalid;
    let mut l3_to_l2_bw: usize = invalid;

    let mut l1_initialize_tensor: Vec<(String, usize)> = vec![];
    let mut l2_initialize_tensor: Vec<(String, usize)> = vec![];
    let mut l3_initialize_tensor: Vec<(String, usize)> = vec![];

    let mut l3_to_l2_tensor: Vec<(String, usize)> = vec![];
    let mut l2_to_l3_tensor: Vec<(String, usize)> = vec![];
    let mut l2_to_l1_tensor: Vec<(String, usize)> = vec![];
    let mut l1_to_l2_tensor: Vec<(String, usize)> = vec![];

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

        if line.starts_with("config 0 3 2") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[4].parse().unwrap();
            let bbb = tmp[5].parse().unwrap();
            l3_to_l2_tensor.push((aaa, bbb));
		}

        if line.starts_with("config 0 2 3") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[4].parse().unwrap();
            let bbb = tmp[5].parse().unwrap();
            l2_to_l3_tensor.push((aaa, bbb));
		}

        if line.starts_with("config 0 2 1") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[4].parse().unwrap();
            let bbb = tmp[5].parse().unwrap();
            l2_to_l1_tensor.push((aaa, bbb));
		}

        if line.starts_with("config 0 1 2") 
		{ 
			let tmp: Vec<&str> = line.split_whitespace().collect();
            let aaa = tmp[4].parse().unwrap();
            let bbb = tmp[5].parse().unwrap();
            l1_to_l2_tensor.push((aaa, bbb));
		}
    }
    


    // println!("{:?}", l2_to_l1_bw);
    // println!("{:?}", l2_initialize_tensor);
    // println!("{:?}", l2_to_l1_tensor);




    let mut parent = ProgramBuilder::default();
    let (l2_to_l1_sender, l2_to_l1_receiver) = parent.unbounded();
    let (l1_to_l2_sender, l1_to_l2_receiver) = parent.unbounded();

    let l1 = l1::init(l1_initialize_tensor, l1_to_l2_sender, l1_to_l2_bw, l1_to_l2_tensor, l2_to_l1_receiver);
    let l2 = l2::init(l2_initialize_tensor, l2_to_l1_sender, l2_to_l1_bw, l2_to_l1_tensor, l1_to_l2_receiver);
    

    parent.add_child(l2);
    parent.add_child(l1);


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


}