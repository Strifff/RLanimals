use rand::{seq::index, Rng};
use rand_distr::{Distribution, Normal};
use serde_json::{json, Value};
use tch::{
    nn,
    nn::VarStore,
    nn::{init, OptimizerConfig},
    nn::{Module, Optimizer},
    Kind, Tensor,
};

use crate::{
    ACTIONS, CHOOSE_MAX_FIT, MAX_FILES, MAX_WEIGHT_BIAS, MUTATION_RATE, NN_RAYS, NN_RAY_DR,
    NN_RAY_LEN, NORMAL_MEAN, NORMAL_STDDEV, N_TYPES, START_SPARCITY,
};

const SIZE_FULL: usize = (NN_RAYS * NN_RAY_LEN);
const SIZE_HALF: usize = (SIZE_FULL as f32 * 0.5) as usize;
const SIZE_QTR: usize = (SIZE_FULL as f32 * 0.25) as usize;

pub struct genAlgoNN {
    //pub wall: nn::Sequential,
    pub plant: nn::Sequential,
    //pub herbi: nn::Sequential,
    //pub carni: nn::Sequential,
    //pub all: nn::Sequential,
    //pub actor: nn::Sequential,
    //pub critic: nn::Sequential,
}
pub fn apply_op<F>(x: f32, operation: F) -> f32
where
    F: Fn(f32) -> f32,
{
    operation(x)
}

pub fn init_models_ws_bs<F>(inputs: [&str; 1], for_model: &str, op: F)
where
    F: Fn(f32) -> f32,
{
    let mut rng = rand::thread_rng();

    let mut first_layer_ws = [0.0; NN_RAYS * NN_RAY_LEN * SIZE_HALF];
    let mut first_layer_bs = [0.0; SIZE_HALF];

    let mut second_layer_ws = [0.0; SIZE_HALF * SIZE_QTR];
    let mut second_layer_bs = [0.0; SIZE_QTR];

    let mut third_layer_ws = [0.0; SIZE_QTR * SIZE_QTR];
    let mut third_layer_bs = [0.0; SIZE_QTR];

    let mut fourth_layer_ws = [0.0; SIZE_QTR * ACTIONS];
    let mut fourth_layer_bs = [0.0; ACTIONS];

    let path = format!("src/genes/{}/{}", for_model, nanoid::nanoid!());

    let mut data_json: Value = json!({
        "fitness" : 0.0,
        "first_layer": first_layer_bs.len(),
        "second_layer": second_layer_bs.len(),
        "third_layer": third_layer_bs.len(),
        "fourth_layer": fourth_layer_bs.len(),
    });

    for input in inputs.iter() {
        first_layer_bs
            .iter_mut()
            .for_each(|x| *x = apply_op(*x, &op));
        first_layer_ws
            .iter_mut()
            .for_each(|x| *x = apply_op(*x, &op));

        second_layer_bs
            .iter_mut()
            .for_each(|x| *x = apply_op(*x, &op));
        second_layer_ws
            .iter_mut()
            .for_each(|x| *x = apply_op(*x, &op));

        third_layer_bs
            .iter_mut()
            .for_each(|x| *x = apply_op(*x, &op));
        third_layer_ws
            .iter_mut()
            .for_each(|x| *x = apply_op(*x, &op));

        fourth_layer_bs
            .iter_mut()
            .for_each(|x| *x = apply_op(*x, &op));
        fourth_layer_ws
            .iter_mut()
            .for_each(|x| *x = apply_op(*x, &op));

        data_json[input] = json!({
            "first_layer_ws": first_layer_ws.to_vec(),
            "first_layer_bs": first_layer_bs.to_vec(),
            "second_layer_ws": second_layer_ws.to_vec(),
            "second_layer_bs": second_layer_bs.to_vec(),
            "third_layer_ws": third_layer_ws.to_vec(),
            "third_layer_bs": third_layer_bs.to_vec(),
            "fourth_layer_ws": fourth_layer_ws.to_vec(),
            "fourth_layer_bs": fourth_layer_bs.to_vec(),
        });
    }
    std::fs::write(path, serde_json::to_string_pretty(&data_json).unwrap()).unwrap();
}

