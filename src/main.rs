extern crate protobuf;

pub mod templates;

use std::mem;
use rand::seq::SliceRandom; // Import the SliceRandom trait
use rand::Rng;
use templates::accelerator::accelerator;
use templates::sram::sram;
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

fn main()
{
    let level2_bw = 13107.2;
    let level3_bw = 1024;

    // define the program builder
    let mut parent = ProgramBuilder::default();
    let capacity = 1000;
    let (sender, receiver) = parent.bounded(capacity);

    // define the compute unit and the L1 cache
    let accelerator = accelerator::init(sender);
    let sram = sram::init(receiver);

    parent.add_child(accelerator);
    parent.add_child(sram);


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
    println!("Elapsed cycles: {:?}", executed.elapsed_cycles());


}