use std::collections::HashMap;

use serde::{Serialize, Serializer, Deserialize, Deserializer};

pub enum MistResult {
    Ok {
        files: HashMap<String, String>,
    },
    Err {
        message: String,
    }
}

impl Serialize for MistResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map
    }
}