pub fn create_new_ws_bs(nr: i64, for_model: &str) {
    let inputs = ["plant"];

    for _ in 0..nr {
        init_models_ws_bs(inputs, for_model, init_values_uniform);
        init_models_ws_bs(inputs, for_model, init_values_normal);
    }
}

pub fn truncate_gene_files(for_model: &str, max_files: i64) {
    let path = format!("src/genes/{}/", for_model);
    let files = get_files(&path);
    let mut file_table: Vec<(f64, String)> = Vec::new();

    for file in files {
        //let path = file.as_ref().unwrap().path();
        let path = file.path();
        let file = read_JSON(path.to_str().unwrap());
        let fitness = file["fitness"].as_f64().unwrap();

        let path_string = path.to_str().unwrap().to_string();
        file_table.push((fitness, path_string));
    }

    file_table.sort_by(|b, a| a.0.partial_cmp(&b.0).unwrap());

    for i in (max_files as i64)..file_table.len() as i64 {
        std::fs::remove_file(file_table[i as usize].1.clone()).unwrap();
    }
}

pub fn init_values_uniform(x: f32) -> f32 {
    let mut rng = rand::thread_rng();

    if rng.gen_range(0.0..1.0) > START_SPARCITY {
        let mut val = rng.gen_range(-(MAX_WEIGHT_BIAS as f32)..MAX_WEIGHT_BIAS as f32);
        return val;
    } else {
        return 0.0;
    }
}

pub fn init_values_normal(x: f32) -> f32 {
    let mut rng = rand::thread_rng();

    let mean = NORMAL_MEAN;
    let std_deviation = NORMAL_STDDEV;
    let normal = Normal::new(mean, std_deviation).unwrap();
    let random_number = rng.sample(normal);

    random_number as f32
}

pub fn choose_parents(for_model: &str) -> (String, String) {
    let mut rng = rand::thread_rng();

    let path = format!("src/genes/{}/", for_model);
    let files = get_files(&path);

    let path1 = files[rng.gen_range(0..files.len())].path();
    let path2 = files[rng.gen_range(0..files.len())].path();
    let path3 = files[rng.gen_range(0..files.len())].path();
    let path4 = files[rng.gen_range(0..files.len())].path();

    let parent1 = read_JSON(path1.to_str().unwrap());
    let parent2 = read_JSON(path2.to_str().unwrap());
    let parent3 = read_JSON(path3.to_str().unwrap());
    let parent4 = read_JSON(path4.to_str().unwrap());

    let parent1_fitness = parent1["fitness"].as_f64().unwrap();
    let parent2_fitness = parent2["fitness"].as_f64().unwrap();
    let parent3_fitness = parent3["fitness"].as_f64().unwrap();
    let parent4_fitness = parent4["fitness"].as_f64().unwrap();

    let max_fit: bool = if rng.gen_range(0.0..=1.0) < CHOOSE_MAX_FIT {
        true
    } else {
        false
    };

    let mut return1 = String::new();
    let mut return2 = String::new();

    if (parent1_fitness >= parent2_fitness) {
        if max_fit {
            return1 = path1.to_str().unwrap().to_string();
        } else {
            return1 = path2.to_str().unwrap().to_string();
        }
    } else {
        if max_fit {
            return1 = path2.to_str().unwrap().to_string();
        } else {
            return1 = path1.to_str().unwrap().to_string();
        }
    };
    if (parent3_fitness >= parent4_fitness) {
        if max_fit {
            return2 = path3.to_str().unwrap().to_string();
        } else {
            return2 = path4.to_str().unwrap().to_string();
        }
    } else {
        if max_fit {
            return2 = path4.to_str().unwrap().to_string();
        } else {
            return2 = path3.to_str().unwrap().to_string();
        }
    }
    (return1, return2)
}

