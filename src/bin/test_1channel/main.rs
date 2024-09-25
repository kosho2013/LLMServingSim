use dam::{shim::RunMode, simulation::{DotConvertible, InitializationOptionsBuilder, ProgramBuilder, RunOptionsBuilder}};
use LLMServingSim::nodes::{gpu_l1::GPU_L1, worker::Worker};

fn main() {
    // define the program builder
    let mut program_builder = ProgramBuilder::default();
    let capacity = 1000;
    let (cu_sender, cu_receiver) = program_builder.bounded(capacity);

    // define the compute unit and the L1 cache
    let compute_unit = Worker::init(cu_sender);
    let l1_cache: GPU_L1 = GPU_L1::init(cu_receiver);

    program_builder.add_child(compute_unit);
    program_builder.add_child(l1_cache);

    // start running
    // program_builder.initialize(Default::default()).unwrap().run(Default::default());

    // run DAM
    let initialized: dam::simulation::Initialized = program_builder
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