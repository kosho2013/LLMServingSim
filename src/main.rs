extern crate protobuf;

mod proto_driver;
pub mod templates;
pub mod utils;

use std::mem;
use rand::seq::SliceRandom; // Import the SliceRandom trait
use rand::Rng;
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
use proto_driver::proto_headers::setup::System;
use prost::Message;
use dam::{logging::LogEvent, simulation::*};
use templates::pcu::{make_simd_pcu, make_systolic_pcu};
use templates::pcu_adapter::{from_simd_pcu, to_simd_pcu, from_systolic_pcu, to_systolic_pcu};
use templates::pmu_adapter::{from_pmu, to_pmu};
use templates::router_adapter::{from_router, to_router};
use templates::router_mesh::router_mesh;
use templates::router_torus::router_torus;
use templates::router_dragonfly::router_dragonfly;


fn sum_elements(vec: Vec<f32>) -> f32 {
    vec.iter().sum()
}


fn mean(data: &Vec<usize>) -> f32 {
    let sum: usize = data.iter().sum();
    sum as f32 / data.len() as f32
}

fn variance(data: &Vec<usize>) -> f32 {
    let mean_val = mean(data);
    let squared_diffs: f32 = data.iter()
        .map(|&value| {
            let diff = value as f32 - mean_val;
            diff * diff
        })
        .sum();
    
    squared_diffs / data.len() as f32
}