pub fn read_JSON(path: &str) -> serde_json::Value {
    let mut data = {
        //let input = std::fs::read_to_string(path).unwrap();
        let input = match std::fs::read_to_string(path) {
            Ok(data) => data,
            Err(e) => {
                println!("Error: {}", e);
                println!("Path: {}", path);
                std::process::exit(1);
            }
        };
        match serde_json::from_str::<Value>(&input) {
            Ok(data) => data,
            Err(e) => {
                // remove dead files
                println!("Error: {}", e);
                println!("Path: {}", path);
                std::fs::remove_file(path).unwrap();
                std::process::exit(1);
            }
        }
        //serde_json::from_str::<Value>(&input).unwrap()
    };
    serde_json::from_value(data).expect("JSON was not well-formatted")
}

pub fn get_files(path: &str) -> Vec<std::fs::DirEntry> {
    let files: Vec<_> = std::fs::read_dir(path)
        .unwrap()
        .filter_map(|entry| {
            let file_name = entry
                .as_ref()
                .ok()?
                .file_name()
                .to_string_lossy()
                .into_owned();
            if !file_name.contains("DS_Store") {
                Some(entry.unwrap())
            } else {
                None
            }
        })
        .collect();
    files
}

pub fn generate_offspring(path1: String, path2: String, for_model: &str, mutate: bool) -> String {
    let mut rng = rand::thread_rng();

    let parent1 = read_JSON(path1.as_str());
    let parent2 = read_JSON(path2.as_str());

    let cut = rng.gen_range(0.0..1.0);

    let inputs = ["plant"];

    let mut child = json!({
        "fitness" : 0.0,
        "first_layer": parent1["first_layer"],
        "second_layer": parent1["second_layer"],
        "third_layer": parent1["third_layer"],
        "fourth_layer": parent1["fourth_layer"],
    });

    for input in inputs {
        let parent1_input = parent1[input].clone();
        let parent1_fc1_ws = parent1_input["first_layer_ws"].as_array().unwrap();
        let parent1_fc1_bs = parent1_input["first_layer_bs"].as_array().unwrap();
        let parent1_fc2_ws = parent1_input["second_layer_ws"].as_array().unwrap();
        let parent1_fc2_bs = parent1_input["second_layer_bs"].as_array().unwrap();
        let parent1_fc3_ws = parent1_input["third_layer_ws"].as_array().unwrap();
        let parent1_fc3_bs = parent1_input["third_layer_bs"].as_array().unwrap();
        let parent1_fc4_ws = parent1_input["fourth_layer_ws"].as_array().unwrap();
        let parent1_fc4_bs = parent1_input["fourth_layer_bs"].as_array().unwrap();

        let parent2_input = parent2[input].clone();
        let parent2_fc1_ws = parent2_input["first_layer_ws"].as_array().unwrap();
        let parent2_fc1_bs = parent2_input["first_layer_bs"].as_array().unwrap();
        let parent2_fc2_ws = parent2_input["second_layer_ws"].as_array().unwrap();
        let parent2_fc2_bs = parent2_input["second_layer_bs"].as_array().unwrap();
        let parent2_fc3_ws = parent2_input["third_layer_ws"].as_array().unwrap();
        let parent2_fc3_bs = parent2_input["third_layer_bs"].as_array().unwrap();
        let parent2_fc4_ws = parent2_input["fourth_layer_ws"].as_array().unwrap();
        let parent2_fc4_bs = parent2_input["fourth_layer_bs"].as_array().unwrap();

        let fc1_ws_len = parent1_fc1_ws.len();
        let fc1_bs_len = parent1_fc1_bs.len();
        let fc2_ws_len = parent1_fc2_ws.len();
        let fc2_bs_len = parent1_fc2_bs.len();
        let fc3_ws_len = parent1_fc3_ws.len();
        let fc3_bs_len = parent1_fc3_bs.len();
        let fc4_ws_len = parent1_fc4_ws.len();
        let fc4_bs_len = parent1_fc4_bs.len();

        // splice parent genes
        let mut child_fc1_ws = Vec::new();
        let mut cut_index = (fc1_ws_len as f32 * cut) as usize;
        child_fc1_ws.extend_from_slice(&parent1_fc1_ws[0..cut_index]);
        child_fc1_ws.extend_from_slice(&parent2_fc1_ws[cut_index..fc1_ws_len]);

        let mut child_fc1_bs = Vec::new();
        let mut cut_index = (fc1_bs_len as f32 * cut) as usize;
        child_fc1_bs.extend_from_slice(&parent1_fc1_bs[0..cut_index]);
        child_fc1_bs.extend_from_slice(&parent2_fc1_bs[cut_index..fc1_bs_len]);

        let mut child_fc2_ws = Vec::new();
        let mut cut_index = (fc2_ws_len as f32 * cut) as usize;
        child_fc2_ws.extend_from_slice(&parent1_fc2_ws[0..cut_index]);
        child_fc2_ws.extend_from_slice(&parent2_fc2_ws[cut_index..fc2_ws_len]);

        let mut child_fc2_bs = Vec::new();
        let mut cut_index = (fc2_bs_len as f32 * cut) as usize;
        child_fc2_bs.extend_from_slice(&parent1_fc2_bs[0..cut_index]);
        child_fc2_bs.extend_from_slice(&parent2_fc2_bs[cut_index..fc2_bs_len]);

        let mut child_fc3_ws = Vec::new();
        let mut cut_index = (fc3_ws_len as f32 * cut) as usize;
        child_fc3_ws.extend_from_slice(&parent1_fc3_ws[0..cut_index]);
        child_fc3_ws.extend_from_slice(&parent2_fc3_ws[cut_index..fc3_ws_len]);

        let mut child_fc3_bs = Vec::new();
        let mut cut_index = (fc3_bs_len as f32 * cut) as usize;
        child_fc3_bs.extend_from_slice(&parent1_fc3_bs[0..cut_index]);
        child_fc3_bs.extend_from_slice(&parent2_fc3_bs[cut_index..fc3_bs_len]);

        let mut child_fc4_ws = Vec::new();
        let mut cut_index = (fc4_ws_len as f32 * cut) as usize;
        child_fc4_ws.extend_from_slice(&parent1_fc4_ws[0..cut_index]);
        child_fc4_ws.extend_from_slice(&parent2_fc4_ws[cut_index..fc4_ws_len]);

        let mut child_fc4_bs = Vec::new();
        let mut cut_index = (fc4_bs_len as f32 * cut) as usize;
        child_fc4_bs.extend_from_slice(&parent1_fc4_bs[0..cut_index]);
        child_fc4_bs.extend_from_slice(&parent2_fc4_bs[cut_index..fc4_bs_len]);

        //mutate
        if mutate {
            child_fc1_ws = mutate_gene(child_fc1_ws);
            child_fc1_bs = mutate_gene(child_fc1_bs);

            child_fc2_ws = mutate_gene(child_fc2_ws);
            child_fc2_bs = mutate_gene(child_fc2_bs);

            child_fc3_ws = mutate_gene(child_fc3_ws);
            child_fc3_bs = mutate_gene(child_fc3_bs);

            child_fc4_ws = mutate_gene(child_fc4_ws);
            child_fc4_bs = mutate_gene(child_fc4_bs);
        }

        // write to json
        child[input] = json!({
            "first_layer_ws": child_fc1_ws,
            "first_layer_bs": child_fc1_bs,
            "second_layer_ws": child_fc2_ws,
            "second_layer_bs": child_fc2_bs,
            "third_layer_ws": child_fc3_ws,
            "third_layer_bs": child_fc3_bs,
            "fourth_layer_ws": child_fc4_ws,
            "fourth_layer_bs": child_fc4_bs,
        });
    }

    let path = format!("src/genes/{}/{}", for_model, nanoid::nanoid!());

    // todo uncomment
    std::fs::write(&path, serde_json::to_string_pretty(&child).unwrap()).unwrap();

    path
}

