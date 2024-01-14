mod mistinput;
pub use mistinput::MistInput;

mod mistpackage;
pub use mistpackage::MistPackage;

mod mistresult;
pub use mistresult::{MistResult, MistOutput, serialize_result, deserialize_result};

use serde::Serialize;
use serde::de::DeserializeOwned;

pub fn yaml_transmute<I, O>(input: I) -> Result<O, serde_yaml::Error>
where
    I: Serialize,
    O: DeserializeOwned
{
    serde_yaml::from_value(serde_yaml::to_value(input)?)
}
