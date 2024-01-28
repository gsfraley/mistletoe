use serde::{Serialize, Deserialize, Serializer, Deserializer};

/// This is the input that is passed from the engine to the package for processing.
/// There is only one field, and that is the freeform `data` field that can take
/// any sort of map data the end-user wishes to provide to the package.
#[derive(Clone, PartialEq, Debug)]
pub struct MistInput {
    /// Freeform data field.
    pub data: serde_yaml::Mapping,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MistInputLayout {
    api_version: String,
    kind: String,
    data: serde_yaml::Mapping,
}

impl From<MistInput> for MistInputLayout {
    fn from(mhi: MistInput) -> MistInputLayout {
        MistInputLayout {
            api_version: "mistletoe.dev/v1alpha1".to_string(),
            kind: "MistInput".to_string(),
            data: mhi.data,
        }
    }
}

impl Into<MistInput> for MistInputLayout {
    fn into(self) -> MistInput {
        MistInput {
            data: self.data,
        }
    }
}

impl Serialize for MistInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        MistInputLayout::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MistInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let mri = MistInputLayout::deserialize(deserializer)?;
        Ok(mri.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_mistinput() {
        let expected_yaml = indoc!("
            apiVersion: mistletoe.dev/v1alpha1
            kind: MistInput
            data:
              name: my-nginx
              namespace: my-namespace
            ");

        let mut data = serde_yaml::Mapping::new();
        data.insert("name".into(), "my-nginx".into());
        data.insert("namespace".into(), "my-namespace".into());

        let mistinput = MistInput { data };

        let yaml = serde_yaml::to_string(&mistinput).unwrap();
        assert_eq!(expected_yaml, yaml);

        let mistinput_parsed = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(mistinput, mistinput_parsed);
    }
}
