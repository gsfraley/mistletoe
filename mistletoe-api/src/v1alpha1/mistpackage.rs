use indexmap::IndexMap;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

/// Info about the package that it returns when queried about.
///
/// This contains a name and some optional labels.
#[derive(Clone, PartialEq, Debug)]
pub struct MistPackage {
    /// Name of the package.
    pub name: String,

    /// Package labels.
    /// 
    /// These can be whatever the package maintainer decides to attach, though
    /// there are some labels with significance that Mistletoe can use to provide
    /// additional information about the package to the end-user, notably
    /// `mistletoe.dev/group`.
    pub labels: Option<IndexMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MistPackageLayout {
    api_version: String,
    kind: String,
    metadata: MistPackageLayoutMetadata,
}

#[derive(Serialize, Deserialize)]
struct MistPackageLayoutMetadata {
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    labels: Option<IndexMap<String, String>>,
}

impl From<MistPackage> for MistPackageLayout {
    fn from(mhp: MistPackage) -> MistPackageLayout {
        MistPackageLayout {
            api_version: "mistletoe.dev/v1alpha1".to_string(),
            kind: "MistPackage".to_string(),
            metadata: MistPackageLayoutMetadata {
                name: mhp.name,
                labels: mhp.labels,
            },
        }
    }
}

impl Into<MistPackage> for MistPackageLayout {
    fn into(self) -> MistPackage {
        MistPackage {
            name: self.metadata.name,
            labels: self.metadata.labels,
        }
    }
}

impl Serialize for MistPackage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        MistPackageLayout::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MistPackage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let mrl = MistPackageLayout::deserialize(deserializer)?;
        Ok(mrl.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_mistpackage() {
        let expected_yaml = indoc! {"
            apiVersion: mistletoe.dev/v1alpha1
            kind: MistPackage
            metadata:
              name: example-nginx
              labels:
                mistletoe.dev/group: mistletoe-examples"};

        let mut labels = IndexMap::new();
        labels.insert("mistletoe.dev/group".to_string(), "mistletoe-examples".to_string());

        let mistpackage = MistPackage {
            name: "example-nginx".to_string(),
            labels: Some(labels),
        };

        let yaml = serde_yaml::to_string(&mistpackage).unwrap();
        assert_eq!(expected_yaml, yaml, "left:\n{expected_yaml}\nright:\n{expected_yaml}");

        let mistpackage_parsed = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(mistpackage, mistpackage_parsed);
    }
}