fn main() {
	let invalid = 999999;
	let dummy = 1;

	let args: Vec<String> = env::args().collect();

    let system_file_path = format!("{}{}", &args[1], "/system.bin");
	let topology_on_chip = format!("{}", &args[2]).parse::<String>().unwrap();
	let topology_off_chip = format!("{}", &args[3]).parse::<String>().unwrap();
	let num_input_on_chip = format!("{}", &args[4]).parse::<usize>().unwrap();
	let num_input_off_chip = format!("{}", &args[5]).parse::<usize>().unwrap();

	// get system
	let system = {
        let file_contents = fs::read(system_file_path).unwrap();
        System::decode(file_contents.as_slice()).unwrap()
    };

	let accelerator = system.accelerator.unwrap();


	let optimize_pr = accelerator.optimize_pr as bool;
	let num_iterations = accelerator.num_iterations as usize;
	let num_swapped = accelerator.num_swapped as usize;


	let x_off_chip: usize = accelerator.x_off_chip as usize;
	let y_off_chip: usize = accelerator.y_off_chip as usize;

	let x_on_chip: usize = accelerator.x_on_chip as usize;
	let y_on_chip: usize = accelerator.y_on_chip as usize;

	let lane_dim: usize = accelerator.lane_dim as usize;
	let stage_dim: usize = accelerator.stage_dim as usize;
	let freq: f32 = accelerator.freq as f32;
	let word: usize = accelerator.word as usize;

	let sram_cap: usize = accelerator.sram_cap as usize;
	let net_bw: f32 = accelerator.net_bw as f32;
	let mut buffer_depth: usize = accelerator.buffer_depth as usize;

	if buffer_depth != 0
	{
		buffer_depth = buffer_depth / lane_dim / word;
	}

	let num_vec_per_pmu: usize = sram_cap / lane_dim / word;

	let mut Compute_Latency: Vec<f32> = vec![];
	let mut Memory_Latency: Vec<f32> = vec![];
	let mut Network_Latency: Vec<f32> = vec![];
	let mut num_tile: usize = 0;
	
	let log_path = format!("{}{}", &args[1], "/log.txt");
	let lines = read_to_string(log_path).unwrap();


	
	let mut dam_compute_time: Vec<f32> = vec![];
	let mut dam_network_time: Vec<f32> = vec![];


	for line in lines.lines() {
		if line.starts_with("num_tile") 
		{
			let tmp = line.split_whitespace().last().unwrap().parse().unwrap();
			let tmp2: f32 = tmp;
			let tmp3: usize = tmp2.round() as usize;
			num_tile = tmp3;
		}
	}

	

 

	for line in lines.lines() {
		if line.starts_with("Compute_Latency[") 
		{ 
			let tmp: f32 = line.split_whitespace().last().unwrap().parse().unwrap();
			let tmp2 = tmp / num_tile as f32;
			Compute_Latency.push(tmp2);
		}

		if line.starts_with("Memory_Latency[") 
		{ 
			let tmp: f32 = line.split_whitespace().last().unwrap().parse().unwrap();
			let tmp2 = tmp / num_tile as f32;
			Memory_Latency.push(tmp2);
		}

		if line.starts_with("Network_Latency[") 
		{
			let tmp: f32 = line.split_whitespace().last().unwrap().parse().unwrap();
			let tmp2 = tmp / num_tile as f32;
			Network_Latency.push(tmp2);
		}

		// if line.starts_with("Per_Config_II[") 
		// { 
		// 	let tmp: f32 = line.split_whitespace().last().unwrap().parse().unwrap();
		// 	let tmp2: f32 = tmp / num_tile as f32;
		// 	dfmodel_time.push(tmp2);
		// }
	}


	let num_config = Network_Latency.len();





	
	println!("Compute_Latency {:?}", Compute_Latency);
	println!("Memory_Latency {:?}", Memory_Latency);
	println!("Network_Latency {:?}", Network_Latency);
	println!("num_tile {}", num_tile);
	println!("num_config {}", num_config);


	























	

	// compute
	for i in 0..num_config
	{
		println!("------------------------------- config: {} -------------------------------------", i);


		let mut connection_first_type: Vec<String> = vec![];
		let mut connection_first_x: Vec<usize> = vec![];
		let mut connection_first_y: Vec<usize> = vec![];
		let mut connection_second_type: Vec<String> = vec![];
		let mut connection_second_x: Vec<usize> = vec![];
		let mut connection_second_y: Vec<usize> = vec![];



		for line in lines.lines() {
			let str = format!("connection on_chip_config {} ", i);
			if line.starts_with(&str)
			{
				let tmp: Vec<&str> = line.split_whitespace().collect();
				let tmp1 = tmp[3].parse().unwrap();
				let tmp2 = tmp[4].parse().unwrap();
				let tmp3 = tmp[5].parse().unwrap();
				let tmp4 = tmp[6].parse().unwrap();
				let tmp5 = tmp[7].parse().unwrap();
				let tmp6 = tmp[8].parse().unwrap();
				connection_first_type.push(tmp1);
				connection_first_x.push(tmp2);
				connection_first_y.push(tmp3);
				connection_second_type.push(tmp4);
				connection_second_x.push(tmp5);
				connection_second_y.push(tmp6);
			}	
		}



		



		let mut pcu_x: Vec<usize> = vec![];
		let mut pcu_y: Vec<usize> = vec![];
		let mut pcu_counter: Vec<usize> = vec![];
		let mut pcu_sender_vec: Vec<Vec<usize>> = vec![];
		let mut pcu_receiver_vec: Vec<Vec<usize>> = vec![];
		let mut pcu_SIMD_or_Systolic: Vec<&str> = vec![];		

		let mut pmu_x: Vec<usize> = vec![];
		let mut pmu_y: Vec<usize> = vec![];
		let mut pmu_counter: Vec<usize> = vec![];
		let mut pmu_sender_vec: Vec<Vec<usize>>  = vec![];
		let mut pmu_receiver_vec: Vec<Vec<usize>>  = vec![];

		let mut pcu_xy = HashSet::new();
		let mut pmu_xy = HashSet::new();

		let mut num_of_connections = 0;

		for line in lines.lines() {
			let str = format!("num_of_connections on_chip_config {} ", i);
			if line.starts_with(&str)
			{
				let tmp: Vec<&str> = line.split_whitespace().collect();
				let tmp1 = tmp[tmp.len() - 1].parse().unwrap();
				num_of_connections = tmp1;
			}	
		}

		for line in lines.lines() {
			let str = format!("pcu on_chip_config {} ", i);
			if line.starts_with(&str)
			{
				let tmp: Vec<&str> = line.split_whitespace().collect();

				pcu_x.push(tmp[4].parse().unwrap());
				pcu_y.push(tmp[5].parse().unwrap());
				pcu_counter.push(tmp[6].parse().unwrap());
				pcu_SIMD_or_Systolic.push(tmp[7]);

				let str = "receiver";
				let mut j = 9;
				
				let mut tmp_sender_vec: Vec<usize> = vec![];
				while tmp[j] != str 
				{
					tmp_sender_vec.push(tmp[j].parse().unwrap());
					j += 1;
				}
				pcu_sender_vec.push(tmp_sender_vec);
				
				let mut tmp_receiver_vec: Vec<usize> = vec![];
				for k in j+1..tmp.len()
				{
					tmp_receiver_vec.push(tmp[k].parse().unwrap());
				}
				pcu_receiver_vec.push(tmp_receiver_vec);


				pcu_xy.insert((tmp[4].parse().unwrap(), tmp[5].parse().unwrap()));

			}	
		}	
		


		for line in lines.lines() {
			let str = format!("pmu on_chip_config {} ", i);
			if line.starts_with(&str)
			{
				let tmp: Vec<&str> = line.split_whitespace().collect();

				pmu_x.push(tmp[4].parse().unwrap());
				pmu_y.push(tmp[5].parse().unwrap());
				pmu_counter.push(tmp[6].parse().unwrap());

				let str = "receiver";
				let mut j = 8;
				
				let mut tmp_sender_vec: Vec<usize> = vec![];
				while tmp[j] != str 
				{
					tmp_sender_vec.push(tmp[j].parse().unwrap());
					j += 1;
				}
				pmu_sender_vec.push(tmp_sender_vec);
				
				let mut tmp_receiver_vec: Vec<usize> = vec![];
				for k in j+1..tmp.len()
				{
					tmp_receiver_vec.push(tmp[k].parse().unwrap());
				}
				pmu_receiver_vec.push(tmp_receiver_vec);


				pmu_xy.insert((tmp[4].parse().unwrap(), tmp[5].parse().unwrap()));

			}	
		}


		if num_of_connections == 0 || topology_on_chip == "skip"
		{
			dam_compute_time.push(0.0);
			continue;
		}


		


		println!("pcu_x{:?}", pcu_x);
		println!("pcu_y{:?}", pcu_y);
		println!("pcu_xy{:?}", pcu_xy);
		println!("pcu_counter {:?}", pcu_counter);
		println!("pcu_sender_vec {:?}", pcu_sender_vec);
		println!("pcu_receiver_vec {:?}", pcu_receiver_vec);
		println!("pcu_SIMD_or_Systolic {:?}", pcu_SIMD_or_Systolic);


		println!("pmu_x{:?}", pmu_x);
		println!("pmu_y{:?}", pmu_y);
		println!("pmu_xy{:?}", pmu_xy);
		println!("pmu_counter {:?}", pmu_counter);
		println!("pmu_sender_vec {:?}", pmu_sender_vec);
		println!("pmu_receiver_vec {:?}", pmu_receiver_vec);

		println!("num_of_connections {}", num_of_connections);

		
		let pcu_max_counter = pcu_counter.iter().max().unwrap();
		let pmu_max_counter = pmu_counter.iter().max().unwrap();
		let max_counter = *cmp::max(pcu_max_counter, pmu_max_counter);
		
		println!("max_counter:{}", max_counter);


		

		println!("--------  compute, {} --------", topology_on_chip);		






		
		// for dragonfly
		let mut inter_link_dict = HashMap::new();
		for m in 0..x_on_chip
		{
			for n in 0..y_on_chip
			{
				let mut tmp = vec![];
				inter_link_dict.insert((m, n), tmp);
			}
		}

		let mut num_connection = vec![];
		let quotient = (y_on_chip-1) / x_on_chip;
		let remainer = (y_on_chip-1) - quotient * x_on_chip;
		for m in 0..x_on_chip
		{						
			num_connection.push(quotient);
		}
		for m in 0..remainer
		{
			num_connection[m] += 1;
		}

		println!("num_connection:{:?}", num_connection);


		for m in 0..y_on_chip
		{
			let dst_x = m % x_on_chip;
			let mut cnt = m+1;

			for n in 0..num_connection.len()
			{
				let mut aaa = 0;
				while aaa < num_connection[n]
				{
					let dst_y = cnt % y_on_chip;
					if inter_link_dict[&(n, m)].len() < num_connection[n] && inter_link_dict[&(dst_x, dst_y)].len() < num_connection[dst_x]
					{
						let mut tmp = inter_link_dict[&(n, m)].clone();
						tmp.push((dst_x, dst_y));
						inter_link_dict.insert((n, m), tmp);

						let mut tmp = inter_link_dict[&(dst_x, dst_y)].clone();
						tmp.push((n, m));
						inter_link_dict.insert((dst_x, dst_y), tmp);

						// println!("n:{}, m:{}, dst_x:{}, dst_y:{}, inter_link_dict:{:?}", n, m, dst_x, dst_y, inter_link_dict);
						
						cnt += 1;
					}
					aaa += 1;
				}

			}

		}


		let mut my_dict: Vec<_> = inter_link_dict.iter().collect();
		my_dict.sort_by(|a, b| a.1.cmp(b.1));

		println!("inter_link_dict:");
		for (key, value) in &my_dict
		{
			println!("{:?}: {:?}", key, value);
		}







		


		

		let mut used_link_map = HashMap::new();

		if topology_on_chip == "mesh"
		{
			// get which global NoC is used for routing
			for j in 0..connection_first_x.len()
			{
				let mut curr_x = connection_first_x[j];
				let mut curr_y = connection_first_y[j];
				let mut dst_x = connection_second_x[j];
				let mut dst_y = connection_second_y[j];

				let link = (curr_x, curr_y, "from_L".to_owned(), curr_x, curr_y, "from_L".to_owned());
				if used_link_map.contains_key(&link)
				{	
					let tmp = used_link_map[&link] + 1;
					used_link_map.insert(link, tmp);
				} else
				{ 
					used_link_map.insert(link, 1);
				}


				let link = (dst_x, dst_y, "to_L".to_owned(), dst_x, dst_y, "to_L".to_owned());
				if used_link_map.contains_key(&link)
				{	
					let tmp = used_link_map[&link] + 1;
					used_link_map.insert(link, tmp);
				} else
				{ 
					used_link_map.insert(link, 1);
				}


				while true
				{
					if dst_x == curr_x && dst_y == curr_y // exit local port
					{
						break;
					} else if dst_x == curr_x && dst_y < curr_y // exit W port
					{
						let link = (curr_x, curr_y, "W".to_owned(), curr_x, curr_y-1, "E".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_y -= 1;

					} else if dst_x < curr_x && dst_y < curr_y // exit N port
					{
						let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x -= 1;

					} else if dst_x < curr_x && dst_y == curr_y // exit N port
					{
						let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x -= 1;

					} else if dst_x < curr_x && dst_y > curr_y // exit N port
					{
						let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x -= 1;

					} else if dst_x == curr_x && dst_y > curr_y // exit E port
					{
						let link = (curr_x, curr_y, "E".to_owned(), curr_x, curr_y+1, "W".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_y += 1;

					} else if dst_x > curr_x && dst_y > curr_y // exit S port
					{
						let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x += 1;

					} else if dst_x > curr_x && dst_y == curr_y // exit S port
					{
						let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x += 1;

					} else if dst_x > curr_x && dst_y < curr_y // exit S port
					{
						let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x += 1;

					} else
					{
						panic!("Wrong!");
					}
				}
			}






			let mut max_channel_load = 0;
			for (key, value) in used_link_map.clone()
			{
				if key.2 != "from_L" && key.2 != "to_L" && max_channel_load < value
				{
					max_channel_load = value;
					
				}
			}
			println!("max_channel_load:{}", max_channel_load);
			






			if optimize_pr
			{
				let mut max_channel_load_history = vec![];
				max_channel_load_history.push(max_channel_load);
				let mut current_max_channel_load = max_channel_load;

				for iter in 0..num_iterations
				{

					// find several pairs of tiles to swap
					let vec: Vec<usize> = (0..(x_on_chip*y_on_chip-1)).collect();
					let mut rng = rand::thread_rng();
					let mut sampled_list: Vec<_> = vec.choose_multiple(&mut rng, (num_swapped*2)).cloned().collect();
					let mut swap_list = vec![];
					for k in 0..num_swapped
					{
						let mut first_x = sampled_list[k] / y_on_chip;
						let mut first_y = sampled_list[k] % y_on_chip;
						let mut second_x = sampled_list[2*num_swapped-1-k] / y_on_chip;
						let mut second_y = sampled_list[2*num_swapped-1-k] % y_on_chip;

						if (pcu_xy.contains(&(first_x, first_y)) && pcu_xy.contains(&(second_x, second_y))) || (pmu_xy.contains(&(first_x, first_y)) && pmu_xy.contains(&(second_x, second_y)))
						{
							swap_list.push((first_x, first_y, second_x, second_y));
						}
					}

					// swap valid pairs of tiles
					let mut connection_first_x_new = connection_first_x.clone();
					let mut connection_first_y_new = connection_first_y.clone();
					let mut connection_second_x_new = connection_second_x.clone();
					let mut connection_second_y_new = connection_second_y.clone();
					

					let mut pcu_counter_new = pcu_counter.clone();
					let mut pcu_SIMD_or_Systolic_new = pcu_SIMD_or_Systolic.clone();
					let mut pcu_sender_vec_new: Vec<Vec<usize>> = vec![];
					let mut pcu_receiver_vec_new: Vec<Vec<usize>> = vec![];

					let mut pmu_counter_new = pmu_counter.clone();
					let mut pmu_sender_vec_new: Vec<Vec<usize>> = vec![];
					let mut pmu_receiver_vec_new: Vec<Vec<usize>> = vec![];


					for (first_x, first_y, second_x, second_y) in swap_list
					{
						for j in 0..connection_first_x.len()
						{
							if (connection_first_x_new[j] == first_x && connection_first_y_new[j] == first_y)
							{
								connection_first_x_new[j] = second_x;
								connection_first_y_new[j] = second_y;
							} else if (connection_first_x_new[j] == second_x && connection_first_y_new[j] == second_y)
							{
								connection_first_x_new[j] = first_x;
								connection_first_y_new[j] = first_y;
							}

							if (connection_second_x_new[j] == first_x && connection_second_y_new[j] == first_y)
							{
								connection_second_x_new[j] = second_x;
								connection_second_y_new[j] = second_y;
							} else if (connection_second_x_new[j] == second_x && connection_second_y_new[j] == second_y)
							{
								connection_second_x_new[j] = first_x;
								connection_second_y_new[j] = first_y;
							}
						}

						
						if pcu_xy.contains(&(first_x, first_y)) && pcu_xy.contains(&(second_x, second_y)) // swap pcus
						{
							let mut idx_first = 0;
							let mut idx_second = 0;
							for j in 0..pcu_x.len()
							{
								if pcu_x[j] == first_x && pcu_y[j] == first_y
								{
									idx_first = j;
								} else if pcu_x[j] == second_x && pcu_y[j] == second_y
								{
									idx_second = j;
								}
							}

							pcu_counter_new.swap(idx_first, idx_second);
							pcu_SIMD_or_Systolic_new.swap(idx_first, idx_second);

						} else if pmu_xy.contains(&(first_x, first_y)) && pmu_xy.contains(&(second_x, second_y)) // swap pmus
						{
							let mut idx_first = 0;
							let mut idx_second = 0;
							for j in 0..pmu_x.len()
							{
								if pmu_x[j] == first_x && pmu_y[j] == first_y
								{
									idx_first = j;
								} else if pmu_x[j] == second_x && pmu_y[j] == second_y
								{
									idx_second = j;
								}
							}

							pmu_counter_new.swap(idx_first, idx_second);

						} else
						{
							panic!("Wrong!");
						}	
					}

					for j in 0..pcu_x.len()
					{
						let mut sender = vec![];
						let mut receiver = vec![];

						for k in 0..connection_first_x_new.len()
						{
							if pcu_x[j] == connection_first_x_new[k] && pcu_y[j] == connection_first_y_new[k]
							{
								sender.push(k);
							}

							if pcu_x[j] == connection_second_x_new[k] && pcu_y[j] == connection_second_y_new[k]
							{
								receiver.push(k);
							}
						}

						if sender.len() == 0
						{
							sender.push(999999);
						}

						if receiver.len() == 0
						{
							receiver.push(999999);
						}

						pcu_sender_vec_new.push(sender);
						pcu_receiver_vec_new.push(receiver);
					}


					for j in 0..pmu_x.len()
					{
						let mut sender = vec![];
						let mut receiver = vec![];

						for k in 0..connection_first_x_new.len()
						{
							if pmu_x[j] == connection_first_x_new[k] && pmu_y[j] == connection_first_y_new[k]
							{
								sender.push(k);
							}

							if pmu_x[j] == connection_second_x_new[k] && pmu_y[j] == connection_second_y_new[k]
							{
								receiver.push(k);
							}
						}

						if sender.len() == 0
						{
							sender.push(999999);
						}

						if receiver.len() == 0
						{
							receiver.push(999999);
						}

						pmu_sender_vec_new.push(sender);
						pmu_receiver_vec_new.push(receiver);
					}



					// calculate new channel load
					let mut used_link_map_new = HashMap::new();

					for j in 0..connection_first_x_new.len()
					{
						let mut curr_x = connection_first_x_new[j];
						let mut curr_y = connection_first_y_new[j];
						let mut dst_x = connection_second_x_new[j];
						let mut dst_y = connection_second_y_new[j];

						let link = (curr_x, curr_y, "from_L".to_owned(), curr_x, curr_y, "from_L".to_owned());
						if used_link_map_new.contains_key(&link)
						{	
							let tmp = used_link_map_new[&link] + 1;
							used_link_map_new.insert(link, tmp);
						} else
						{ 
							used_link_map_new.insert(link, 1);
						}


						let link = (dst_x, dst_y, "to_L".to_owned(), dst_x, dst_y, "to_L".to_owned());
						if used_link_map_new.contains_key(&link)
						{	
							let tmp = used_link_map_new[&link] + 1;
							used_link_map_new.insert(link, tmp);
						} else
						{ 
							used_link_map_new.insert(link, 1);
						}


						while true
						{
							if dst_x == curr_x && dst_y == curr_y // exit local port
							{
								break;
							} else if dst_x == curr_x && dst_y < curr_y // exit W port
							{
								let link = (curr_x, curr_y, "W".to_owned(), curr_x, curr_y-1, "E".to_owned());
								if used_link_map_new.contains_key(&link)
								{	
									let tmp = used_link_map_new[&link] + 1;
									used_link_map_new.insert(link, tmp);
								} else
								{ 
									used_link_map_new.insert(link, 1);
								}
								curr_y -= 1;

							} else if dst_x < curr_x && dst_y < curr_y // exit N port
							{
								let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
								if used_link_map_new.contains_key(&link)
								{	
									let tmp = used_link_map_new[&link] + 1;
									used_link_map_new.insert(link, tmp);
								} else
								{ 
									used_link_map_new.insert(link, 1);
								}
								curr_x -= 1;

							} else if dst_x < curr_x && dst_y == curr_y // exit N port
							{
								let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
								if used_link_map_new.contains_key(&link)
								{	
									let tmp = used_link_map_new[&link] + 1;
									used_link_map_new.insert(link, tmp);
								} else
								{ 
									used_link_map_new.insert(link, 1);
								}
								curr_x -= 1;

							} else if dst_x < curr_x && dst_y > curr_y // exit N port
							{
								let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
								if used_link_map_new.contains_key(&link)
								{	
									let tmp = used_link_map_new[&link] + 1;
									used_link_map_new.insert(link, tmp);
								} else
								{ 
									used_link_map_new.insert(link, 1);
								}
								curr_x -= 1;

							} else if dst_x == curr_x && dst_y > curr_y // exit E port
							{
								let link = (curr_x, curr_y, "E".to_owned(), curr_x, curr_y+1, "W".to_owned());
								if used_link_map_new.contains_key(&link)
								{	
									let tmp = used_link_map_new[&link] + 1;
									used_link_map_new.insert(link, tmp);
								} else
								{ 
									used_link_map_new.insert(link, 1);
								}
								curr_y += 1;

							} else if dst_x > curr_x && dst_y > curr_y // exit S port
							{
								let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
								if used_link_map_new.contains_key(&link)
								{	
									let tmp = used_link_map_new[&link] + 1;
									used_link_map_new.insert(link, tmp);
								} else
								{ 
									used_link_map_new.insert(link, 1);
								}
								curr_x += 1;

							} else if dst_x > curr_x && dst_y == curr_y // exit S port
							{
								let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
								if used_link_map_new.contains_key(&link)
								{	
									let tmp = used_link_map_new[&link] + 1;
									used_link_map_new.insert(link, tmp);
								} else
								{ 
									used_link_map_new.insert(link, 1);
								}
								curr_x += 1;

							} else if dst_x > curr_x && dst_y < curr_y // exit S port
							{
								let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
								if used_link_map_new.contains_key(&link)
								{	
									let tmp = used_link_map_new[&link] + 1;
									used_link_map_new.insert(link, tmp);
								} else
								{ 
									used_link_map_new.insert(link, 1);
								}
								curr_x += 1;

							} else
							{
								panic!("Wrong!");
							}
						}
					}



					let mut new_max_channel_load = 0;
					for (key, value) in used_link_map_new.clone()
					{
						if key.2 != "from_L" && key.2 != "to_L" && new_max_channel_load < value
						{
							new_max_channel_load = value;
						}
					}






					if new_max_channel_load < current_max_channel_load
					{
						current_max_channel_load = new_max_channel_load;

						connection_first_x = connection_first_x_new.clone();
						connection_first_y = connection_first_y_new.clone();
						connection_second_x = connection_second_x_new.clone();
						connection_second_y = connection_second_y_new.clone();


						pcu_counter = pcu_counter_new.clone();
						pcu_SIMD_or_Systolic = pcu_SIMD_or_Systolic_new.clone();
						pcu_sender_vec = pcu_sender_vec_new.clone();
						pcu_receiver_vec = pcu_receiver_vec_new.clone();

						pmu_counter = pmu_counter_new.clone();
						pmu_sender_vec = pmu_sender_vec_new.clone();
						pmu_receiver_vec = pmu_receiver_vec_new.clone();


						used_link_map = used_link_map_new.clone();
					}
					max_channel_load_history.push(current_max_channel_load);
					println!("iter:{}, current_max_channel_load:{}", iter, current_max_channel_load);
				}
				println!("max_channel_load_history:{:?}", max_channel_load_history);
			}








		} else if topology_on_chip == "torus"
		{

			for j in 0..connection_first_x.len()
			{
				let mut curr_x = connection_first_x[j];
				let mut curr_y = connection_first_y[j];
				let mut dst_x = connection_second_x[j];
				let mut dst_y = connection_second_y[j];

				let link = (curr_x, curr_y, "from_L".to_owned(), curr_x, curr_y, "from_L".to_owned());
				if used_link_map.contains_key(&link)
				{	
					let tmp = used_link_map[&link] + 1;
					used_link_map.insert(link, tmp);
				} else
				{ 
					used_link_map.insert(link, 1);
				}


				let link = (dst_x, dst_y, "to_L".to_owned(), dst_x, dst_y, "to_L".to_owned());
				if used_link_map.contains_key(&link)
				{	
					let tmp = used_link_map[&link] + 1;
					used_link_map.insert(link, tmp);
				} else
				{ 
					used_link_map.insert(link, 1);
				}


				while true
				{

					// println!("j:{}, curr_x:{}, curr_y:{}, dst_x:{}, dst_y:{}, used_link_map:{:?}", j, curr_x, curr_y, dst_x, dst_y, used_link_map);

					if dst_x == curr_x && dst_y == curr_y // exit local port
					{
						break;
					} else if dst_x == curr_x && dst_y < curr_y
					{
						let tmp1 = curr_y - dst_y;
						let tmp2 = y_on_chip - tmp1;


						if tmp1 < tmp2 // exit W port
						{
							let link = (curr_x, curr_y, "W".to_owned(), curr_x, curr_y-1, "E".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_y -= 1;
						} else // exit E port
						{
							let link = (curr_x, curr_y, "E".to_owned(), curr_x, (curr_y+1)%y_on_chip, "W".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_y = (curr_y+1)%y_on_chip;
						}
						

					} else if dst_x < curr_x && dst_y < curr_y
					{
						let tmp1 = curr_x - dst_x;
						let tmp2 = x_on_chip - tmp1;

						if tmp1 < tmp2 // exit N port
						{
							let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x -= 1;
						} else // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), (curr_x+1)%x_on_chip, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = (curr_x+1)%x_on_chip;
						}
						






					} else if dst_x < curr_x && dst_y == curr_y
					{
						let tmp1 = curr_x - dst_x;
						let tmp2 = x_on_chip - tmp1;

						if tmp1 < tmp2 // exit N port
						{
							let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x -= 1;
						} else // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), (curr_x+1)%x_on_chip, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = (curr_x+1)%x_on_chip;
						}
						


					} else if dst_x < curr_x && dst_y > curr_y // exit N port
					{
						let tmp1 = curr_x - dst_x;
						let tmp2 = x_on_chip - tmp1;

						if tmp1 < tmp2 // exit N port
						{
							let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x -= 1;
						} else // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), (curr_x+1)%x_on_chip, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = (curr_x+1)%x_on_chip;
						}
						





					} else if dst_x == curr_x && dst_y > curr_y
					{
						let tmp1 = dst_y - curr_y;
						let tmp2 = y_on_chip - tmp1;

						if tmp1 < tmp2 // exit E port
						{
							let link = (curr_x, curr_y, "E".to_owned(), curr_x, curr_y+1, "W".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_y += 1;
						} else // exit W port
						{
							let mut ttt;
							if curr_y == 0
							{
								ttt = y_on_chip-1;
							} else
							{
								ttt = curr_y-1;	
							}		 
							let link = (curr_x, curr_y, "W".to_owned(), curr_x, ttt, "E".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_y = ttt;
						}







					} else if dst_x > curr_x && dst_y > curr_y
					{
						let tmp1 = dst_x - curr_x;
						let tmp2 = x_on_chip - tmp1;

						if tmp1 < tmp2 // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x += 1;
						} else // exit N port
						{
							let mut ttt;
							if curr_x == 0
							{
								ttt = x_on_chip-1;
							} else
							{
								ttt = curr_x-1;	
							}	
							let link = (curr_x, curr_y, "N".to_owned(), ttt, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = ttt;
						}
						




					} else if dst_x > curr_x && dst_y == curr_y
					{
						let tmp1 = dst_x - curr_x;
						let tmp2 = x_on_chip - tmp1;

						if tmp1 < tmp2 // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x += 1;
						} else // exit N port
						{
							let mut ttt;
							if curr_x == 0
							{
								ttt = x_on_chip-1;
							} else
							{
								ttt = curr_x-1;	
							}	
							let link = (curr_x, curr_y, "N".to_owned(), ttt, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = ttt;
						}




					} else if dst_x > curr_x && dst_y < curr_y
					{
						let tmp1 = dst_x - curr_x;
						let tmp2 = x_on_chip - tmp1;

						if tmp1 < tmp2 // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x += 1;
						} else // exit N port
						{
							let mut ttt;
							if curr_x == 0
							{
								ttt = x_on_chip-1;
							} else
							{
								ttt = curr_x-1;	
							}	
							let link = (curr_x, curr_y, "N".to_owned(), ttt, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = ttt;
						}

					} else
					{
						panic!("Wrong!");
					}
				}
			}


			let mut max_channel_load = 0;
			for (key, value) in used_link_map.clone()
			{
				if key.2 != "from_L" && key.2 != "to_L" && max_channel_load < value
				{
					max_channel_load = value;
				}
			}

			println!("max_channel_load:{}", max_channel_load);






			if optimize_pr
			{
				let mut max_channel_load_history = vec![];
				max_channel_load_history.push(max_channel_load);
				let mut current_max_channel_load = max_channel_load;
				
				for iter in 0..num_iterations
				{

					// find several pairs of tiles to swap
					let vec: Vec<usize> = (0..(x_on_chip*y_on_chip-1)).collect();
					let mut rng = rand::thread_rng();
					let mut sampled_list: Vec<_> = vec.choose_multiple(&mut rng, (num_swapped*2)).cloned().collect();
					let mut swap_list = vec![];
					for k in 0..num_swapped
					{
						let mut first_x = sampled_list[k] / y_on_chip;
						let mut first_y = sampled_list[k] % y_on_chip;
						let mut second_x = sampled_list[2*num_swapped-1-k] / y_on_chip;
						let mut second_y = sampled_list[2*num_swapped-1-k] % y_on_chip;

						if (pcu_xy.contains(&(first_x, first_y)) && pcu_xy.contains(&(second_x, second_y))) || (pmu_xy.contains(&(first_x, first_y)) && pmu_xy.contains(&(second_x, second_y)))
						{
							swap_list.push((first_x, first_y, second_x, second_y));
						}
					}

					// swap valid pairs of tiles
					let mut connection_first_x_new = connection_first_x.clone();
					let mut connection_first_y_new = connection_first_y.clone();
					let mut connection_second_x_new = connection_second_x.clone();
					let mut connection_second_y_new = connection_second_y.clone();
					

					let mut pcu_counter_new = pcu_counter.clone();
					let mut pcu_SIMD_or_Systolic_new = pcu_SIMD_or_Systolic.clone();
					let mut pcu_sender_vec_new: Vec<Vec<usize>> = vec![];
					let mut pcu_receiver_vec_new: Vec<Vec<usize>> = vec![];

					let mut pmu_counter_new = pmu_counter.clone();
					let mut pmu_sender_vec_new: Vec<Vec<usize>> = vec![];
					let mut pmu_receiver_vec_new: Vec<Vec<usize>> = vec![];


					for (first_x, first_y, second_x, second_y) in swap_list
					{
						for j in 0..connection_first_x.len()
						{
							if (connection_first_x_new[j] == first_x && connection_first_y_new[j] == first_y)
							{
								connection_first_x_new[j] = second_x;
								connection_first_y_new[j] = second_y;
							} else if (connection_first_x_new[j] == second_x && connection_first_y_new[j] == second_y)
							{
								connection_first_x_new[j] = first_x;
								connection_first_y_new[j] = first_y;
							}

							if (connection_second_x_new[j] == first_x && connection_second_y_new[j] == first_y)
							{
								connection_second_x_new[j] = second_x;
								connection_second_y_new[j] = second_y;
							} else if (connection_second_x_new[j] == second_x && connection_second_y_new[j] == second_y)
							{
								connection_second_x_new[j] = first_x;
								connection_second_y_new[j] = first_y;
							}
						}

						
						if pcu_xy.contains(&(first_x, first_y)) && pcu_xy.contains(&(second_x, second_y)) // swap pcus
						{
							let mut idx_first = 0;
							let mut idx_second = 0;
							for j in 0..pcu_x.len()
							{
								if pcu_x[j] == first_x && pcu_y[j] == first_y
								{
									idx_first = j;
								} else if pcu_x[j] == second_x && pcu_y[j] == second_y
								{
									idx_second = j;
								}
							}

							pcu_counter_new.swap(idx_first, idx_second);
							pcu_SIMD_or_Systolic_new.swap(idx_first, idx_second);

						} else if pmu_xy.contains(&(first_x, first_y)) && pmu_xy.contains(&(second_x, second_y)) // swap pmus
						{
							let mut idx_first = 0;
							let mut idx_second = 0;
							for j in 0..pmu_x.len()
							{
								if pmu_x[j] == first_x && pmu_y[j] == first_y
								{
									idx_first = j;
								} else if pmu_x[j] == second_x && pmu_y[j] == second_y
								{
									idx_second = j;
								}
							}

							pmu_counter_new.swap(idx_first, idx_second);

						} else
						{
							panic!("Wrong!");
						}	
					}

					for j in 0..pcu_x.len()
					{
						let mut sender = vec![];
						let mut receiver = vec![];

						for k in 0..connection_first_x_new.len()
						{
							if pcu_x[j] == connection_first_x_new[k] && pcu_y[j] == connection_first_y_new[k]
							{
								sender.push(k);
							}

							if pcu_x[j] == connection_second_x_new[k] && pcu_y[j] == connection_second_y_new[k]
							{
								receiver.push(k);
							}
						}

						if sender.len() == 0
						{
							sender.push(999999);
						}

						if receiver.len() == 0
						{
							receiver.push(999999);
						}

						pcu_sender_vec_new.push(sender);
						pcu_receiver_vec_new.push(receiver);
					}





					for j in 0..pmu_x.len()
					{
						let mut sender = vec![];
						let mut receiver = vec![];

						for k in 0..connection_first_x_new.len()
						{
							if pmu_x[j] == connection_first_x_new[k] && pmu_y[j] == connection_first_y_new[k]
							{
								sender.push(k);
							}

							if pmu_x[j] == connection_second_x_new[k] && pmu_y[j] == connection_second_y_new[k]
							{
								receiver.push(k);
							}
						}

						if sender.len() == 0
						{
							sender.push(999999);
						}

						if receiver.len() == 0
						{
							receiver.push(999999);
						}

						pmu_sender_vec_new.push(sender);
						pmu_receiver_vec_new.push(receiver);
					}








					// calculate new channel load
					let mut used_link_map_new = HashMap::new();

					for j in 0..connection_first_x_new.len()
					{
						let mut curr_x = connection_first_x_new[j];
						let mut curr_y = connection_first_y_new[j];
						let mut dst_x = connection_second_x_new[j];
						let mut dst_y = connection_second_y_new[j];
		
						let link = (curr_x, curr_y, "from_L".to_owned(), curr_x, curr_y, "from_L".to_owned());
						if used_link_map_new.contains_key(&link)
						{	
							let tmp = used_link_map_new[&link] + 1;
							used_link_map_new.insert(link, tmp);
						} else
						{ 
							used_link_map_new.insert(link, 1);
						}
		
		
						let link = (dst_x, dst_y, "to_L".to_owned(), dst_x, dst_y, "to_L".to_owned());
						if used_link_map_new.contains_key(&link)
						{	
							let tmp = used_link_map_new[&link] + 1;
							used_link_map_new.insert(link, tmp);
						} else
						{ 
							used_link_map_new.insert(link, 1);
						}
		
		
						while true
						{
		
							// println!("j:{}, curr_x:{}, curr_y:{}, dst_x:{}, dst_y:{}, used_link_map_new:{:?}", j, curr_x, curr_y, dst_x, dst_y, used_link_map_new);
		
							if dst_x == curr_x && dst_y == curr_y // exit local port
							{
								break;
							} else if dst_x == curr_x && dst_y < curr_y
							{
								let tmp1 = curr_y - dst_y;
								let tmp2 = y_on_chip - tmp1;
		
		
								if tmp1 < tmp2 // exit W port
								{
									let link = (curr_x, curr_y, "W".to_owned(), curr_x, curr_y-1, "E".to_owned());
									if used_link_map_new.contains_key(&link)
									{	
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_y -= 1;
								} else // exit E port
								{
									let link = (curr_x, curr_y, "E".to_owned(), curr_x, (curr_y+1)%y_on_chip, "W".to_owned());
									if used_link_map_new.contains_key(&link)
									{	
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_y = (curr_y+1)%y_on_chip;
								}
								
		
							} else if dst_x < curr_x && dst_y < curr_y
							{
								let tmp1 = curr_x - dst_x;
								let tmp2 = x_on_chip - tmp1;
		
								if tmp1 < tmp2 // exit N port
								{
									let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
									if used_link_map_new.contains_key(&link)
									{	
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_x -= 1;
								} else // exit S port
								{
									let link = (curr_x, curr_y, "S".to_owned(), (curr_x+1)%x_on_chip, curr_y, "N".to_owned());
									if used_link_map_new.contains_key(&link)
									{
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_x = (curr_x+1)%x_on_chip;
								}
								
		
		
		
		
		
		
							} else if dst_x < curr_x && dst_y == curr_y
							{
								let tmp1 = curr_x - dst_x;
								let tmp2 = x_on_chip - tmp1;
		
								if tmp1 < tmp2 // exit N port
								{
									let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
									if used_link_map_new.contains_key(&link)
									{	
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_x -= 1;
								} else // exit S port
								{
									let link = (curr_x, curr_y, "S".to_owned(), (curr_x+1)%x_on_chip, curr_y, "N".to_owned());
									if used_link_map_new.contains_key(&link)
									{
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_x = (curr_x+1)%x_on_chip;
								}
								
		
		
							} else if dst_x < curr_x && dst_y > curr_y // exit N port
							{
								let tmp1 = curr_x - dst_x;
								let tmp2 = x_on_chip - tmp1;
		
								if tmp1 < tmp2 // exit N port
								{
									let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
									if used_link_map_new.contains_key(&link)
									{	
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_x -= 1;
								} else // exit S port
								{
									let link = (curr_x, curr_y, "S".to_owned(), (curr_x+1)%x_on_chip, curr_y, "N".to_owned());
									if used_link_map_new.contains_key(&link)
									{
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_x = (curr_x+1)%x_on_chip;
								}
								
		
		
		
		
		
							} else if dst_x == curr_x && dst_y > curr_y
							{
								let tmp1 = dst_y - curr_y;
								let tmp2 = y_on_chip - tmp1;
		
								if tmp1 < tmp2 // exit E port
								{
									let link = (curr_x, curr_y, "E".to_owned(), curr_x, curr_y+1, "W".to_owned());
									if used_link_map_new.contains_key(&link)
									{	
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_y += 1;
								} else // exit W port
								{
									let mut ttt;
									if curr_y == 0
									{
										ttt = y_on_chip-1;
									} else
									{
										ttt = curr_y-1;	
									}		 
									let link = (curr_x, curr_y, "W".to_owned(), curr_x, ttt, "E".to_owned());
									if used_link_map_new.contains_key(&link)
									{
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_y = ttt;
								}
		
		
		
		
		
		
		
							} else if dst_x > curr_x && dst_y > curr_y
							{
								let tmp1 = dst_x - curr_x;
								let tmp2 = x_on_chip - tmp1;
		
								if tmp1 < tmp2 // exit S port
								{
									let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
									if used_link_map_new.contains_key(&link)
									{	
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_x += 1;
								} else // exit N port
								{
									let mut ttt;
									if curr_x == 0
									{
										ttt = x_on_chip-1;
									} else
									{
										ttt = curr_x-1;	
									}	
									let link = (curr_x, curr_y, "N".to_owned(), ttt, curr_y, "S".to_owned());
									if used_link_map_new.contains_key(&link)
									{
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_x = ttt;
								}
								
		
		
		
		
							} else if dst_x > curr_x && dst_y == curr_y
							{
								let tmp1 = dst_x - curr_x;
								let tmp2 = x_on_chip - tmp1;
		
								if tmp1 < tmp2 // exit S port
								{
									let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
									if used_link_map_new.contains_key(&link)
									{	
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_x += 1;
								} else // exit N port
								{
									let mut ttt;
									if curr_x == 0
									{
										ttt = x_on_chip-1;
									} else
									{
										ttt = curr_x-1;	
									}	
									let link = (curr_x, curr_y, "N".to_owned(), ttt, curr_y, "S".to_owned());
									if used_link_map_new.contains_key(&link)
									{
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_x = ttt;
								}
		
		
		
		
							} else if dst_x > curr_x && dst_y < curr_y
							{
								let tmp1 = dst_x - curr_x;
								let tmp2 = x_on_chip - tmp1;
		
								if tmp1 < tmp2 // exit S port
								{
									let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
									if used_link_map_new.contains_key(&link)
									{	
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_x += 1;
								} else // exit N port
								{
									let mut ttt;
									if curr_x == 0
									{
										ttt = x_on_chip-1;
									} else
									{
										ttt = curr_x-1;	
									}	
									let link = (curr_x, curr_y, "N".to_owned(), ttt, curr_y, "S".to_owned());
									if used_link_map_new.contains_key(&link)
									{
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_x = ttt;
								}
		
							} else
							{
								panic!("Wrong!");
							}
						}
					}

					let mut new_max_channel_load = 0;
					for (key, value) in used_link_map_new.clone()
					{
						if key.2 != "from_L" && key.2 != "to_L" && new_max_channel_load < value
						{
							new_max_channel_load = value;
						}
					}






					if new_max_channel_load < current_max_channel_load
					{
						current_max_channel_load = new_max_channel_load;

						connection_first_x = connection_first_x_new.clone();
						connection_first_y = connection_first_y_new.clone();
						connection_second_x = connection_second_x_new.clone();
						connection_second_y = connection_second_y_new.clone();


						pcu_counter = pcu_counter_new.clone();
						pcu_SIMD_or_Systolic = pcu_SIMD_or_Systolic_new.clone();
						pcu_sender_vec = pcu_sender_vec_new.clone();
						pcu_receiver_vec = pcu_receiver_vec_new.clone();

						pmu_counter = pmu_counter_new.clone();
						pmu_sender_vec = pmu_sender_vec_new.clone();
						pmu_receiver_vec = pmu_receiver_vec_new.clone();


						used_link_map = used_link_map_new.clone();
					}
					max_channel_load_history.push(current_max_channel_load);
					println!("iter:{}, current_max_channel_load:{}", iter, current_max_channel_load);
				}
				println!("max_channel_load_history:{:?}", max_channel_load_history);
			}
			







		} else if topology_on_chip == "dragonfly"
		{
			let radix_intra = x_on_chip-1;
			let radix_inter = y_on_chip-1;








			for j in 0..connection_first_x.len()
			{
				let mut curr_x = connection_first_x[j];
				let mut curr_y = connection_first_y[j];
				let mut dst_x = connection_second_x[j];
				let mut dst_y = connection_second_y[j];

				let link = (curr_x, curr_y, "from_L".to_owned(), curr_x, curr_y, "from_L".to_owned());
				if used_link_map.contains_key(&link)
				{	
					let tmp = used_link_map[&link] + 1;
					used_link_map.insert(link, tmp);
				} else
				{ 
					used_link_map.insert(link, 1);
				}


				let link = (dst_x, dst_y, "to_L".to_owned(), dst_x, dst_y, "to_L".to_owned());
				if used_link_map.contains_key(&link)
				{	
					let tmp = used_link_map[&link] + 1;
					used_link_map.insert(link, tmp);
				} else
				{ 
					used_link_map.insert(link, 1);
				}


				while true
				{
					// println!("j:{}, curr_x:{}, curr_y:{}, dst_x:{}, dst_y:{}, used_link_map:{:?}", j, curr_x, curr_y, dst_x, dst_y, used_link_map);

					if dst_x == curr_x && dst_y == curr_y // exit local port
					{
						break;
					} else if dst_x != curr_x && dst_y == curr_y // third hop, (curr_x, curr_y) -> (dst_x, curr_y)
					{
						let mut idx1 = 0;
						while (curr_x+idx1+1) % x_on_chip != dst_x
						{
							idx1 += 1;
						}

						let mut idx2 = 0;
						while (dst_x+idx2+1) % x_on_chip != curr_x
						{
							idx2 += 1;
						}

						let link = (curr_x, curr_y, "intra_".to_owned()+&(idx1).to_string(), dst_x, dst_y, "intra_".to_owned()+&(idx2).to_string());
						if used_link_map.contains_key(&link)
						{
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x = dst_x;
							
					} else
					{

						
						
						let mut dst_x_tmp = invalid;
						let mut curr_x_tmp = invalid;
						let mut curr_out_port_idx = invalid;
						for (key, value) in &inter_link_dict
						{
							if key.1 == curr_y
							{
								for m in 0..value.len()
								{
									if value[m].1 == dst_y
									{
										curr_x_tmp = key.0;
										dst_x_tmp = value[m].0;
										curr_out_port_idx = m;
										break;
									}
								}
							}
						}

						let mut dst_in_port_idx = invalid;
						for (key, value) in &inter_link_dict
						{
							if key.1 == dst_y
							{
								for m in 0..value.len()
								{
									if value[m].1 == curr_y
									{
										dst_in_port_idx = m;
										break;
									}
								}
							}
						}



						



						if curr_x == curr_x_tmp // second hop, (curr_x, curr_y) -> (dst_x_tmp, dst_y)
						{
							let link = (curr_x, curr_y, "inter_".to_owned()+&(radix_intra + curr_out_port_idx).to_string(), dst_x_tmp, dst_y, "inter_".to_owned()+&(radix_intra + dst_in_port_idx).to_string());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}



							// if curr_y == 0 && dst_y == 2
							// {
							// 	let xxx = (curr_x, curr_y, "inter_".to_owned()+&(radix_intra + curr_out_port_idx).to_string(), curr_x, dst_y, "inter_".to_owned()+&(radix_intra + dst_in_port_idx).to_string());
							// 	println!("xxxxx curr_x:{}, curr_y:{}, dst_x:{}, dst_y:{}, curr_x_tmp:{}, xxx:{:?}", curr_x, curr_y, dst_x, dst_y, curr_x_tmp, xxx);
							// }

							curr_x = dst_x_tmp;
							curr_y = dst_y

						} else // first hop, (curr_x, curr_y) -> (curr_x_tmp, curr_y)
						{
							let mut idx1 = 0;
							while (curr_x+idx1+1) % x_on_chip != curr_x_tmp
							{
								idx1 += 1;
							}

							let mut idx2 = 0;
							while (curr_x_tmp+idx2+1) % x_on_chip != curr_x
							{
								idx2 += 1;
							}

							let link = (curr_x, curr_y, "intra_".to_owned()+&(idx1).to_string(), curr_x_tmp, curr_y, "intra_".to_owned()+&(idx2).to_string());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = curr_x_tmp;
						}

					}
				}
			}






			let mut max_channel_load = 0;
			for (key, value) in used_link_map.clone()
			{
				if key.2 != "from_L" && key.2 != "to_L" && max_channel_load < value
				{
					max_channel_load = value;
				}
			}

			println!("max_channel_load:{}", max_channel_load);




			if optimize_pr
			{
				let mut max_channel_load_history = vec![];
				max_channel_load_history.push(max_channel_load);
				let mut current_max_channel_load = max_channel_load;
				
				for iter in 0..num_iterations
				{

					// find several pairs of tiles to swap
					let vec: Vec<usize> = (0..(x_on_chip*y_on_chip-1)).collect();
					let mut rng = rand::thread_rng();
					let mut sampled_list: Vec<_> = vec.choose_multiple(&mut rng, (num_swapped*2)).cloned().collect();
					let mut swap_list = vec![];
					for k in 0..num_swapped
					{
						let mut first_x = sampled_list[k] / y_on_chip;
						let mut first_y = sampled_list[k] % y_on_chip;
						let mut second_x = sampled_list[2*num_swapped-1-k] / y_on_chip;
						let mut second_y = sampled_list[2*num_swapped-1-k] % y_on_chip;

						if (pcu_xy.contains(&(first_x, first_y)) && pcu_xy.contains(&(second_x, second_y))) || (pmu_xy.contains(&(first_x, first_y)) && pmu_xy.contains(&(second_x, second_y)))
						{
							swap_list.push((first_x, first_y, second_x, second_y));
						}
					}

					// swap valid pairs of tiles
					let mut connection_first_x_new = connection_first_x.clone();
					let mut connection_first_y_new = connection_first_y.clone();
					let mut connection_second_x_new = connection_second_x.clone();
					let mut connection_second_y_new = connection_second_y.clone();
					

					let mut pcu_counter_new = pcu_counter.clone();
					let mut pcu_SIMD_or_Systolic_new = pcu_SIMD_or_Systolic.clone();
					let mut pcu_sender_vec_new: Vec<Vec<usize>> = vec![];
					let mut pcu_receiver_vec_new: Vec<Vec<usize>> = vec![];

					let mut pmu_counter_new = pmu_counter.clone();
					let mut pmu_sender_vec_new: Vec<Vec<usize>> = vec![];
					let mut pmu_receiver_vec_new: Vec<Vec<usize>> = vec![];


					for (first_x, first_y, second_x, second_y) in swap_list
					{
						for j in 0..connection_first_x.len()
						{
							if (connection_first_x_new[j] == first_x && connection_first_y_new[j] == first_y)
							{
								connection_first_x_new[j] = second_x;
								connection_first_y_new[j] = second_y;
							} else if (connection_first_x_new[j] == second_x && connection_first_y_new[j] == second_y)
							{
								connection_first_x_new[j] = first_x;
								connection_first_y_new[j] = first_y;
							}

							if (connection_second_x_new[j] == first_x && connection_second_y_new[j] == first_y)
							{
								connection_second_x_new[j] = second_x;
								connection_second_y_new[j] = second_y;
							} else if (connection_second_x_new[j] == second_x && connection_second_y_new[j] == second_y)
							{
								connection_second_x_new[j] = first_x;
								connection_second_y_new[j] = first_y;
							}
						}

						
						if pcu_xy.contains(&(first_x, first_y)) && pcu_xy.contains(&(second_x, second_y)) // swap pcus
						{
							let mut idx_first = 0;
							let mut idx_second = 0;
							for j in 0..pcu_x.len()
							{
								if pcu_x[j] == first_x && pcu_y[j] == first_y
								{
									idx_first = j;
								} else if pcu_x[j] == second_x && pcu_y[j] == second_y
								{
									idx_second = j;
								}
							}

							pcu_counter_new.swap(idx_first, idx_second);
							pcu_SIMD_or_Systolic_new.swap(idx_first, idx_second);

						} else if pmu_xy.contains(&(first_x, first_y)) && pmu_xy.contains(&(second_x, second_y)) // swap pmus
						{
							let mut idx_first = 0;
							let mut idx_second = 0;
							for j in 0..pmu_x.len()
							{
								if pmu_x[j] == first_x && pmu_y[j] == first_y
								{
									idx_first = j;
								} else if pmu_x[j] == second_x && pmu_y[j] == second_y
								{
									idx_second = j;
								}
							}

							pmu_counter_new.swap(idx_first, idx_second);

						} else
						{
							panic!("Wrong!");
						}	
					}

					for j in 0..pcu_x.len()
					{
						let mut sender = vec![];
						let mut receiver = vec![];

						for k in 0..connection_first_x_new.len()
						{
							if pcu_x[j] == connection_first_x_new[k] && pcu_y[j] == connection_first_y_new[k]
							{
								sender.push(k);
							}

							if pcu_x[j] == connection_second_x_new[k] && pcu_y[j] == connection_second_y_new[k]
							{
								receiver.push(k);
							}
						}

						if sender.len() == 0
						{
							sender.push(999999);
						}

						if receiver.len() == 0
						{
							receiver.push(999999);
						}

						pcu_sender_vec_new.push(sender);
						pcu_receiver_vec_new.push(receiver);
					}





					for j in 0..pmu_x.len()
					{
						let mut sender = vec![];
						let mut receiver = vec![];

						for k in 0..connection_first_x_new.len()
						{
							if pmu_x[j] == connection_first_x_new[k] && pmu_y[j] == connection_first_y_new[k]
							{
								sender.push(k);
							}

							if pmu_x[j] == connection_second_x_new[k] && pmu_y[j] == connection_second_y_new[k]
							{
								receiver.push(k);
							}
						}

						if sender.len() == 0
						{
							sender.push(999999);
						}

						if receiver.len() == 0
						{
							receiver.push(999999);
						}

						pmu_sender_vec_new.push(sender);
						pmu_receiver_vec_new.push(receiver);
					}








					// calculate new channel load
					let mut used_link_map_new = HashMap::new();

					for j in 0..connection_first_x_new.len()
					{
						let mut curr_x = connection_first_x_new[j];
						let mut curr_y = connection_first_y_new[j];
						let mut dst_x = connection_second_x_new[j];
						let mut dst_y = connection_second_y_new[j];

						let link = (curr_x, curr_y, "from_L".to_owned(), curr_x, curr_y, "from_L".to_owned());
						if used_link_map_new.contains_key(&link)
						{	
							let tmp = used_link_map_new[&link] + 1;
							used_link_map_new.insert(link, tmp);
						} else
						{ 
							used_link_map_new.insert(link, 1);
						}


						let link = (dst_x, dst_y, "to_L".to_owned(), dst_x, dst_y, "to_L".to_owned());
						if used_link_map_new.contains_key(&link)
						{	
							let tmp = used_link_map_new[&link] + 1;
							used_link_map_new.insert(link, tmp);
						} else
						{ 
							used_link_map_new.insert(link, 1);
						}


						while true
						{
							// println!("j:{}, curr_x:{}, curr_y:{}, dst_x:{}, dst_y:{}, used_link_map_new:{:?}", j, curr_x, curr_y, dst_x, dst_y, used_link_map_new);

							if dst_x == curr_x && dst_y == curr_y // exit local port
							{
								break;
							} else if dst_x != curr_x && dst_y == curr_y // third hop, (curr_x, curr_y) -> (dst_x, curr_y)
							{
								let mut idx1 = 0;
								while (curr_x+idx1+1) % x_on_chip != dst_x
								{
									idx1 += 1;
								}

								let mut idx2 = 0;
								while (dst_x+idx2+1) % x_on_chip != curr_x
								{
									idx2 += 1;
								}

								let link = (curr_x, curr_y, "intra_".to_owned()+&(idx1).to_string(), dst_x, dst_y, "intra_".to_owned()+&(idx2).to_string());
								if used_link_map_new.contains_key(&link)
								{
									let tmp = used_link_map_new[&link] + 1;
									used_link_map_new.insert(link, tmp);
								} else
								{ 
									used_link_map_new.insert(link, 1);
								}
								curr_x = dst_x;
									
							} else
							{

								
								
								let mut dst_x_tmp = invalid;
								let mut curr_x_tmp = invalid;
								let mut curr_out_port_idx = invalid;
								for (key, value) in &inter_link_dict
								{
									if key.1 == curr_y
									{
										for m in 0..value.len()
										{
											if value[m].1 == dst_y
											{
												curr_x_tmp = key.0;
												dst_x_tmp = value[m].0;
												curr_out_port_idx = m;
												break;
											}
										}
									}
								}

								let mut dst_in_port_idx = invalid;
								for (key, value) in &inter_link_dict
								{
									if key.1 == dst_y
									{
										for m in 0..value.len()
										{
											if value[m].1 == curr_y
											{
												dst_in_port_idx = m;
												break;
											}
										}
									}
								}



								



								if curr_x == curr_x_tmp // second hop, (curr_x, curr_y) -> (dst_x_tmp, dst_y)
								{
									let link = (curr_x, curr_y, "inter_".to_owned()+&(radix_intra + curr_out_port_idx).to_string(), dst_x_tmp, dst_y, "inter_".to_owned()+&(radix_intra + dst_in_port_idx).to_string());
									if used_link_map_new.contains_key(&link)
									{
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}



									// if curr_y == 0 && dst_y == 2
									// {
									// 	let xxx = (curr_x, curr_y, "inter_".to_owned()+&(radix_intra + curr_out_port_idx).to_string(), curr_x, dst_y, "inter_".to_owned()+&(radix_intra + dst_in_port_idx).to_string());
									// 	println!("xxxxx curr_x:{}, curr_y:{}, dst_x:{}, dst_y:{}, curr_x_tmp:{}, xxx:{:?}", curr_x, curr_y, dst_x, dst_y, curr_x_tmp, xxx);
									// }

									curr_x = dst_x_tmp;
									curr_y = dst_y

								} else // first hop, (curr_x, curr_y) -> (curr_x_tmp, curr_y)
								{
									let mut idx1 = 0;
									while (curr_x+idx1+1) % x_on_chip != curr_x_tmp
									{
										idx1 += 1;
									}

									let mut idx2 = 0;
									while (curr_x_tmp+idx2+1) % x_on_chip != curr_x
									{
										idx2 += 1;
									}

									let link = (curr_x, curr_y, "intra_".to_owned()+&(idx1).to_string(), curr_x_tmp, curr_y, "intra_".to_owned()+&(idx2).to_string());
									if used_link_map_new.contains_key(&link)
									{
										let tmp = used_link_map_new[&link] + 1;
										used_link_map_new.insert(link, tmp);
									} else
									{ 
										used_link_map_new.insert(link, 1);
									}
									curr_x = curr_x_tmp;
								}

							}
						}
					}




					let mut new_max_channel_load = 0;
					for (key, value) in used_link_map_new.clone()
					{
						if key.2 != "from_L" && key.2 != "to_L" && new_max_channel_load < value
						{
							new_max_channel_load = value;
						}
					}






					if new_max_channel_load < current_max_channel_load
					{
						current_max_channel_load = new_max_channel_load;

						connection_first_x = connection_first_x_new.clone();
						connection_first_y = connection_first_y_new.clone();
						connection_second_x = connection_second_x_new.clone();
						connection_second_y = connection_second_y_new.clone();


						pcu_counter = pcu_counter_new.clone();
						pcu_SIMD_or_Systolic = pcu_SIMD_or_Systolic_new.clone();
						pcu_sender_vec = pcu_sender_vec_new.clone();
						pcu_receiver_vec = pcu_receiver_vec_new.clone();

						pmu_counter = pmu_counter_new.clone();
						pmu_sender_vec = pmu_sender_vec_new.clone();
						pmu_receiver_vec = pmu_receiver_vec_new.clone();


						used_link_map = used_link_map_new.clone();
					}
					max_channel_load_history.push(current_max_channel_load);
					println!("iter:{}, current_max_channel_load:{}", iter, current_max_channel_load);
				}
				println!("max_channel_load_history:{:?}", max_channel_load_history);
			}










		} else
		{
			panic!("Wrong!");
		}



		println!("-------------------------------------- used_link_map ---------------------------------------");
		let mut entries: Vec<_> = used_link_map.iter().collect();
		entries.sort_by(|a, b| b.1.cmp(a.1));
		let mut load = 0;
		for (key, value) in entries {
			if key.2 != "from_L" && key.2 != "to_L"
			{
				println!("key:{:?}, value:{}", key, value);
				load += value;
			}
		}
		println!("load:{}", load);
		println!("-------------------------------------- used_link_map ---------------------------------------");

		
		println!("pcu_counter {:?}", pcu_counter);
		println!("pcu_SIMD_or_Systolic {:?}", pcu_SIMD_or_Systolic);
		println!("pcu_sender_vec {:?}", pcu_sender_vec);
		println!("pcu_receiver_vec {:?}", pcu_receiver_vec);
		
		println!("pmu_counter {:?}", pmu_counter);
		println!("pmu_sender_vec {:?}", pmu_sender_vec);
		println!("pmu_receiver_vec {:?}", pmu_receiver_vec);







		




		// NoC global links
		let mut parent = ProgramBuilder::default();

		let mut sender_map_noc_global: HashMap<(usize, usize, String, usize, usize, String), dam::channel::Sender<usize>> = HashMap::new();
		let mut receiver_map_noc_global: HashMap<(usize, usize, String, usize, usize, String), dam::channel::Receiver<usize>> = HashMap::new();

		for ele in used_link_map.keys()
		{
			if ele.2 == "to_L".to_owned() || ele.2 == "from_L".to_owned()
			{

			} else
			{
				let (sender, receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
				sender_map_noc_global.insert(ele.clone(), sender);
				receiver_map_noc_global.insert(ele.clone(), receiver);
			}
		}

		



		// all involved routers
		let mut all_routers = HashSet::new();
		for ele in sender_map_noc_global.keys()
		{
			all_routers.insert((ele.0, ele.1));
			all_routers.insert((ele.3, ele.4));
		}

		for ele in receiver_map_noc_global.keys()
		{
			all_routers.insert((ele.0, ele.1));
			all_routers.insert((ele.3, ele.4));
		}
		println!("all_routers{:?}", all_routers);



		// extra routers, not attached to local PCUs/PMUs
		let mut extra_routers = HashSet::new();
		for ele in all_routers
		{
			let mut flag = false;
			for j in 0..pcu_x.len()
			{
				if pcu_x[j] == ele.0 && pcu_y[j] == ele.1
				{
					flag = true;
				}
			}
			for j in 0..pmu_x.len()
			{
				if pmu_x[j] == ele.0 && pmu_y[j] == ele.1
				{
					flag = true;
				}
			}

			if !flag
			{
				extra_routers.insert(ele.clone());
			}
		}
		println!("extra_routers{:?}", extra_routers);



		




		// compute tile
		for x in 0..x_on_chip
		{	
			for y in 0..y_on_chip
			{
				for j in 0..pcu_x.len()
				{
					if pcu_x[j] == x && pcu_y[j] == y
					{
						// router setup
						let mut router_in_stream = vec![];
						let mut router_in_dict: HashMap<String, (usize, usize)> = HashMap::new();
						let mut router_in_len = 0;
						
						let mut router_out_stream = vec![];
						let mut router_out_dict: HashMap<String, (usize, usize)> = HashMap::new();
						let mut router_out_len = 0;


						


						






						if topology_on_chip == "mesh"
						{
							// global links
							if receiver_map_noc_global.contains_key(&(x-1, y, "S".to_owned(), x, y, "N".to_owned()))
							{
								let N_in = receiver_map_noc_global.remove(&(x-1, y, "S".to_owned(), x, y, "N".to_owned())).unwrap();
								router_in_stream.push(N_in);
								
								let tmp = used_link_map[&(x-1, y, "S".to_owned(), x, y, "N".to_owned())];
								router_in_dict.insert("N_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}
							
							if receiver_map_noc_global.contains_key(&(x+1, y, "N".to_owned(), x, y, "S".to_owned()))
							{
								let S_in = receiver_map_noc_global.remove(&(x+1, y, "N".to_owned(), x, y, "S".to_owned())).unwrap();
								router_in_stream.push(S_in);

								let tmp = used_link_map[&(x+1, y, "N".to_owned(), x, y, "S".to_owned())];
								router_in_dict.insert("S_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}
							
							if receiver_map_noc_global.contains_key(&(x, y+1, "W".to_owned(), x, y, "E".to_owned()))
							{
								let E_in = receiver_map_noc_global.remove(&(x, y+1, "W".to_owned(), x, y, "E".to_owned())).unwrap();
								router_in_stream.push(E_in);

								let tmp = used_link_map[&(x, y+1, "W".to_owned(), x, y, "E".to_owned())];
								router_in_dict.insert("E_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}
							
							if receiver_map_noc_global.contains_key(&(x, y-1, "E".to_owned(), x, y, "W".to_owned()))
							{
								let W_in = receiver_map_noc_global.remove(&(x, y-1, "E".to_owned(), x, y, "W".to_owned())).unwrap();
								router_in_stream.push(W_in);

								let tmp = used_link_map[&(x, y-1, "E".to_owned(), x, y, "W".to_owned())];
								router_in_dict.insert("W_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}
							
							if sender_map_noc_global.contains_key(&(x, y, "N".to_owned(), x-1, y, "S".to_owned()))
							{
								let N_out = sender_map_noc_global.remove(&(x, y, "N".to_owned(), x-1, y, "S".to_owned())).unwrap();
								router_out_stream.push(N_out);

								let tmp = used_link_map[&(x, y, "N".to_owned(), x-1, y, "S".to_owned())];
								router_out_dict.insert("N_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}
							
							if sender_map_noc_global.contains_key(&(x, y, "S".to_owned(), x+1, y, "N".to_owned()))
							{
								let S_out = sender_map_noc_global.remove(&(x, y, "S".to_owned(), x+1, y, "N".to_owned())).unwrap();
								router_out_stream.push(S_out);

								let tmp = used_link_map[&(x, y, "S".to_owned(), x+1, y, "N".to_owned())];
								router_out_dict.insert("S_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}
							
							if sender_map_noc_global.contains_key(&(x, y, "E".to_owned(), x, y+1, "W".to_owned()))
							{
								let E_out = sender_map_noc_global.remove(&(x, y, "E".to_owned(), x, y+1, "W".to_owned())).unwrap();
								router_out_stream.push(E_out);

								let tmp = used_link_map[&(x, y, "E".to_owned(), x, y+1, "W".to_owned())];
								router_out_dict.insert("E_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}
							
							if sender_map_noc_global.contains_key(&(x, y, "W".to_owned(), x, y-1, "E".to_owned()))
							{
								let W_out = sender_map_noc_global.remove(&(x, y, "W".to_owned(), x, y-1, "E".to_owned())).unwrap();	
								router_out_stream.push(W_out);

								let tmp = used_link_map[&(x, y, "W".to_owned(), x, y-1, "E".to_owned())];
								router_out_dict.insert("W_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}

						} else if topology_on_chip == "torus"
						{
							let mut aaa;
							if x == 0
							{
								aaa = x_on_chip-1;
							} else
							{
								aaa = x-1;
							}
							if receiver_map_noc_global.contains_key(&(aaa, y, "S".to_owned(), x, y, "N".to_owned()))
							{
								let N_in = receiver_map_noc_global.remove(&(aaa, y, "S".to_owned(), x, y, "N".to_owned())).unwrap();
								router_in_stream.push(N_in);
								
								let tmp = used_link_map[&(aaa, y, "S".to_owned(), x, y, "N".to_owned())];
								router_in_dict.insert("N_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}
							



							if receiver_map_noc_global.contains_key(&((x+1)%x_on_chip, y, "N".to_owned(), x, y, "S".to_owned()))
							{
								let S_in = receiver_map_noc_global.remove(&((x+1)%x_on_chip, y, "N".to_owned(), x, y, "S".to_owned())).unwrap();
								router_in_stream.push(S_in);

								let tmp = used_link_map[&((x+1)%x_on_chip, y, "N".to_owned(), x, y, "S".to_owned())];
								router_in_dict.insert("S_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}



							
							if receiver_map_noc_global.contains_key(&(x, (y+1)%y_on_chip, "W".to_owned(), x, y, "E".to_owned()))
							{
								let E_in = receiver_map_noc_global.remove(&(x, (y+1)%y_on_chip, "W".to_owned(), x, y, "E".to_owned())).unwrap();
								router_in_stream.push(E_in);

								let tmp = used_link_map[&(x, (y+1)%y_on_chip, "W".to_owned(), x, y, "E".to_owned())];
								router_in_dict.insert("E_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}
							


							let mut aaa;
							if y == 0
							{
								aaa = y_on_chip-1;
							} else
							{
								aaa = y-1;
							}
							if receiver_map_noc_global.contains_key(&(x, aaa, "E".to_owned(), x, y, "W".to_owned()))
							{
								let W_in = receiver_map_noc_global.remove(&(x, aaa, "E".to_owned(), x, y, "W".to_owned())).unwrap();
								router_in_stream.push(W_in);

								let tmp = used_link_map[&(x, aaa, "E".to_owned(), x, y, "W".to_owned())];
								router_in_dict.insert("W_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}



							let mut aaa;
							if x == 0
							{
								aaa = x_on_chip-1;
							} else
							{
								aaa = x-1;
							}
							if sender_map_noc_global.contains_key(&(x, y, "N".to_owned(), aaa, y, "S".to_owned()))
							{
								let N_out = sender_map_noc_global.remove(&(x, y, "N".to_owned(), aaa, y, "S".to_owned())).unwrap();
								router_out_stream.push(N_out);

								let tmp = used_link_map[&(x, y, "N".to_owned(), aaa, y, "S".to_owned())];
								router_out_dict.insert("N_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}
							




							if sender_map_noc_global.contains_key(&(x, y, "S".to_owned(), (x+1)%x_on_chip, y, "N".to_owned()))
							{
								let S_out = sender_map_noc_global.remove(&(x, y, "S".to_owned(), (x+1)%x_on_chip, y, "N".to_owned())).unwrap();
								router_out_stream.push(S_out);

								let tmp = used_link_map[&(x, y, "S".to_owned(), (x+1)%x_on_chip, y, "N".to_owned())];
								router_out_dict.insert("S_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}
							



							if sender_map_noc_global.contains_key(&(x, y, "E".to_owned(), x, (y+1)%y_on_chip, "W".to_owned()))
							{
								let E_out = sender_map_noc_global.remove(&(x, y, "E".to_owned(), x, (y+1)%y_on_chip, "W".to_owned())).unwrap();
								router_out_stream.push(E_out);

								let tmp = used_link_map[&(x, y, "E".to_owned(), x, (y+1)%y_on_chip, "W".to_owned())];
								router_out_dict.insert("E_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}
							



							let mut aaa;
							if y == 0
							{
								aaa = y_on_chip-1;
							} else
							{
								aaa = y-1;
							}
							if sender_map_noc_global.contains_key(&(x, y, "W".to_owned(), x, aaa, "E".to_owned()))
							{
								let W_out = sender_map_noc_global.remove(&(x, y, "W".to_owned(), x, aaa, "E".to_owned())).unwrap();	
								router_out_stream.push(W_out);

								let tmp = used_link_map[&(x, y, "W".to_owned(), x, aaa, "E".to_owned())];
								router_out_dict.insert("W_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}


						} else if topology_on_chip == "dragonfly" 
						{

							let radix_intra = x_on_chip-1;
							let radix_inter = y_on_chip-1;

							

							// intra links
							for m in 0..x_on_chip
							{
								let mut idx1 = 0;
								while (x+idx1+1) % x_on_chip != m
								{
									idx1 += 1;
								}

								let mut idx2 = 0;
								while (m+idx2+1) % x_on_chip != x
								{
									idx2 += 1;
								}

								// let aaa = (m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string());
								// println!("{:?}", aaa);

								if receiver_map_noc_global.contains_key(&(m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string()))
								{
									// println!("exist!!!");

									let port = receiver_map_noc_global.remove(&(m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string())).unwrap();
									router_in_stream.push(port);
									
									let tmp = used_link_map[&(m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string())];
									router_in_dict.insert("intra_".to_owned()+&(idx1).to_string()+"_in", (router_in_len, tmp));
									router_in_len += 1;
								}


								// let aaa = (x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string());
								// println!("{:?}", aaa);

								if sender_map_noc_global.contains_key(&(x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string()))
								{
									// println!("exist!!!");

									let port = sender_map_noc_global.remove(&(x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string())).unwrap();
									router_out_stream.push(port);
									
									let tmp = used_link_map[&(x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string())];
									router_out_dict.insert("intra_".to_owned()+&(idx1).to_string()+"_out", (router_out_len, tmp));
									router_out_len += 1;
								}
							}
							

							// inter links
							for n in 0..y_on_chip
							{
								let mut curr_port_idx = invalid;
								let mut dst_x = invalid;
								for (key, value) in &inter_link_dict
								{
									if key.1 == y
									{
										for m in 0..value.len()
										{
											if value[m].1 == n
											{
												curr_port_idx = m;
												dst_x = value[m].0;
												break;
											}
										}
									}
								}

								let mut dst_port_idx = invalid;
								for (key, value) in &inter_link_dict
								{
									if key.1 == n
									{
										for m in 0..value.len()
										{
											if value[m].1 == y
											{
												dst_port_idx = m;
												break;
											}
										}
									}
								}

								if curr_port_idx == invalid && dst_port_idx == invalid
								{
									continue;
								}


								if receiver_map_noc_global.contains_key(&(dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string(), x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string()))
								{
									let port = receiver_map_noc_global.remove(&(dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string(), x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string())).unwrap();
									router_in_stream.push(port);
									
									let tmp = used_link_map[&(dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string(), x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string())];
									router_in_dict.insert("inter_".to_owned()+&(radix_intra + curr_port_idx).to_string()+"_in", (router_in_len, tmp));
									router_in_len += 1;
								}


								if sender_map_noc_global.contains_key(&(x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string(), dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string()))
								{
									let port = sender_map_noc_global.remove(&(x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string(), dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string())).unwrap();
									router_out_stream.push(port);
									
									let tmp = used_link_map[&(x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string(), dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string())];
									router_out_dict.insert("inter_".to_owned()+&(radix_intra + curr_port_idx).to_string()+"_out", (router_out_len, tmp));
									router_out_len += 1;
								}
							}
							
						} else
						{
							panic!("Wrong!");
						}


									



									
									





						let mut pcu_sender_vec_tmp = vec![];
						for n in 0..pcu_sender_vec[j].len()
						{
							pcu_sender_vec_tmp.push(pcu_sender_vec[j][n]);
						}		
						let mut pcu_receiver_vec_tmp = vec![];
						for n in 0..pcu_receiver_vec[j].len()
						{
							pcu_receiver_vec_tmp.push(pcu_receiver_vec[j][n]);
						}


						let simd_or_systolic = pcu_SIMD_or_Systolic[j];
						// let counter = pcu_counter[j];


						let no_connection = invalid;
						if pcu_receiver_vec_tmp[0] == no_connection && pcu_sender_vec_tmp[0] == no_connection
						{

						} else if pcu_receiver_vec_tmp[0] == no_connection
						{
							let mut tile_receiver_vec = vec![];
							let mut tile_sender_vec = vec![];
							let mut tile_dst_vec = vec![];
							let mut router_receiver_vec = vec![];

							for k in 0..pcu_sender_vec_tmp.len()
							{
								let (sender, receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								tile_sender_vec.push(sender);
								router_receiver_vec.push(receiver);

								let mut connection_id = pcu_sender_vec_tmp[k];
								if connection_first_type[connection_id] == "pcu" && connection_first_x[connection_id] == x && connection_first_y[connection_id] == y
								{
									tile_dst_vec.push(connection_second_x[connection_id] * y_on_chip + connection_second_y[connection_id]);
								} else {
									panic!("Wrong!");
								}
							}
							
							let (sender, receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let iter = || (0..(num_input_on_chip*max_counter)).map(|i| (i as usize) * 1_usize);
							let gen = GeneratorContext::new(iter, sender);
							parent.add_child(gen);
							tile_receiver_vec.push(receiver);

							if simd_or_systolic == "SIMD"
							{					
								let (sender1, receiver1) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								let (sender2, receiver2) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								
								let to_simd_pcu = to_simd_pcu::new(tile_receiver_vec, 1, sender1, num_input_on_chip as usize, max_counter, dummy);
								parent.add_child(to_simd_pcu);

								let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
								parent.add_child(pcu);

								let from_simd_pcu = from_simd_pcu::new(receiver2, tile_sender_vec, pcu_sender_vec_tmp.len() as usize, tile_dst_vec, num_input_on_chip as usize, max_counter, dummy);
								parent.add_child(from_simd_pcu);
							} else if simd_or_systolic == "Systolic"
							{
								let (sender1, receiver1) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								let (sender2, receiver2) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								let (sender3, receiver3) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								let (sender4, receiver4) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};

								let to_systolic_pcu = to_systolic_pcu::new(tile_receiver_vec, 1, sender1, sender2, num_input_on_chip as usize, max_counter, lane_dim, stage_dim, dummy);
								parent.add_child(to_systolic_pcu);

								let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
								parent.add_child(pcu_lane);

								let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
								parent.add_child(pcu_stage);

								let from_systolic_pcu = from_systolic_pcu::new(receiver3, receiver4, tile_sender_vec, pcu_sender_vec_tmp.len() as usize, tile_dst_vec, num_input_on_chip as usize, max_counter, lane_dim, stage_dim, dummy);
								parent.add_child(from_systolic_pcu);
							} else {
								panic!("Wrong!");
							}


							let (sender, receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let to_router = to_router::new(router_receiver_vec, pcu_sender_vec_tmp.len(), sender, num_input_on_chip, max_counter, dummy);
							parent.add_child(to_router);

							router_in_stream.push(receiver);


							let tmp = used_link_map[&(x, y, "from_L".to_owned(), x, y, "from_L".to_owned())];
							router_in_dict.insert("L_in".to_owned(), (router_in_len, tmp));
							router_in_len += 1;



							// println!("PCU: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
							if topology_on_chip == "mesh"
							{
								let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_mesh);
							} else if topology_on_chip == "torus"
							{
								let router_torus = router_torus::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_torus);
							} else if topology_on_chip == "dragonfly"
							{
								let router_dragonfly = router_dragonfly::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_dragonfly);
							} else
							{
								panic!("Wrong!");
							}

							

						} else if pcu_sender_vec_tmp[0] == no_connection
						{
							let (sender, receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							router_out_stream.push(sender);

							let tmp = used_link_map[&(x, y, "to_L".to_owned(), x, y, "to_L".to_owned())];
							router_out_dict.insert("L_out".to_owned(), (router_out_len, tmp));
							router_out_len += 1;


							// println!("PCU: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
							if topology_on_chip == "mesh"
							{
								let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_mesh);
							} else if topology_on_chip == "torus"
							{
								let router_torus = router_torus::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_torus);
							} else if topology_on_chip == "dragonfly"
							{
								let router_dragonfly = router_dragonfly::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_dragonfly);
							} else
							{
								panic!("Wrong!");
							}

							let con = ConsumerContext::new(receiver);
							parent.add_child(con);






						} else
						{
							let mut tile_sender_vec = vec![];
							let mut tile_dst_vec = vec![];
							let mut router_receiver_vec = vec![];

							let mut router_sender_vec = vec![];
							let mut tile_receiver_vec = vec![];

							for k in 0..pcu_receiver_vec_tmp.len()
							{
								let (sender, receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								router_sender_vec.push(sender);
								tile_receiver_vec.push(receiver);
							}
							for k in 0..pcu_sender_vec_tmp.len()
							{
								let (sender, receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								tile_sender_vec.push(sender);
								router_receiver_vec.push(receiver);

								let mut connection_id = pcu_sender_vec_tmp[k];
								if connection_first_type[connection_id] == "pcu" && connection_first_x[connection_id] == x && connection_first_y[connection_id] == y
								{
									tile_dst_vec.push(connection_second_x[connection_id] * y_on_chip + connection_second_y[connection_id]);
								} else {
									panic!("Wrong!");
								}
							}



							if simd_or_systolic == "SIMD"
							{					
								let (sender1, receiver1) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								let (sender2, receiver2) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								
								let to_simd_pcu = to_simd_pcu::new(tile_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, num_input_on_chip as usize, max_counter, dummy);
								parent.add_child(to_simd_pcu);

								let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
								parent.add_child(pcu);

								let from_simd_pcu = from_simd_pcu::new(receiver2, tile_sender_vec, pcu_sender_vec_tmp.len() as usize, tile_dst_vec, num_input_on_chip as usize, max_counter, dummy);
								parent.add_child(from_simd_pcu);
							} else if simd_or_systolic == "Systolic"
							{
								let (sender1, receiver1) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								let (sender2, receiver2) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								let (sender3, receiver3) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								let (sender4, receiver4) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};

								let to_systolic_pcu = to_systolic_pcu::new(tile_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, sender2, num_input_on_chip as usize, max_counter, lane_dim, stage_dim, dummy);
								parent.add_child(to_systolic_pcu);

								let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
								parent.add_child(pcu_lane);

								let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
								parent.add_child(pcu_stage);

								let from_systolic_pcu = from_systolic_pcu::new(receiver3, receiver4, tile_sender_vec, pcu_sender_vec_tmp.len() as usize, tile_dst_vec, num_input_on_chip as usize, max_counter, lane_dim, stage_dim, dummy);
								parent.add_child(from_systolic_pcu);
							} else {
								panic!("Wrong!");
							}




							let (sender1, receiver1) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let (sender2, receiver2) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};

							router_in_stream.push(receiver1);

							let tmp = used_link_map[&(x, y, "from_L".to_owned(), x, y, "from_L".to_owned())];
							router_in_dict.insert("L_in".to_owned(), (router_in_len, tmp));
							router_in_len += 1;


							router_out_stream.push(sender2);

							let tmp = used_link_map[&(x, y, "to_L".to_owned(), x, y, "to_L".to_owned())];
							router_out_dict.insert("L_out".to_owned(), (router_out_len, tmp));
							router_out_len += 1;




							let to_router = to_router::new(router_receiver_vec, pcu_sender_vec_tmp.len(), sender1, num_input_on_chip, max_counter, dummy);
							parent.add_child(to_router);

							// println!("PCU: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
							if topology_on_chip == "mesh"
							{
								let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_mesh);
							} else if topology_on_chip == "torus"
							{
								let router_torus = router_torus::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_torus);
							} else if topology_on_chip == "dragonfly"
							{
								let router_dragonfly = router_dragonfly::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_dragonfly);
							} else
							{
								panic!("Wrong!");
							}



							let from_router = from_router::new(receiver2, router_sender_vec, pcu_receiver_vec_tmp.len(), num_input_on_chip, max_counter, dummy);
							parent.add_child(from_router);

						}
					}
				}
			}
		}







		// memory tile
		for x in 0..x_on_chip
		{	
			for y in 0..y_on_chip
			{
				for j in 0..pmu_x.len()
				{
					if pmu_x[j] == x && pmu_y[j] == y
					{

						let mut x = pmu_x[j];
						let mut y = pmu_y[j];
						
						// router setup
						let mut router_in_stream = vec![];
						let mut router_in_dict: HashMap<String, (usize, usize)> = HashMap::new();
						let mut router_in_len = 0;
						
						let mut router_out_stream = vec![];
						let mut router_out_dict: HashMap<String, (usize, usize)> = HashMap::new();
						let mut router_out_len = 0;





						


						


						if topology_on_chip == "mesh"
						{
							// global links
							if receiver_map_noc_global.contains_key(&(x-1, y, "S".to_owned(), x, y, "N".to_owned()))
							{
								let N_in = receiver_map_noc_global.remove(&(x-1, y, "S".to_owned(), x, y, "N".to_owned())).unwrap();
								router_in_stream.push(N_in);
								
								let tmp = used_link_map[&(x-1, y, "S".to_owned(), x, y, "N".to_owned())];
								router_in_dict.insert("N_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}
							
							if receiver_map_noc_global.contains_key(&(x+1, y, "N".to_owned(), x, y, "S".to_owned()))
							{
								let S_in = receiver_map_noc_global.remove(&(x+1, y, "N".to_owned(), x, y, "S".to_owned())).unwrap();
								router_in_stream.push(S_in);

								let tmp = used_link_map[&(x+1, y, "N".to_owned(), x, y, "S".to_owned())];
								router_in_dict.insert("S_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}
							
							if receiver_map_noc_global.contains_key(&(x, y+1, "W".to_owned(), x, y, "E".to_owned()))
							{
								let E_in = receiver_map_noc_global.remove(&(x, y+1, "W".to_owned(), x, y, "E".to_owned())).unwrap();
								router_in_stream.push(E_in);

								let tmp = used_link_map[&(x, y+1, "W".to_owned(), x, y, "E".to_owned())];
								router_in_dict.insert("E_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}
							
							if receiver_map_noc_global.contains_key(&(x, y-1, "E".to_owned(), x, y, "W".to_owned()))
							{
								let W_in = receiver_map_noc_global.remove(&(x, y-1, "E".to_owned(), x, y, "W".to_owned())).unwrap();
								router_in_stream.push(W_in);

								let tmp = used_link_map[&(x, y-1, "E".to_owned(), x, y, "W".to_owned())];
								router_in_dict.insert("W_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}
							
							if sender_map_noc_global.contains_key(&(x, y, "N".to_owned(), x-1, y, "S".to_owned()))
							{
								let N_out = sender_map_noc_global.remove(&(x, y, "N".to_owned(), x-1, y, "S".to_owned())).unwrap();
								router_out_stream.push(N_out);

								let tmp = used_link_map[&(x, y, "N".to_owned(), x-1, y, "S".to_owned())];
								router_out_dict.insert("N_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}
							
							if sender_map_noc_global.contains_key(&(x, y, "S".to_owned(), x+1, y, "N".to_owned()))
							{
								let S_out = sender_map_noc_global.remove(&(x, y, "S".to_owned(), x+1, y, "N".to_owned())).unwrap();
								router_out_stream.push(S_out);

								let tmp = used_link_map[&(x, y, "S".to_owned(), x+1, y, "N".to_owned())];
								router_out_dict.insert("S_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}
							
							if sender_map_noc_global.contains_key(&(x, y, "E".to_owned(), x, y+1, "W".to_owned()))
							{
								let E_out = sender_map_noc_global.remove(&(x, y, "E".to_owned(), x, y+1, "W".to_owned())).unwrap();
								router_out_stream.push(E_out);

								let tmp = used_link_map[&(x, y, "E".to_owned(), x, y+1, "W".to_owned())];
								router_out_dict.insert("E_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}
							
							if sender_map_noc_global.contains_key(&(x, y, "W".to_owned(), x, y-1, "E".to_owned()))
							{
								let W_out = sender_map_noc_global.remove(&(x, y, "W".to_owned(), x, y-1, "E".to_owned())).unwrap();	
								router_out_stream.push(W_out);

								let tmp = used_link_map[&(x, y, "W".to_owned(), x, y-1, "E".to_owned())];
								router_out_dict.insert("W_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}

						} else if topology_on_chip == "torus"
						{
							let mut aaa;
							if x == 0
							{
								aaa = x_on_chip-1;
							} else
							{
								aaa = x-1;
							}
							if receiver_map_noc_global.contains_key(&(aaa, y, "S".to_owned(), x, y, "N".to_owned()))
							{
								let N_in = receiver_map_noc_global.remove(&(aaa, y, "S".to_owned(), x, y, "N".to_owned())).unwrap();
								router_in_stream.push(N_in);
								
								let tmp = used_link_map[&(aaa, y, "S".to_owned(), x, y, "N".to_owned())];
								router_in_dict.insert("N_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}
							



							if receiver_map_noc_global.contains_key(&((x+1)%x_on_chip, y, "N".to_owned(), x, y, "S".to_owned()))
							{
								let S_in = receiver_map_noc_global.remove(&((x+1)%x_on_chip, y, "N".to_owned(), x, y, "S".to_owned())).unwrap();
								router_in_stream.push(S_in);

								let tmp = used_link_map[&((x+1)%x_on_chip, y, "N".to_owned(), x, y, "S".to_owned())];
								router_in_dict.insert("S_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}



							
							if receiver_map_noc_global.contains_key(&(x, (y+1)%y_on_chip, "W".to_owned(), x, y, "E".to_owned()))
							{
								let E_in = receiver_map_noc_global.remove(&(x, (y+1)%y_on_chip, "W".to_owned(), x, y, "E".to_owned())).unwrap();
								router_in_stream.push(E_in);

								let tmp = used_link_map[&(x, (y+1)%y_on_chip, "W".to_owned(), x, y, "E".to_owned())];
								router_in_dict.insert("E_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}
							


							let mut aaa;
							if y == 0
							{
								aaa = y_on_chip-1;
							} else
							{
								aaa = y-1;
							}
							if receiver_map_noc_global.contains_key(&(x, aaa, "E".to_owned(), x, y, "W".to_owned()))
							{
								let W_in = receiver_map_noc_global.remove(&(x, aaa, "E".to_owned(), x, y, "W".to_owned())).unwrap();
								router_in_stream.push(W_in);

								let tmp = used_link_map[&(x, aaa, "E".to_owned(), x, y, "W".to_owned())];
								router_in_dict.insert("W_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;
							}



							let mut aaa;
							if x == 0
							{
								aaa = x_on_chip-1;
							} else
							{
								aaa = x-1;
							}
							if sender_map_noc_global.contains_key(&(x, y, "N".to_owned(), aaa, y, "S".to_owned()))
							{
								let N_out = sender_map_noc_global.remove(&(x, y, "N".to_owned(), aaa, y, "S".to_owned())).unwrap();
								router_out_stream.push(N_out);

								let tmp = used_link_map[&(x, y, "N".to_owned(), aaa, y, "S".to_owned())];
								router_out_dict.insert("N_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}
							




							if sender_map_noc_global.contains_key(&(x, y, "S".to_owned(), (x+1)%x_on_chip, y, "N".to_owned()))
							{
								let S_out = sender_map_noc_global.remove(&(x, y, "S".to_owned(), (x+1)%x_on_chip, y, "N".to_owned())).unwrap();
								router_out_stream.push(S_out);

								let tmp = used_link_map[&(x, y, "S".to_owned(), (x+1)%x_on_chip, y, "N".to_owned())];
								router_out_dict.insert("S_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}
							



							if sender_map_noc_global.contains_key(&(x, y, "E".to_owned(), x, (y+1)%y_on_chip, "W".to_owned()))
							{
								let E_out = sender_map_noc_global.remove(&(x, y, "E".to_owned(), x, (y+1)%y_on_chip, "W".to_owned())).unwrap();
								router_out_stream.push(E_out);

								let tmp = used_link_map[&(x, y, "E".to_owned(), x, (y+1)%y_on_chip, "W".to_owned())];
								router_out_dict.insert("E_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}
							



							let mut aaa;
							if y == 0
							{
								aaa = y_on_chip-1;
							} else
							{
								aaa = y-1;
							}
							if sender_map_noc_global.contains_key(&(x, y, "W".to_owned(), x, aaa, "E".to_owned()))
							{
								let W_out = sender_map_noc_global.remove(&(x, y, "W".to_owned(), x, aaa, "E".to_owned())).unwrap();	
								router_out_stream.push(W_out);

								let tmp = used_link_map[&(x, y, "W".to_owned(), x, aaa, "E".to_owned())];
								router_out_dict.insert("W_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;
							}


						} else if topology_on_chip == "dragonfly"
						{
							
							let radix_intra = x_on_chip-1;
							let radix_inter = y_on_chip-1;

							

							// intra links
							for m in 0..x_on_chip
							{
								let mut idx1 = 0;
								while (x+idx1+1) % x_on_chip != m
								{
									idx1 += 1;
								}

								let mut idx2 = 0;
								while (m+idx2+1) % x_on_chip != x
								{
									idx2 += 1;
								}

								// let aaa = (m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string());
								// println!("{:?}", aaa);

								if receiver_map_noc_global.contains_key(&(m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string()))
								{
									// println!("exist!!!");

									let port = receiver_map_noc_global.remove(&(m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string())).unwrap();
									router_in_stream.push(port);
									
									let tmp = used_link_map[&(m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string())];
									router_in_dict.insert("intra_".to_owned()+&(idx1).to_string()+"_in", (router_in_len, tmp));
									router_in_len += 1;
								}


								// let aaa = (x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string());
								// println!("{:?}", aaa);

								if sender_map_noc_global.contains_key(&(x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string()))
								{
									// println!("exist!!!");

									let port = sender_map_noc_global.remove(&(x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string())).unwrap();
									router_out_stream.push(port);
									
									let tmp = used_link_map[&(x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string())];
									router_out_dict.insert("intra_".to_owned()+&(idx1).to_string()+"_out", (router_out_len, tmp));
									router_out_len += 1;
								}
							}
							

							// inter links
							for n in 0..y_on_chip
							{
								let mut curr_port_idx = invalid;
								let mut dst_x = invalid;
								for (key, value) in &inter_link_dict
								{
									if key.1 == y
									{
										for m in 0..value.len()
										{
											if value[m].1 == n
											{
												curr_port_idx = m;
												dst_x = value[m].0;
												break;
											}
										}
									}
								}

								let mut dst_port_idx = invalid;
								for (key, value) in &inter_link_dict
								{
									if key.1 == n
									{
										for m in 0..value.len()
										{
											if value[m].1 == y
											{
												dst_port_idx = m;
												break;
											}
										}
									}
								}

								if curr_port_idx == invalid && dst_port_idx == invalid
								{
									continue;
								}


								if receiver_map_noc_global.contains_key(&(dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string(), x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string()))
								{
									let port = receiver_map_noc_global.remove(&(dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string(), x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string())).unwrap();
									router_in_stream.push(port);
									
									let tmp = used_link_map[&(dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string(), x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string())];
									router_in_dict.insert("inter_".to_owned()+&(radix_intra + curr_port_idx).to_string()+"_in", (router_in_len, tmp));
									router_in_len += 1;
								}


								if sender_map_noc_global.contains_key(&(x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string(), dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string()))
								{
									let port = sender_map_noc_global.remove(&(x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string(), dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string())).unwrap();
									router_out_stream.push(port);
									
									let tmp = used_link_map[&(x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string(), dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string())];
									router_out_dict.insert("inter_".to_owned()+&(radix_intra + curr_port_idx).to_string()+"_out", (router_out_len, tmp));
									router_out_len += 1;
								}
							}
							
						} else
						{
							panic!("Wrong!");
						}
									













						







						// let counter = pmu_counter[j];
						let mut pmu_sender_vec_tmp = vec![];
						for n in 0..pmu_sender_vec[j].len()
						{
							pmu_sender_vec_tmp.push(pmu_sender_vec[j][n]);
						}		
						let mut pmu_receiver_vec_tmp = vec![];
						for n in 0..pmu_receiver_vec[j].len()
						{
							pmu_receiver_vec_tmp.push(pmu_receiver_vec[j][n]);
						}
						



						let no_connection = invalid;
						if pmu_receiver_vec_tmp[0] == no_connection && pmu_sender_vec_tmp[0] == no_connection
						{
							
						} else if pmu_receiver_vec_tmp[0] == no_connection
						{
							let mut tile_receiver_vec = vec![];
							let mut tile_sender_vec = vec![];
							let mut tile_dst_vec = vec![];
							let mut router_receiver_vec = vec![];




							for k in 0..pmu_sender_vec_tmp.len()
							{
								let (sender, receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								tile_sender_vec.push(sender);
								router_receiver_vec.push(receiver);

								let mut connection_id = pmu_sender_vec_tmp[k];

								if connection_first_type[connection_id] == "pmu" && connection_first_x[connection_id] == x && connection_first_y[connection_id] == y
								{
									tile_dst_vec.push(connection_second_x[connection_id] * y_on_chip + connection_second_y[connection_id]);
								} else {
									panic!("Wrong!");
								}
							}

							
							let (sender, receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let iter = || (0..(num_input_on_chip*max_counter)).map(|i| (i as usize) * 1_usize);
							let gen = GeneratorContext::new(iter, sender);
							parent.add_child(gen);
							tile_receiver_vec.push(receiver);








							// PMU setup
							let (wr_addr_sender, wr_addr_receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let (wr_data_sender, wr_data_receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let (ack_sender, ack_receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let (rd_addr_sender, rd_addr_receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let (rd_data_sender, rd_data_receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};

							let to_pmu = to_pmu::new(tile_receiver_vec, 1, wr_addr_sender, wr_data_sender, num_input_on_chip as usize, max_counter, dummy);
							parent.add_child(to_pmu);

							let mut pmu: PMU<usize, usize, bool> = PMU::<usize, usize, bool>::new(
								num_vec_per_pmu,
								Behavior {
									mod_address: false,
									use_default_value: false,
								},
							);
							pmu.add_writer(PMUWriteBundle {
								addr: wr_addr_receiver,
								data: wr_data_receiver,
								ack: ack_sender,
							});
							pmu.add_reader(PMUReadBundle {
								addr: rd_addr_receiver,
								resp: rd_data_sender,
							});
							parent.add_child(pmu);
			
							let mut rd_addr_gen = FunctionContext::new();
							ack_receiver.attach_receiver(&rd_addr_gen);
							rd_addr_sender.attach_sender(&rd_addr_gen);
							let tmp = max_counter * num_input_on_chip;
							rd_addr_gen.set_run(move |time| {
								for idx in 0..tmp
								{
									ack_receiver.dequeue(time).unwrap();
									let curr_time = time.tick();
									rd_addr_sender.enqueue(time, ChannelElement{time: curr_time, data: usize::try_from(0).unwrap(),},).unwrap();
								}
							});
							parent.add_child(rd_addr_gen);
			
							let from_pmu = from_pmu::new(rd_data_receiver, tile_sender_vec, pmu_sender_vec_tmp.len() as usize, tile_dst_vec, num_input_on_chip as usize, max_counter, dummy);
							parent.add_child(from_pmu);










							let (sender, receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let to_router = to_router::new(router_receiver_vec, pmu_sender_vec_tmp.len(), sender, num_input_on_chip, max_counter, dummy);
							parent.add_child(to_router);

							router_in_stream.push(receiver);

							let tmp = used_link_map[&(x, y, "from_L".to_owned(), x, y, "from_L".to_owned())];
							router_in_dict.insert("L_in".to_owned(), (router_in_len, tmp));
							router_in_len += 1;

							// println!("PMU: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
							if topology_on_chip == "mesh"
							{
								let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_mesh);
							} else if topology_on_chip == "torus"
							{
								let router_torus = router_torus::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_torus);
							} else if topology_on_chip == "dragonfly"
							{
								let router_dragonfly = router_dragonfly::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_dragonfly);
							} else
							{
								panic!("Wrong!");
							}



						} else if pmu_sender_vec_tmp[0] == no_connection
						{
							let (sender, receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							router_out_stream.push(sender);

							let tmp = used_link_map[&(x, y, "to_L".to_owned(), x, y, "to_L".to_owned())];
							router_out_dict.insert("L_out".to_owned(), (router_out_len, tmp));
							router_out_len += 1;

							// println!("PMU: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
							if topology_on_chip == "mesh"
							{
								let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_mesh);
							} else if topology_on_chip == "torus"
							{
								let router_torus = router_torus::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_torus);
							} else if topology_on_chip == "dragonfly"
							{
								let router_dragonfly = router_dragonfly::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_dragonfly);
							} else
							{
								panic!("Wrong!");
							}


							let con = ConsumerContext::new(receiver);
							parent.add_child(con);




						} else
						{
							let mut tile_sender_vec = vec![];
							let mut tile_dst_vec = vec![];
							let mut router_receiver_vec = vec![];

							let mut router_sender_vec = vec![];
							let mut tile_receiver_vec = vec![];

							for k in 0..pmu_receiver_vec_tmp.len()
							{
								let (sender, receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								router_sender_vec.push(sender);
								tile_receiver_vec.push(receiver);
							}
							for k in 0..pmu_sender_vec_tmp.len()
							{
								let (sender, receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
								tile_sender_vec.push(sender);
								router_receiver_vec.push(receiver);

								let mut connection_id = pmu_sender_vec_tmp[k];
								if connection_first_type[connection_id] == "pmu" && connection_first_x[connection_id] == x && connection_first_y[connection_id] == y
								{
									tile_dst_vec.push(connection_second_x[connection_id] * y_on_chip + connection_second_y[connection_id]);
								} else {
									panic!("Wrong!");
								}
							}

							


							// PMU setup
							let (wr_addr_sender, wr_addr_receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let (wr_data_sender, wr_data_receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let (ack_sender, ack_receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let (rd_addr_sender, rd_addr_receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let (rd_data_sender, rd_data_receiver) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};

							let to_pmu = to_pmu::new(tile_receiver_vec, pmu_receiver_vec_tmp.len() as usize, wr_addr_sender, wr_data_sender, num_input_on_chip as usize, max_counter, dummy);
							parent.add_child(to_pmu);

							let mut pmu: PMU<usize, usize, bool> = PMU::<usize, usize, bool>::new(
								num_vec_per_pmu,
								Behavior {
									mod_address: false,
									use_default_value: false,
								},
							);
							pmu.add_writer(PMUWriteBundle {
								addr: wr_addr_receiver,
								data: wr_data_receiver,
								ack: ack_sender,
							});
							pmu.add_reader(PMUReadBundle {
								addr: rd_addr_receiver,
								resp: rd_data_sender,
							});
							parent.add_child(pmu);
							
							let mut rd_addr_gen = FunctionContext::new();
							ack_receiver.attach_receiver(&rd_addr_gen);
							rd_addr_sender.attach_sender(&rd_addr_gen);
							let tmp = max_counter * num_input_on_chip;
							rd_addr_gen.set_run(move |time| {
								for idx in 0..tmp
								{
									ack_receiver.dequeue(time).unwrap();
									let curr_time = time.tick();
									rd_addr_sender.enqueue(time, ChannelElement{time: curr_time, data: usize::try_from(0).unwrap(),},).unwrap();
								}
							});
							parent.add_child(rd_addr_gen);
			
							let from_pmu = from_pmu::new(rd_data_receiver, tile_sender_vec, pmu_sender_vec_tmp.len() as usize, tile_dst_vec, num_input_on_chip as usize, max_counter, dummy);
							parent.add_child(from_pmu);









							let (sender1, receiver1) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							let (sender2, receiver2) = if (buffer_depth != 0) {parent.bounded(buffer_depth)} else {parent.unbounded()};
							router_in_stream.push(receiver1);

							let tmp = used_link_map[&(x, y, "from_L".to_owned(), x, y, "from_L".to_owned())];
							router_in_dict.insert("L_in".to_owned(), (router_in_len, tmp));
							router_in_len += 1;

							router_out_stream.push(sender2);
							
							let tmp = used_link_map[&(x, y, "to_L".to_owned(), x, y, "to_L".to_owned())];
							router_out_dict.insert("L_out".to_owned(), (router_out_len, tmp));
							router_out_len += 1;






							let to_router = to_router::new(router_receiver_vec, pmu_sender_vec_tmp.len(), sender1, num_input_on_chip, max_counter, dummy);
							parent.add_child(to_router);

							// println!("PMU: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
							if topology_on_chip == "mesh"
							{
								let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_mesh);
							} else if topology_on_chip == "torus"
							{
								let router_torus = router_torus::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_torus);
							} else if topology_on_chip == "dragonfly"
							{
								let router_dragonfly = router_dragonfly::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
								parent.add_child(router_dragonfly);
							} else
							{
								panic!("Wrong!");
							}




							let from_router = from_router::new(receiver2, router_sender_vec, pmu_receiver_vec_tmp.len(), num_input_on_chip, max_counter, dummy);
							parent.add_child(from_router);

						}
					}
				}
			}
		}




		// extra routers
		for ele in extra_routers
		{
			let x = ele.0;
			let y = ele.1;
			// router setup
			let mut router_in_stream = vec![];
			let mut router_in_dict: HashMap<String, (usize, usize)> = HashMap::new();
			let mut router_in_len = 0;
			
			let mut router_out_stream = vec![];
			let mut router_out_dict: HashMap<String, (usize, usize)> = HashMap::new();
			let mut router_out_len = 0;


			if topology_on_chip == "mesh"
			{
				// global links
				if receiver_map_noc_global.contains_key(&(x-1, y, "S".to_owned(), x, y, "N".to_owned()))
				{
					let N_in = receiver_map_noc_global.remove(&(x-1, y, "S".to_owned(), x, y, "N".to_owned())).unwrap();
					router_in_stream.push(N_in);
					
					let tmp = used_link_map[&(x-1, y, "S".to_owned(), x, y, "N".to_owned())];
					router_in_dict.insert("N_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}
				
				if receiver_map_noc_global.contains_key(&(x+1, y, "N".to_owned(), x, y, "S".to_owned()))
				{
					let S_in = receiver_map_noc_global.remove(&(x+1, y, "N".to_owned(), x, y, "S".to_owned())).unwrap();
					router_in_stream.push(S_in);

					let tmp = used_link_map[&(x+1, y, "N".to_owned(), x, y, "S".to_owned())];
					router_in_dict.insert("S_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}
				
				if receiver_map_noc_global.contains_key(&(x, y+1, "W".to_owned(), x, y, "E".to_owned()))
				{
					let E_in = receiver_map_noc_global.remove(&(x, y+1, "W".to_owned(), x, y, "E".to_owned())).unwrap();
					router_in_stream.push(E_in);

					let tmp = used_link_map[&(x, y+1, "W".to_owned(), x, y, "E".to_owned())];
					router_in_dict.insert("E_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}
				
				if receiver_map_noc_global.contains_key(&(x, y-1, "E".to_owned(), x, y, "W".to_owned()))
				{
					let W_in = receiver_map_noc_global.remove(&(x, y-1, "E".to_owned(), x, y, "W".to_owned())).unwrap();
					router_in_stream.push(W_in);

					let tmp = used_link_map[&(x, y-1, "E".to_owned(), x, y, "W".to_owned())];
					router_in_dict.insert("W_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}
				
				if sender_map_noc_global.contains_key(&(x, y, "N".to_owned(), x-1, y, "S".to_owned()))
				{
					let N_out = sender_map_noc_global.remove(&(x, y, "N".to_owned(), x-1, y, "S".to_owned())).unwrap();
					router_out_stream.push(N_out);

					let tmp = used_link_map[&(x, y, "N".to_owned(), x-1, y, "S".to_owned())];
					router_out_dict.insert("N_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}
				
				if sender_map_noc_global.contains_key(&(x, y, "S".to_owned(), x+1, y, "N".to_owned()))
				{
					let S_out = sender_map_noc_global.remove(&(x, y, "S".to_owned(), x+1, y, "N".to_owned())).unwrap();
					router_out_stream.push(S_out);

					let tmp = used_link_map[&(x, y, "S".to_owned(), x+1, y, "N".to_owned())];
					router_out_dict.insert("S_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}
				
				if sender_map_noc_global.contains_key(&(x, y, "E".to_owned(), x, y+1, "W".to_owned()))
				{
					let E_out = sender_map_noc_global.remove(&(x, y, "E".to_owned(), x, y+1, "W".to_owned())).unwrap();
					router_out_stream.push(E_out);

					let tmp = used_link_map[&(x, y, "E".to_owned(), x, y+1, "W".to_owned())];
					router_out_dict.insert("E_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}
				
				if sender_map_noc_global.contains_key(&(x, y, "W".to_owned(), x, y-1, "E".to_owned()))
				{
					let W_out = sender_map_noc_global.remove(&(x, y, "W".to_owned(), x, y-1, "E".to_owned())).unwrap();	
					router_out_stream.push(W_out);

					let tmp = used_link_map[&(x, y, "W".to_owned(), x, y-1, "E".to_owned())];
					router_out_dict.insert("W_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}

			} else if topology_on_chip == "torus"
			{
				let mut aaa;
				if x == 0
				{
					aaa = x_on_chip-1;
				} else
				{
					aaa = x-1;
				}
				if receiver_map_noc_global.contains_key(&(aaa, y, "S".to_owned(), x, y, "N".to_owned()))
				{
					let N_in = receiver_map_noc_global.remove(&(aaa, y, "S".to_owned(), x, y, "N".to_owned())).unwrap();
					router_in_stream.push(N_in);
					
					let tmp = used_link_map[&(aaa, y, "S".to_owned(), x, y, "N".to_owned())];
					router_in_dict.insert("N_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}
				



				if receiver_map_noc_global.contains_key(&((x+1)%x_on_chip, y, "N".to_owned(), x, y, "S".to_owned()))
				{
					let S_in = receiver_map_noc_global.remove(&((x+1)%x_on_chip, y, "N".to_owned(), x, y, "S".to_owned())).unwrap();
					router_in_stream.push(S_in);

					let tmp = used_link_map[&((x+1)%x_on_chip, y, "N".to_owned(), x, y, "S".to_owned())];
					router_in_dict.insert("S_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}



				
				if receiver_map_noc_global.contains_key(&(x, (y+1)%y_on_chip, "W".to_owned(), x, y, "E".to_owned()))
				{
					let E_in = receiver_map_noc_global.remove(&(x, (y+1)%y_on_chip, "W".to_owned(), x, y, "E".to_owned())).unwrap();
					router_in_stream.push(E_in);

					let tmp = used_link_map[&(x, (y+1)%y_on_chip, "W".to_owned(), x, y, "E".to_owned())];
					router_in_dict.insert("E_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}
				


				let mut aaa;
				if y == 0
				{
					aaa = y_on_chip-1;
				} else
				{
					aaa = y-1;
				}
				if receiver_map_noc_global.contains_key(&(x, aaa, "E".to_owned(), x, y, "W".to_owned()))
				{
					let W_in = receiver_map_noc_global.remove(&(x, aaa, "E".to_owned(), x, y, "W".to_owned())).unwrap();
					router_in_stream.push(W_in);

					let tmp = used_link_map[&(x, aaa, "E".to_owned(), x, y, "W".to_owned())];
					router_in_dict.insert("W_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}



				let mut aaa;
				if x == 0
				{
					aaa = x_on_chip-1;
				} else
				{
					aaa = x-1;
				}
				if sender_map_noc_global.contains_key(&(x, y, "N".to_owned(), aaa, y, "S".to_owned()))
				{
					let N_out = sender_map_noc_global.remove(&(x, y, "N".to_owned(), aaa, y, "S".to_owned())).unwrap();
					router_out_stream.push(N_out);

					let tmp = used_link_map[&(x, y, "N".to_owned(), aaa, y, "S".to_owned())];
					router_out_dict.insert("N_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}
				




				if sender_map_noc_global.contains_key(&(x, y, "S".to_owned(), (x+1)%x_on_chip, y, "N".to_owned()))
				{
					let S_out = sender_map_noc_global.remove(&(x, y, "S".to_owned(), (x+1)%x_on_chip, y, "N".to_owned())).unwrap();
					router_out_stream.push(S_out);

					let tmp = used_link_map[&(x, y, "S".to_owned(), (x+1)%x_on_chip, y, "N".to_owned())];
					router_out_dict.insert("S_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}
				



				if sender_map_noc_global.contains_key(&(x, y, "E".to_owned(), x, (y+1)%y_on_chip, "W".to_owned()))
				{
					let E_out = sender_map_noc_global.remove(&(x, y, "E".to_owned(), x, (y+1)%y_on_chip, "W".to_owned())).unwrap();
					router_out_stream.push(E_out);

					let tmp = used_link_map[&(x, y, "E".to_owned(), x, (y+1)%y_on_chip, "W".to_owned())];
					router_out_dict.insert("E_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}
				



				let mut aaa;
				if y == 0
				{
					aaa = y_on_chip-1;
				} else
				{
					aaa = y-1;
				}
				if sender_map_noc_global.contains_key(&(x, y, "W".to_owned(), x, aaa, "E".to_owned()))
				{
					let W_out = sender_map_noc_global.remove(&(x, y, "W".to_owned(), x, aaa, "E".to_owned())).unwrap();	
					router_out_stream.push(W_out);

					let tmp = used_link_map[&(x, y, "W".to_owned(), x, aaa, "E".to_owned())];
					router_out_dict.insert("W_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}


			} else if topology_on_chip == "dragonfly" 
			{
				
				let radix_intra = x_on_chip-1;
				let radix_inter = y_on_chip-1;

				

				// intra links
				for m in 0..x_on_chip
				{
					let mut idx1 = 0;
					while (x+idx1+1) % x_on_chip != m
					{
						idx1 += 1;
					}

					let mut idx2 = 0;
					while (m+idx2+1) % x_on_chip != x
					{
						idx2 += 1;
					}

					// let aaa = (m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string());
					// println!("{:?}", aaa);

					if receiver_map_noc_global.contains_key(&(m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string()))
					{
						// println!("exist!!!");

						let port = receiver_map_noc_global.remove(&(m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string())).unwrap();
						router_in_stream.push(port);
						
						let tmp = used_link_map[&(m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string())];
						router_in_dict.insert("intra_".to_owned()+&(idx1).to_string()+"_in", (router_in_len, tmp));
						router_in_len += 1;
					}


					// let aaa = (x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string());
					// println!("{:?}", aaa);

					if sender_map_noc_global.contains_key(&(x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string()))
					{
						// println!("exist!!!");

						let port = sender_map_noc_global.remove(&(x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string())).unwrap();
						router_out_stream.push(port);
						
						let tmp = used_link_map[&(x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string())];
						router_out_dict.insert("intra_".to_owned()+&(idx1).to_string()+"_out", (router_out_len, tmp));
						router_out_len += 1;
					}
				}
				

				// inter links
				for n in 0..y_on_chip
				{
					let mut curr_port_idx = invalid;
					let mut dst_x = invalid;
					for (key, value) in &inter_link_dict
					{
						if key.1 == y
						{
							for m in 0..value.len()
							{
								if value[m].1 == n
								{
									curr_port_idx = m;
									dst_x = value[m].0;
									break;
								}
							}
						}
					}

					let mut dst_port_idx = invalid;
					for (key, value) in &inter_link_dict
					{
						if key.1 == n
						{
							for m in 0..value.len()
							{
								if value[m].1 == y
								{
									dst_port_idx = m;
									break;
								}
							}
						}
					}

					if curr_port_idx == invalid && dst_port_idx == invalid
					{
						continue;
					}


					if receiver_map_noc_global.contains_key(&(dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string(), x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string()))
					{
						let port = receiver_map_noc_global.remove(&(dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string(), x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string())).unwrap();
						router_in_stream.push(port);
						
						let tmp = used_link_map[&(dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string(), x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string())];
						router_in_dict.insert("inter_".to_owned()+&(radix_intra + curr_port_idx).to_string()+"_in", (router_in_len, tmp));
						router_in_len += 1;
					}


					if sender_map_noc_global.contains_key(&(x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string(), dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string()))
					{
						let port = sender_map_noc_global.remove(&(x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string(), dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string())).unwrap();
						router_out_stream.push(port);
						
						let tmp = used_link_map[&(x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string(), dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string())];
						router_out_dict.insert("inter_".to_owned()+&(radix_intra + curr_port_idx).to_string()+"_out", (router_out_len, tmp));
						router_out_len += 1;
					}
				}
				
			} else
			{
				panic!("Wrong!");
			}




			



			// println!("router: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
			if topology_on_chip == "mesh"
			{
				let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
				parent.add_child(router_mesh);
			} else if topology_on_chip == "torus"
			{
				let router_torus = router_torus::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
				parent.add_child(router_torus);
			} else if topology_on_chip == "dragonfly"
			{
				let router_dragonfly = router_dragonfly::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_on_chip, y_on_chip, x, y, num_input_on_chip, max_counter, dummy);
				parent.add_child(router_dragonfly);
			} else
			{
				panic!("Wrong!");
			}


		}


		


		


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


		let start = Instant::now();
		let executed = initialized.run(
			RunOptionsBuilder::default()
				.mode(RunMode::Simple)
				.build()
				.unwrap(),
		);
		println!("Elapsed cycles: {:?}", executed.elapsed_cycles());
		let duration = start.elapsed();
		let seconds = duration.as_secs_f32();
		println!("Runtime (s): {}", seconds);

		let tmp: u64 = executed.elapsed_cycles().unwrap();
		let tmp: f32 = tmp as f32 / num_input_on_chip as f32 / freq as f32;
		dam_compute_time.push(tmp);


		println!("\n\n\n\n\n\n\n\n\n\n");



	}





	println!("\n\n\n\n\n\n\n\n\n\n");

























		
















	// network
	for i in 0..num_config
	{
		println!("------------------------------- config: {} -------------------------------------", i);


		let mut connection_first_type: Vec<String> = vec![];
		let mut connection_first_x: Vec<usize> = vec![];
		let mut connection_first_y: Vec<usize> = vec![];
		let mut connection_second_type: Vec<String> = vec![];
		let mut connection_second_x: Vec<usize> = vec![];
		let mut connection_second_y: Vec<usize> = vec![];



		for line in lines.lines() {
			let str = format!("connection off_chip_config {} ", i);
			if line.starts_with(&str)
			{
				let tmp: Vec<&str> = line.split_whitespace().collect();
				let tmp1 = tmp[3].parse().unwrap();
				let tmp2 = tmp[4].parse().unwrap();
				let tmp3 = tmp[5].parse().unwrap();
				let tmp4 = tmp[6].parse().unwrap();
				let tmp5 = tmp[7].parse().unwrap();
				let tmp6 = tmp[8].parse().unwrap();
				connection_first_type.push(tmp1);
				connection_first_x.push(tmp2);
				connection_first_y.push(tmp3);
				connection_second_type.push(tmp4);
				connection_second_x.push(tmp5);
				connection_second_y.push(tmp6);
			}	
		}



		


		let mut num_of_connections = 0;
		let mut network_bytes: f32 = 0.0;



		for line in lines.lines() {
			let str = format!("num_of_connections off_chip_config {} ", i);
			if line.starts_with(&str)
			{
				let tmp: Vec<&str> = line.split_whitespace().collect();
				let tmp1 = tmp[tmp.len() - 1].parse().unwrap();
				num_of_connections = tmp1;
			}	
		}

		for line in lines.lines() {
			let str = format!("network_bytes off_chip_config {} ", i);
			if line.starts_with(&str)
			{
				let tmp: Vec<&str> = line.split_whitespace().collect();
				let tmp1: f32 = tmp[tmp.len() - 1].parse().unwrap();
				network_bytes = tmp1;
			}	
		}


		if num_of_connections == 0 || topology_off_chip == "skip"
		{
			dam_network_time.push(0.0);
			continue;
		}











		println!("--------  network, {} --------", topology_off_chip);
		
		// for dragonfly
		let mut inter_link_dict = HashMap::new();
		for m in 0..x_off_chip
		{
			for n in 0..y_off_chip
			{
				let mut tmp = vec![];
				inter_link_dict.insert((m, n), tmp);
			}
		}

		let mut num_connection = vec![];
		let quotient = (y_off_chip-1) / x_off_chip;
		let remainer = (y_off_chip-1) - quotient * x_off_chip;
		for m in 0..x_off_chip
		{						
			num_connection.push(quotient);
		}
		for m in 0..remainer
		{
			num_connection[m] += 1;
		}

		println!("num_connection:{:?}", num_connection);
		

		for m in 0..y_off_chip
		{
			let dst_x = m % x_off_chip;
			let mut cnt = m+1;

			for n in 0..num_connection.len()
			{
				let mut aaa = 0;
				while aaa < num_connection[n]
				{
					let dst_y = cnt % y_off_chip;
					if inter_link_dict[&(n, m)].len() < num_connection[n] && inter_link_dict[&(dst_x, dst_y)].len() < num_connection[dst_x]
					{
						let mut tmp = inter_link_dict[&(n, m)].clone();
						tmp.push((dst_x, dst_y));
						inter_link_dict.insert((n, m), tmp);

						let mut tmp = inter_link_dict[&(dst_x, dst_y)].clone();
						tmp.push((n, m));
						inter_link_dict.insert((dst_x, dst_y), tmp);

						// println!("n:{}, m:{}, dst_x:{}, dst_y:{}, inter_link_dict:{:?}", n, m, dst_x, dst_y, inter_link_dict);
						
						cnt += 1;
					}
					aaa += 1;
				}

			}

		}

		
		let mut my_dict: Vec<_> = inter_link_dict.iter().collect();
		my_dict.sort_by(|a, b| a.1.cmp(b.1));

		println!("inter_link_dict:");
		for (key, value) in &my_dict
		{
			println!("{:?}: {:?}", key, value);
		}




		let mut parent = ProgramBuilder::default();
		let mut used_link_map = HashMap::new();

		if topology_off_chip == "mesh"
		{
			for j in 0..connection_first_x.len()
			{
				let mut curr_x = connection_first_x[j];
				let mut curr_y = connection_first_y[j];
				let mut dst_x = connection_second_x[j];
				let mut dst_y = connection_second_y[j];

				let link = (curr_x, curr_y, "from_L".to_owned(), curr_x, curr_y, "from_L".to_owned());
				if used_link_map.contains_key(&link)
				{	
					let tmp = used_link_map[&link] + 1;
					used_link_map.insert(link, tmp);
				} else
				{ 
					used_link_map.insert(link, 1);
				}


				let link = (dst_x, dst_y, "to_L".to_owned(), dst_x, dst_y, "to_L".to_owned());
				if used_link_map.contains_key(&link)
				{	
					let tmp = used_link_map[&link] + 1;
					used_link_map.insert(link, tmp);
				} else
				{ 
					used_link_map.insert(link, 1);
				}


				while true
				{
					if dst_x == curr_x && dst_y == curr_y // exit local port
					{
						break;
					} else if dst_x == curr_x && dst_y < curr_y // exit W port
					{
						let link = (curr_x, curr_y, "W".to_owned(), curr_x, curr_y-1, "E".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_y -= 1;

					} else if dst_x < curr_x && dst_y < curr_y // exit N port
					{
						let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x -= 1;

					} else if dst_x < curr_x && dst_y == curr_y // exit N port
					{
						let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x -= 1;

					} else if dst_x < curr_x && dst_y > curr_y // exit N port
					{
						let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x -= 1;

					} else if dst_x == curr_x && dst_y > curr_y // exit E port
					{
						let link = (curr_x, curr_y, "E".to_owned(), curr_x, curr_y+1, "W".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_y += 1;

					} else if dst_x > curr_x && dst_y > curr_y // exit S port
					{
						let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x += 1;

					} else if dst_x > curr_x && dst_y == curr_y // exit S port
					{
						let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x += 1;

					} else if dst_x > curr_x && dst_y < curr_y // exit S port
					{
						let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
						if used_link_map.contains_key(&link)
						{	
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x += 1;

					} else
					{
						panic!("Wrong!");
					}
				}
			}

		} else if topology_off_chip == "torus"
		{
			for j in 0..connection_first_x.len()
			{
				let mut curr_x = connection_first_x[j];
				let mut curr_y = connection_first_y[j];
				let mut dst_x = connection_second_x[j];
				let mut dst_y = connection_second_y[j];

				let link = (curr_x, curr_y, "from_L".to_owned(), curr_x, curr_y, "from_L".to_owned());
				if used_link_map.contains_key(&link)
				{	
					let tmp = used_link_map[&link] + 1;
					used_link_map.insert(link, tmp);
				} else
				{ 
					used_link_map.insert(link, 1);
				}


				let link = (dst_x, dst_y, "to_L".to_owned(), dst_x, dst_y, "to_L".to_owned());
				if used_link_map.contains_key(&link)
				{	
					let tmp = used_link_map[&link] + 1;
					used_link_map.insert(link, tmp);
				} else
				{ 
					used_link_map.insert(link, 1);
				}


				while true
				{

					// println!("j:{}, curr_x:{}, curr_y:{}, dst_x:{}, dst_y:{}, used_link_map:{:?}", j, curr_x, curr_y, dst_x, dst_y, used_link_map);

					if dst_x == curr_x && dst_y == curr_y // exit local port
					{
						break;
					} else if dst_x == curr_x && dst_y < curr_y
					{
						let tmp1 = curr_y - dst_y;
						let tmp2 = y_off_chip - tmp1;


						if tmp1 < tmp2 // exit W port
						{
							let link = (curr_x, curr_y, "W".to_owned(), curr_x, curr_y-1, "E".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_y -= 1;
						} else // exit E port
						{
							let link = (curr_x, curr_y, "E".to_owned(), curr_x, (curr_y+1)%y_off_chip, "W".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_y = (curr_y+1)%y_off_chip;
						}
						

					} else if dst_x < curr_x && dst_y < curr_y
					{
						let tmp1 = curr_x - dst_x;
						let tmp2 = x_off_chip - tmp1;

						if tmp1 < tmp2 // exit N port
						{
							let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x -= 1;
						} else // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), (curr_x+1)%x_off_chip, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = (curr_x+1)%x_off_chip;
						}
						






					} else if dst_x < curr_x && dst_y == curr_y
					{
						let tmp1 = curr_x - dst_x;
						let tmp2 = x_off_chip - tmp1;

						if tmp1 < tmp2 // exit N port
						{
							let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x -= 1;
						} else // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), (curr_x+1)%x_off_chip, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = (curr_x+1)%x_off_chip;
						}
						


					} else if dst_x < curr_x && dst_y > curr_y // exit N port
					{
						let tmp1 = curr_x - dst_x;
						let tmp2 = x_off_chip - tmp1;

						if tmp1 < tmp2 // exit N port
						{
							let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x -= 1;
						} else // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), (curr_x+1)%x_off_chip, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = (curr_x+1)%x_off_chip;
						}
						





					} else if dst_x == curr_x && dst_y > curr_y
					{
						let tmp1 = dst_y - curr_y;
						let tmp2 = y_off_chip - tmp1;

						if tmp1 < tmp2 // exit E port
						{
							let link = (curr_x, curr_y, "E".to_owned(), curr_x, curr_y+1, "W".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_y += 1;
						} else // exit W port
						{
							let mut ttt;
							if curr_y == 0
							{
								ttt = y_off_chip-1;
							} else
							{
								ttt = curr_y-1;	
							}		 
							let link = (curr_x, curr_y, "W".to_owned(), curr_x, ttt, "E".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_y = ttt;
						}







					} else if dst_x > curr_x && dst_y > curr_y
					{
						let tmp1 = dst_x - curr_x;
						let tmp2 = x_off_chip - tmp1;

						if tmp1 < tmp2 // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x += 1;
						} else // exit N port
						{
							let mut ttt;
							if curr_x == 0
							{
								ttt = x_off_chip-1;
							} else
							{
								ttt = curr_x-1;	
							}	
							let link = (curr_x, curr_y, "N".to_owned(), ttt, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = ttt;
						}
						




					} else if dst_x > curr_x && dst_y == curr_y
					{
						let tmp1 = dst_x - curr_x;
						let tmp2 = x_off_chip - tmp1;

						if tmp1 < tmp2 // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x += 1;
						} else // exit N port
						{
							let mut ttt;
							if curr_x == 0
							{
								ttt = x_off_chip-1;
							} else
							{
								ttt = curr_x-1;	
							}	
							let link = (curr_x, curr_y, "N".to_owned(), ttt, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = ttt;
						}




					} else if dst_x > curr_x && dst_y < curr_y
					{
						let tmp1 = dst_x - curr_x;
						let tmp2 = x_off_chip - tmp1;

						if tmp1 < tmp2 // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x += 1;
						} else // exit N port
						{
							let mut ttt;
							if curr_x == 0
							{
								ttt = x_off_chip-1;
							} else
							{
								ttt = curr_x-1;	
							}	
							let link = (curr_x, curr_y, "N".to_owned(), ttt, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = ttt;
						}

					} else
					{
						panic!("Wrong!");
					}
				}
			}

		} else if topology_off_chip == "dragonfly"
		{
			let radix_intra = x_off_chip-1;
			let radix_inter = y_off_chip-1;








			for j in 0..connection_first_x.len()
			{
				let mut curr_x = connection_first_x[j];
				let mut curr_y = connection_first_y[j];
				let mut dst_x = connection_second_x[j];
				let mut dst_y = connection_second_y[j];

				let link = (curr_x, curr_y, "from_L".to_owned(), curr_x, curr_y, "from_L".to_owned());
				if used_link_map.contains_key(&link)
				{	
					let tmp = used_link_map[&link] + 1;
					used_link_map.insert(link, tmp);
				} else
				{ 
					used_link_map.insert(link, 1);
				}


				let link = (dst_x, dst_y, "to_L".to_owned(), dst_x, dst_y, "to_L".to_owned());
				if used_link_map.contains_key(&link)
				{	
					let tmp = used_link_map[&link] + 1;
					used_link_map.insert(link, tmp);
				} else
				{ 
					used_link_map.insert(link, 1);
				}


				while true
				{
					// println!("j:{}, curr_x:{}, curr_y:{}, dst_x:{}, dst_y:{}, used_link_map:{:?}", j, curr_x, curr_y, dst_x, dst_y, used_link_map);

					if dst_x == curr_x && dst_y == curr_y // exit local port
					{
						break;
					} else if dst_x != curr_x && dst_y == curr_y // third hop, (curr_x, curr_y) -> (dst_x, curr_y)
					{
						let mut idx1 = 0;
						while (curr_x+idx1+1) % x_off_chip != dst_x
						{
							idx1 += 1;
						}

						let mut idx2 = 0;
						while (dst_x+idx2+1) % x_off_chip != curr_x
						{
							idx2 += 1;
						}

						let link = (curr_x, curr_y, "intra_".to_owned()+&(idx1).to_string(), dst_x, dst_y, "intra_".to_owned()+&(idx2).to_string());
						if used_link_map.contains_key(&link)
						{
							let tmp = used_link_map[&link] + 1;
							used_link_map.insert(link, tmp);
						} else
						{ 
							used_link_map.insert(link, 1);
						}
						curr_x = dst_x;
							
					} else
					{

						
						
						let mut dst_x_tmp = invalid;
						let mut curr_x_tmp = invalid;
						let mut curr_out_port_idx = invalid;
						for (key, value) in &inter_link_dict
						{
							if key.1 == curr_y
							{
								for m in 0..value.len()
								{
									if value[m].1 == dst_y
									{
										curr_x_tmp = key.0;
										dst_x_tmp = value[m].0;
										curr_out_port_idx = m;
										break;
									}
								}
							}
						}

						let mut dst_in_port_idx = invalid;
						for (key, value) in &inter_link_dict
						{
							if key.1 == dst_y
							{
								for m in 0..value.len()
								{
									if value[m].1 == curr_y
									{
										dst_in_port_idx = m;
										break;
									}
								}
							}
						}



						



						if curr_x == curr_x_tmp // second hop, (curr_x, curr_y) -> (dst_x_tmp, dst_y)
						{
							let link = (curr_x, curr_y, "inter_".to_owned()+&(radix_intra + curr_out_port_idx).to_string(), dst_x_tmp, dst_y, "inter_".to_owned()+&(radix_intra + dst_in_port_idx).to_string());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}



							// if curr_y == 0 && dst_y == 2
							// {
							// 	let xxx = (curr_x, curr_y, "inter_".to_owned()+&(radix_intra + curr_out_port_idx).to_string(), curr_x, dst_y, "inter_".to_owned()+&(radix_intra + dst_in_port_idx).to_string());
							// 	println!("xxxxx curr_x:{}, curr_y:{}, dst_x:{}, dst_y:{}, curr_x_tmp:{}, xxx:{:?}", curr_x, curr_y, dst_x, dst_y, curr_x_tmp, xxx);
							// }

							curr_x = dst_x_tmp;
							curr_y = dst_y

						} else // first hop, (curr_x, curr_y) -> (curr_x_tmp, curr_y)
						{
							let mut idx1 = 0;
							while (curr_x+idx1+1) % x_off_chip != curr_x_tmp
							{
								idx1 += 1;
							}

							let mut idx2 = 0;
							while (curr_x_tmp+idx2+1) % x_off_chip != curr_x
							{
								idx2 += 1;
							}

							let link = (curr_x, curr_y, "intra_".to_owned()+&(idx1).to_string(), curr_x_tmp, curr_y, "intra_".to_owned()+&(idx2).to_string());
							if used_link_map.contains_key(&link)
							{
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x = curr_x_tmp;
						}

					}
				}
			}

		} else
		{
			panic!("Wrong!");
		}

		
		

		




		// NoC global links
		let mut sender_map_noc_global: HashMap<(usize, usize, String, usize, usize, String), dam::channel::Sender<usize>> = HashMap::new();
		let mut receiver_map_noc_global: HashMap<(usize, usize, String, usize, usize, String), dam::channel::Receiver<usize>> = HashMap::new();

		for ele in used_link_map.keys()
		{
			let (sender, receiver) = parent.unbounded();
			sender_map_noc_global.insert(ele.clone(), sender);
			receiver_map_noc_global.insert(ele.clone(), receiver);
		}


		let mut my_dict: Vec<_> = used_link_map.iter().collect();
		my_dict.sort_by(|a, b| a.1.cmp(b.1));

		println!("used_link_map:");
		for (key, value) in &my_dict
		{
			println!("{:?}: {}", key, value);
		}








		// all involved routers
		let mut all_routers = HashSet::new();
		for ele in sender_map_noc_global.keys()
		{
			all_routers.insert((ele.0, ele.1));
			all_routers.insert((ele.3, ele.4));
		}

		for ele in receiver_map_noc_global.keys()
		{
			all_routers.insert((ele.0, ele.1));
			all_routers.insert((ele.3, ele.4));
		}

		println!("all_routers{:?}", all_routers);







		// get all dsts for all routers
		let mut dst_dict: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
		for j in 0..connection_first_x.len()
		{
			let src_x = connection_first_x[j];
			let src_y = connection_first_y[j];
			let dst_x = connection_second_x[j];
			let dst_y = connection_second_y[j];

			if dst_dict.contains_key(&(src_x, src_y))
			{
				let mut tmp = dst_dict[&(src_x, src_y)].clone();
				tmp.push(dst_x*y_off_chip+dst_y);
				dst_dict.insert((src_x, src_y), tmp);
			} else
			{
				let mut tmp = vec![];
				tmp.push(dst_x*y_off_chip+dst_y);
				dst_dict.insert((src_x, src_y), tmp);
			}	
		}

		println!("dst_dict:{:?}", dst_dict);



		// all routers
		for ele in all_routers
		{
			let x = ele.0;
			let y = ele.1;
			// router setup
			let mut router_in_stream = vec![];
			let mut router_in_dict: HashMap<String, (usize, usize)> = HashMap::new();
			let mut router_in_len = 0;
			
			let mut router_out_stream = vec![];
			let mut router_out_dict: HashMap<String, (usize, usize)> = HashMap::new();
			let mut router_out_len = 0;


			if topology_off_chip == "mesh"
			{
				
				// global links
				if receiver_map_noc_global.contains_key(&(x-1, y, "S".to_owned(), x, y, "N".to_owned()))
				{
					let N_in = receiver_map_noc_global.remove(&(x-1, y, "S".to_owned(), x, y, "N".to_owned())).unwrap();
					router_in_stream.push(N_in);
					
					let tmp = used_link_map[&(x-1, y, "S".to_owned(), x, y, "N".to_owned())];
					router_in_dict.insert("N_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}
				
				if receiver_map_noc_global.contains_key(&(x+1, y, "N".to_owned(), x, y, "S".to_owned()))
				{
					let S_in = receiver_map_noc_global.remove(&(x+1, y, "N".to_owned(), x, y, "S".to_owned())).unwrap();
					router_in_stream.push(S_in);

					let tmp = used_link_map[&(x+1, y, "N".to_owned(), x, y, "S".to_owned())];
					router_in_dict.insert("S_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}
				
				if receiver_map_noc_global.contains_key(&(x, y+1, "W".to_owned(), x, y, "E".to_owned()))
				{
					let E_in = receiver_map_noc_global.remove(&(x, y+1, "W".to_owned(), x, y, "E".to_owned())).unwrap();
					router_in_stream.push(E_in);

					let tmp = used_link_map[&(x, y+1, "W".to_owned(), x, y, "E".to_owned())];
					router_in_dict.insert("E_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}
				
				if receiver_map_noc_global.contains_key(&(x, y-1, "E".to_owned(), x, y, "W".to_owned()))
				{
					let W_in = receiver_map_noc_global.remove(&(x, y-1, "E".to_owned(), x, y, "W".to_owned())).unwrap();
					router_in_stream.push(W_in);

					let tmp = used_link_map[&(x, y-1, "E".to_owned(), x, y, "W".to_owned())];
					router_in_dict.insert("W_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}
				
				if sender_map_noc_global.contains_key(&(x, y, "N".to_owned(), x-1, y, "S".to_owned()))
				{
					let N_out = sender_map_noc_global.remove(&(x, y, "N".to_owned(), x-1, y, "S".to_owned())).unwrap();
					router_out_stream.push(N_out);

					let tmp = used_link_map[&(x, y, "N".to_owned(), x-1, y, "S".to_owned())];
					router_out_dict.insert("N_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}
				
				if sender_map_noc_global.contains_key(&(x, y, "S".to_owned(), x+1, y, "N".to_owned()))
				{
					let S_out = sender_map_noc_global.remove(&(x, y, "S".to_owned(), x+1, y, "N".to_owned())).unwrap();
					router_out_stream.push(S_out);

					let tmp = used_link_map[&(x, y, "S".to_owned(), x+1, y, "N".to_owned())];
					router_out_dict.insert("S_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}
				
				if sender_map_noc_global.contains_key(&(x, y, "E".to_owned(), x, y+1, "W".to_owned()))
				{
					let E_out = sender_map_noc_global.remove(&(x, y, "E".to_owned(), x, y+1, "W".to_owned())).unwrap();
					router_out_stream.push(E_out);

					let tmp = used_link_map[&(x, y, "E".to_owned(), x, y+1, "W".to_owned())];
					router_out_dict.insert("E_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}
				
				if sender_map_noc_global.contains_key(&(x, y, "W".to_owned(), x, y-1, "E".to_owned()))
				{
					let W_out = sender_map_noc_global.remove(&(x, y, "W".to_owned(), x, y-1, "E".to_owned())).unwrap();	
					router_out_stream.push(W_out);

					let tmp = used_link_map[&(x, y, "W".to_owned(), x, y-1, "E".to_owned())];
					router_out_dict.insert("W_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}

			} else if topology_off_chip == "torus"
			{
				let mut aaa;
				if x == 0
				{
					aaa = x_off_chip-1;
				} else
				{
					aaa = x-1;
				}
				if receiver_map_noc_global.contains_key(&(aaa, y, "S".to_owned(), x, y, "N".to_owned()))
				{
					let N_in = receiver_map_noc_global.remove(&(aaa, y, "S".to_owned(), x, y, "N".to_owned())).unwrap();
					router_in_stream.push(N_in);
					
					let tmp = used_link_map[&(aaa, y, "S".to_owned(), x, y, "N".to_owned())];
					router_in_dict.insert("N_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}
				



				if receiver_map_noc_global.contains_key(&((x+1)%x_off_chip, y, "N".to_owned(), x, y, "S".to_owned()))
				{
					let S_in = receiver_map_noc_global.remove(&((x+1)%x_off_chip, y, "N".to_owned(), x, y, "S".to_owned())).unwrap();
					router_in_stream.push(S_in);

					let tmp = used_link_map[&((x+1)%x_off_chip, y, "N".to_owned(), x, y, "S".to_owned())];
					router_in_dict.insert("S_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}



				
				if receiver_map_noc_global.contains_key(&(x, (y+1)%y_off_chip, "W".to_owned(), x, y, "E".to_owned()))
				{
					let E_in = receiver_map_noc_global.remove(&(x, (y+1)%y_off_chip, "W".to_owned(), x, y, "E".to_owned())).unwrap();
					router_in_stream.push(E_in);

					let tmp = used_link_map[&(x, (y+1)%y_off_chip, "W".to_owned(), x, y, "E".to_owned())];
					router_in_dict.insert("E_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}
				


				let mut aaa;
				if y == 0
				{
					aaa = y_off_chip-1;
				} else
				{
					aaa = y-1;
				}
				if receiver_map_noc_global.contains_key(&(x, aaa, "E".to_owned(), x, y, "W".to_owned()))
				{
					let W_in = receiver_map_noc_global.remove(&(x, aaa, "E".to_owned(), x, y, "W".to_owned())).unwrap();
					router_in_stream.push(W_in);

					let tmp = used_link_map[&(x, aaa, "E".to_owned(), x, y, "W".to_owned())];
					router_in_dict.insert("W_in".to_owned(), (router_in_len, tmp));
					router_in_len += 1;
				}



				let mut aaa;
				if x == 0
				{
					aaa = x_off_chip-1;
				} else
				{
					aaa = x-1;
				}
				if sender_map_noc_global.contains_key(&(x, y, "N".to_owned(), aaa, y, "S".to_owned()))
				{
					let N_out = sender_map_noc_global.remove(&(x, y, "N".to_owned(), aaa, y, "S".to_owned())).unwrap();
					router_out_stream.push(N_out);

					let tmp = used_link_map[&(x, y, "N".to_owned(), aaa, y, "S".to_owned())];
					router_out_dict.insert("N_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}
				




				if sender_map_noc_global.contains_key(&(x, y, "S".to_owned(), (x+1)%x_off_chip, y, "N".to_owned()))
				{
					let S_out = sender_map_noc_global.remove(&(x, y, "S".to_owned(), (x+1)%x_off_chip, y, "N".to_owned())).unwrap();
					router_out_stream.push(S_out);

					let tmp = used_link_map[&(x, y, "S".to_owned(), (x+1)%x_off_chip, y, "N".to_owned())];
					router_out_dict.insert("S_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}
				



				if sender_map_noc_global.contains_key(&(x, y, "E".to_owned(), x, (y+1)%y_off_chip, "W".to_owned()))
				{
					let E_out = sender_map_noc_global.remove(&(x, y, "E".to_owned(), x, (y+1)%y_off_chip, "W".to_owned())).unwrap();
					router_out_stream.push(E_out);

					let tmp = used_link_map[&(x, y, "E".to_owned(), x, (y+1)%y_off_chip, "W".to_owned())];
					router_out_dict.insert("E_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}
				



				let mut aaa;
				if y == 0
				{
					aaa = y_off_chip-1;
				} else
				{
					aaa = y-1;
				}
				if sender_map_noc_global.contains_key(&(x, y, "W".to_owned(), x, aaa, "E".to_owned()))
				{
					let W_out = sender_map_noc_global.remove(&(x, y, "W".to_owned(), x, aaa, "E".to_owned())).unwrap();	
					router_out_stream.push(W_out);

					let tmp = used_link_map[&(x, y, "W".to_owned(), x, aaa, "E".to_owned())];
					router_out_dict.insert("W_out".to_owned(), (router_out_len, tmp));
					router_out_len += 1;
				}


			} else if topology_off_chip == "dragonfly"
			{

				println!("-------------------------- {}, {} -------------------------------", x, y);


				let radix_intra = x_off_chip-1;
				let radix_inter = y_off_chip-1;

				

				// intra links
				for m in 0..x_off_chip
				{
					let mut idx1 = 0;
					while (x+idx1+1) % x_off_chip != m
					{
						idx1 += 1;
					}

					let mut idx2 = 0;
					while (m+idx2+1) % x_off_chip != x
					{
						idx2 += 1;
					}

					// let aaa = (m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string());
					// println!("{:?}", aaa);

					if receiver_map_noc_global.contains_key(&(m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string()))
					{
						// println!("exist!!!");

						let port = receiver_map_noc_global.remove(&(m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string())).unwrap();
						router_in_stream.push(port);
						
						let tmp = used_link_map[&(m, y, "intra_".to_owned()+&(idx2).to_string(), x, y, "intra_".to_owned()+&(idx1).to_string())];
						router_in_dict.insert("intra_".to_owned()+&(idx1).to_string()+"_in", (router_in_len, tmp));
						router_in_len += 1;
					}


					// let aaa = (x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string());
					// println!("{:?}", aaa);

					if sender_map_noc_global.contains_key(&(x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string()))
					{
						// println!("exist!!!");

						let port = sender_map_noc_global.remove(&(x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string())).unwrap();
						router_out_stream.push(port);
						
						let tmp = used_link_map[&(x, y, "intra_".to_owned()+&(idx1).to_string(), m, y, "intra_".to_owned()+&(idx2).to_string())];
						router_out_dict.insert("intra_".to_owned()+&(idx1).to_string()+"_out", (router_out_len, tmp));
						router_out_len += 1;
					}
				}
				



				// inter links
				for n in 0..y_off_chip
				{


					// (dst_x, n) -> (x, y)
					let mut curr_port_idx = invalid;
					let mut dst_x = invalid;
					for (key, value) in &inter_link_dict
					{
						if key.1 == y
						{
							for m in 0..value.len()
							{
								if value[m].1 == n
								{
									curr_port_idx = m;
									dst_x = value[m].0;
									break;
								}
							}
						}
					}


					// (x, y) -> (dst_x, n)
					let mut dst_port_idx = invalid;
					for (key, value) in &inter_link_dict
					{
						if key.1 == n
						{
							for m in 0..value.len()
							{
								if value[m].1 == y
								{
									dst_port_idx = m;
									break;
								}
							}
						}
					}



					if curr_port_idx == invalid && dst_port_idx == invalid
					{
						continue;
					}


					if receiver_map_noc_global.contains_key(&(dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string(), x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string()))
					{
						let port = receiver_map_noc_global.remove(&(dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string(), x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string())).unwrap();
						router_in_stream.push(port);
						
						let tmp = used_link_map[&(dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string(), x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string())];
						router_in_dict.insert("inter_".to_owned()+&(radix_intra + curr_port_idx).to_string()+"_in", (router_in_len, tmp));
						router_in_len += 1;

						
					}


					if sender_map_noc_global.contains_key(&(x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string(), dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string()))
					{
						let port = sender_map_noc_global.remove(&(x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string(), dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string())).unwrap();
						router_out_stream.push(port);
						
						let tmp = used_link_map[&(x, y, "inter_".to_owned()+&(radix_intra + curr_port_idx).to_string(), dst_x, n, "inter_".to_owned()+&(radix_intra + dst_port_idx).to_string())];
						router_out_dict.insert("inter_".to_owned()+&(radix_intra + curr_port_idx).to_string()+"_out", (router_out_len, tmp));
						router_out_len += 1;
					}
				}
			

				
				

			} else
			{
				panic!("Wrong!");
			}







			if sender_map_noc_global.contains_key(&(x, y, "from_L".to_owned(), x, y, "from_L".to_owned()))
			{
				let L_in = receiver_map_noc_global.remove(&(x, y, "from_L".to_owned(), x, y, "from_L".to_owned())).unwrap();
				router_in_stream.push(L_in);

				let tmp = used_link_map[&(x, y, "from_L".to_owned(), x, y, "from_L".to_owned())];
				router_in_dict.insert("L_in".to_owned(), (router_in_len, tmp));
				router_in_len += 1;


				// attach generators and to_router
				let mut in_stream = vec![];
				if dst_dict.contains_key(&(x, y)) && dst_dict[&(x, y)].len() == tmp
				{
					let value = dst_dict[&(x, y)].clone();
					for ele in value
					{
						let (s, r) = parent.unbounded();
						let iter = {
							let ele_copy = ele as usize;
							move || (0..(num_input_off_chip)).map(move |i| ele_copy)
						};
						let gen = GeneratorContext::new(iter, s);
						parent.add_child(gen);
						in_stream.push(r);
					}
				} else
				{
					panic!("Wrong!");
				}

				let sender = sender_map_noc_global.remove(&(x, y, "from_L".to_owned(), x, y, "from_L".to_owned())).unwrap();
				let adapter = to_router::new(in_stream, tmp, sender, num_input_off_chip, 1, dummy);
				parent.add_child(adapter);
			}


			if sender_map_noc_global.contains_key(&(x, y, "to_L".to_owned(), x, y, "to_L".to_owned()))
			{
				let L_out = sender_map_noc_global.remove(&(x, y, "to_L".to_owned(), x, y, "to_L".to_owned())).unwrap();
				router_out_stream.push(L_out);

				let tmp = used_link_map[&(x, y, "to_L".to_owned(), x, y, "to_L".to_owned())];
				router_out_dict.insert("L_out".to_owned(), (router_out_len, tmp));
				router_out_len += 1;


				// attach sink
				let receiver = receiver_map_noc_global.remove(&(x, y, "to_L".to_owned(), x, y, "to_L".to_owned())).unwrap();
				let con = ConsumerContext::new(receiver);
				parent.add_child(con);
			}





			

			if topology_off_chip == "mesh"
			{
				let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_off_chip, y_off_chip, x, y, num_input_off_chip, 1, dummy);
				parent.add_child(router_mesh);
			} else if topology_off_chip == "torus"
			{
				let router_torus = router_torus::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_off_chip, y_off_chip, x, y, num_input_off_chip, 1, dummy);
				parent.add_child(router_torus);
			} else if topology_off_chip == "dragonfly"
			{
				// println!("router_in_dict:{:?}, router_in_len:{}, router_out_dict:{:?}, router_out_len:{}", router_in_dict, router_in_len, router_out_dict, router_out_len);

				let router_dragonfly = router_dragonfly::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_off_chip, y_off_chip, x, y, num_input_off_chip, 1, dummy);
				parent.add_child(router_dragonfly);
			} else
			{
				panic!("Wrong!");
			}


		}


	


	


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


		let start = Instant::now();
		let executed = initialized.run(
			RunOptionsBuilder::default()
				.mode(RunMode::Simple)
				.build()
				.unwrap(),
		);
		println!("Elapsed cycles: {:?}", executed.elapsed_cycles());
		let duration = start.elapsed();
		let seconds = duration.as_secs_f32();
		println!("Runtime (s): {}", seconds);

		let tmp: u64 = executed.elapsed_cycles().unwrap();
		let tmp: f32 = network_bytes as f32 / net_bw as f32 * tmp as f32 / num_input_off_chip as f32;
		dam_network_time.push(tmp);

		
		println!("\n\n\n\n\n\n\n\n\n\n");

	}















	let mut dfmodel_time: Vec<f32> = vec![];
	let mut dam_memory_time = Memory_Latency.clone();
	let mut dam_time = vec![];

	for i in 0..num_config
	{
		dam_time.push(dam_compute_time[i].max(dam_memory_time[i].max(dam_network_time[i])));
	}

	for i in 0..num_config
	{
		dfmodel_time.push(Compute_Latency[i].max(Memory_Latency[i].max(Network_Latency[i])));
	}

	
	println!("dfmodel_compute_time:{:?}, total:{}", Compute_Latency.clone(), sum_elements(Compute_Latency.clone()));
	println!("dfmodel_memory_time:{:?}, total:{}", Memory_Latency.clone(), sum_elements(Memory_Latency.clone()));
	println!("dfmodel_network_time:{:?}, total:{}", Network_Latency.clone(), sum_elements(Network_Latency.clone()));
	println!("dfmodel_time:{:?}, total:{}", dfmodel_time.clone(), sum_elements(dfmodel_time.clone()));
	println!("dam_compute_time:{:?}, total:{}", dam_compute_time.clone(), sum_elements(dam_compute_time.clone()));
	println!("dam_memory_time:{:?}, total:{}", dam_memory_time.clone(), sum_elements(dam_memory_time.clone()));
	println!("dam_network_time:{:?}, total:{}", dam_network_time.clone(), sum_elements(dam_network_time.clone()));
	println!("dam_time:{:?}, total:{}", dam_time.clone(), sum_elements(dam_time.clone()));

}