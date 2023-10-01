use crate::{ACTIONS, NN_RAYS, NN_RAY_DR, NN_RAY_LEN, N_TYPES, MAX_WEIGHT_BIAS, MUTATION_RATE, START_SPARCITY};

const SIZE_FULL: i64 = (NN_RAYS * NN_RAY_LEN) as i64;
const SIZE_HALF: i64 = (SIZE_FULL as f64 * 0.5) as i64;
const SIZE_QTR: i64 = (SIZE_FULL as f64 * 0.25) as i64;



pub fn forward(signal: [[[f32; NN_RAY_LEN]; NN_RAYS]; N_TYPES]) -> [f32; ACTIONS] {
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
}
