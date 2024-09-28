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
    let mut parent = ProgramBuilder::default();
    let (sender, receiver) = parent.unbounded();



    
    let l2_bw = 1024; // KB/us
    let mut initial_tensor = vec![(12000000, "weight_expert1".to_owned()), // KB, name
                                                    (2000000, "kvcache_expert1".to_owned()),
                                                    (12000000, "weight_expert2".to_owned()),
                                                    (2000000, "kvcache_expert2".to_owned()),
                                                    (12000000, "weight_expert3".to_owned()),
                                                    (2000000, "kvcache_expert3".to_owned())];

    let mut send_to_l1_tensor = vec!["weight_expert1".to_owned(), "kvcache_expert1".to_owned()];





    let l2 = l2::init(sender, l2_bw, initial_tensor, send_to_l1_tensor);
    let l1 = l1::init(receiver);

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