pub fn get_child_model(for_model: &str) -> (genAlgoNN, String) {
    let (parent1, parent2) = choose_parents(for_model);

    let child = generate_offspring(parent1, parent2, for_model, true);

    let child_model = genAlgoNN::new(child.clone());

    (child_model, child)
}

pub fn mutate_gene(mut gene: Vec<Value>) -> Vec<Value> {
    let mut rng = rand::thread_rng();

    let gen_length = gene.len();

    for mutations in 0..(gen_length as f32 * MUTATION_RATE) as i64 {
        let mut index = rng.gen_range(0..gen_length);
        let mut value = gene[index].as_f64().unwrap();

        value = value + rng.gen_range(-(MAX_WEIGHT_BIAS as f64)..MAX_WEIGHT_BIAS as f64);

        gene[index] = json!(value);
    }

    gene
}
impl genAlgoNN {
    pub fn new(path: String) -> genAlgoNN {
        let file = read_JSON(&path);

        let plant_values = file["plant"].clone();

        let vs = VarStore::new(tch::Device::Cpu);

        let mut plant_fc1 = nn::linear(
            vs.root(),
            SIZE_FULL as i64,
            SIZE_HALF as i64,
            Default::default(),
        );

        let mut plant_fc2 = nn::linear(
            vs.root(),
            SIZE_HALF as i64,
            SIZE_QTR as i64,
            Default::default(),
        );

        let mut plant_fc3 = nn::linear(
            vs.root(),
            SIZE_QTR as i64,
            SIZE_QTR as i64,
            Default::default(),
        );

        let mut plant_fc4 = nn::linear(
            vs.root(),
            SIZE_QTR as i64,
            ACTIONS as i64,
            Default::default(),
        );

        let mut plant_fc1_ws_tensor = Tensor::of_slice(
            &serde_json::from_value::<Vec<f32>>(plant_values["first_layer_ws"].clone()).unwrap(),
        )
        .reshape(&[144, 288]);
        let mut plant_fc1_bs_tensor = Tensor::of_slice(
            &serde_json::from_value::<Vec<f32>>(plant_values["first_layer_bs"].clone()).unwrap(),
        );
        let mut plant_fc2_ws_tensor = Tensor::of_slice(
            &serde_json::from_value::<Vec<f32>>(plant_values["second_layer_ws"].clone()).unwrap(),
        )
        .reshape(&[72, 144]);
        let mut plant_fc2_bs_tensor = Tensor::of_slice(
            &serde_json::from_value::<Vec<f32>>(plant_values["second_layer_bs"].clone()).unwrap(),
        );
        let mut plant_fc3_ws_tensor = Tensor::of_slice(
            &serde_json::from_value::<Vec<f32>>(plant_values["third_layer_ws"].clone()).unwrap(),
        )
        .reshape(&[72, 72]);
        let mut plant_fc3_bs_tensor = Tensor::of_slice(
            &serde_json::from_value::<Vec<f32>>(plant_values["third_layer_bs"].clone()).unwrap(),
        );
        let mut plant_fc4_ws_tensor = Tensor::of_slice(
            &serde_json::from_value::<Vec<f32>>(plant_values["fourth_layer_ws"].clone()).unwrap(),
        )
        .reshape(&[7, 72]);
        let mut plant_fc4_bs_tensor = Tensor::of_slice(
            &serde_json::from_value::<Vec<f32>>(plant_values["fourth_layer_bs"].clone()).unwrap(),
        );

        plant_fc1.ws = plant_fc1_ws_tensor;
        plant_fc1.bs = Some(plant_fc1_bs_tensor);
        plant_fc2.ws = plant_fc2_ws_tensor;
        plant_fc2.bs = Some(plant_fc2_bs_tensor);
        plant_fc3.ws = plant_fc3_ws_tensor;
        plant_fc3.bs = Some(plant_fc3_bs_tensor);
        plant_fc4.ws = plant_fc4_ws_tensor;
        plant_fc4.bs = Some(plant_fc4_bs_tensor);

        let plant = nn::seq()
            .add(plant_fc1)
            .add_fn(|xs| xs.relu())
            .add(plant_fc2)
            .add_fn(|xs| xs.relu())
            .add(plant_fc3)
            .add_fn(|xs| xs.relu())
            .add(plant_fc4)
            .add_fn(|xs| xs.softmax(-1, Kind::Float));

        genAlgoNN { plant }
    }
    pub fn forward(&self, p: &Tensor) -> Tensor {
        let mut plant = self.plant.forward(&p.view([1, SIZE_FULL as i64]));
        plant
    }
}
