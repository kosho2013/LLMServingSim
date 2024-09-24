use std::collections::HashMap;

use dam::simulation::ProgramBuilder;
use dam::{context_tools::*, RegisterALUOp};
use dam::templates::ops::ALUAddOp;
use frunk::labelled::chars::{K, M, T};
use half::f16;
use rand::Rng;
use super::primitive::ALUIdentityOp_1input;
use dam::{context_tools::*, dam_macros::context_macro};

use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

pub fn make_simd_pcu<A: dam::types::DAMType + num::Num> (
    dim: usize,
    pcu_receiver: Receiver<A>,
    pcu_sender: Sender<A>,
) -> impl Context { 
    let ingress_op = PCU::<A>::READ_ALL_INPUTS;
    let egress_op = PCU::<A>::WRITE_ALL_RESULTS;

    let mut pcu = PCU::new(
        PCUConfig {
            pipeline_depth: dim as usize,
            num_registers: 1 as usize,
        },
        ingress_op,
        egress_op,
    );

    let mut prev_register_ids: Vec<_> = vec![];
    for i in 0..1
    {
        prev_register_ids.push(i as usize);
    }

    let mut output_register_ids: Vec<_> = vec![];
    for i in 0..1
    {
        output_register_ids.push(i as usize);
    }

    pcu.push_stage(PipelineStage {
        op: ALUIdentityOp_1input(),
        forward: vec![],
        prev_register_ids: prev_register_ids,
        next_register_ids: vec![],
        output_register_ids: output_register_ids,
    });

    pcu.add_input_channel(pcu_receiver);
    pcu.add_output_channel(pcu_sender);

    pcu
}


pub fn make_systolic_pcu<A: dam::types::DAMType + num::Num> (
    dim: usize,
    pcu_receiver: Receiver<A>,
    pcu_sender: Sender<A>,
) -> impl Context { 
    let ingress_op = PCU::<A>::READ_ALL_INPUTS;
    let egress_op = PCU::<A>::WRITE_ALL_RESULTS;

    let mut pcu = PCU::new(
        PCUConfig {
            pipeline_depth: dim as usize,
            num_registers: 1 as usize,
        },
        ingress_op,
        egress_op,
    );

    let mut prev_register_ids: Vec<_> = vec![];
    for i in 0..1
    {
        prev_register_ids.push(i as usize);
    }

    let mut output_register_ids: Vec<_> = vec![];
    for i in 0..1
    {
        output_register_ids.push(i as usize);
    }

    for _ in 0..dim
    {
        pcu.push_stage(PipelineStage {
            op: ALUIdentityOp_1input(),
            forward: vec![],
            prev_register_ids: prev_register_ids.clone(),
            next_register_ids: vec![],
            output_register_ids: output_register_ids.clone(),
        });
    }
    

    pcu.add_input_channel(pcu_receiver);
    pcu.add_output_channel(pcu_sender);

    pcu
}


