use serde::{Serialize, Deserialize, Serializer, Deserializer, de::DeserializeOwned};

#[derive(Clone, PartialEq, Debug)]
pub struct MistHuskInput {
    pub data: serde_yaml::Mapping,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct MistHuskInputLayout {
    apiVersion: String,
    kind: String,
    data: serde_yaml::Mapping,
}

impl From<MistHuskInput> for MistHuskInputLayout {
    fn from(mhi: MistHuskInput) -> MistHuskInputLayout {
        MistHuskInputLayout {
            apiVersion: "mistletoe.dev/v1alpha1".to_string(),
            kind: "MistHuskInput".to_string(),
            data: mhi.data,
        }
    }
}

impl Into<MistHuskInput> for MistHuskInputLayout {
    fn into(self) -> MistHuskInput {
        MistHuskInput {
            data: self.data,
        }
    }
}

impl Serialize for MistHuskInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        MistHuskInputLayout::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MistHuskInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let mri = MistHuskInputLayout::deserialize(deserializer)?;
        Ok(mri.into())
    }
}

impl MistHuskInput {
    pub fn try_into_data<'a, T>(&self) -> Result<T, serde_yaml::Error>
    where
        T: DeserializeOwned
    {
        let serialized = serde_yaml::to_string(&self.data)?;
        let value = serde_yaml::from_str(&serialized)?;
        serde_yaml::from_value(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_misthuskinput() {
        let expected_yaml = indoc! {"
            apiVersion: mistletoe.dev/v1alpha1
            kind: MistHuskInput
            data:
              name: my-nginx
              namespace: my-namespace
        "};

        let mut data = serde_yaml::Mapping::new();
        data.insert("name".into(), "my-nginx".into());
        data.insert("namespace".into(), "my-namespace".into());

        let misthuskinput = MistHuskInput { data };

        let yaml = serde_yaml::to_string(&misthuskinput).unwrap();
        assert_eq!(expected_yaml, yaml);

        let misthuskinput_parsed = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(misthuskinput, misthuskinput_parsed);
    }
}
