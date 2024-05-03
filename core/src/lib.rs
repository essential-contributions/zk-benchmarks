use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Outputs {
    pub hash_output: [u8; 32],
    pub num_iterations: u32,
}
