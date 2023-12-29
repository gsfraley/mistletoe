use indexmap::IndexMap;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

/// Info about the package the module returns when queried about.
///
/// This contains a name, some optional labels, as well as the exported names of
/// a few significant functions.
#[derive(Clone, PartialEq, Debug)]
pub struct MistPackage {
    /// Name of the package.
    pub name: String,

    /// Package labels.
    /// 
    /// These can be whatever the package maintainer decides to attach, though
    /// there are some labels with significance that Mistletoe can use to provide
    /// additional information about the module to the end-user, notably
    /// `mistletoe.dev/group`.
    pub labels: Option<IndexMap<String, String>>,

    /// The generate function/the main entrypoint to the module.
    /// 
    /// This is called with an input YAML string and then returns a Kubernetes
    /// resource output YAML string.
    /// 
    /// The signature of this function is \[i32, i32] -> \[i32], where the provided
    /// parameters are a (pointer to a buffer, length of buffer), and the returned
    /// output is a pointer to another fat pointer, where the fat pointer starts with
    /// 4 bytes of pointer to an output buffer, followed by 4 bytes of length of the buffer.
    pub function_generate: Option<String>,

    /// The function inside the module used by the engine to allocate data into its
    /// memory.
    /// 
    /// The signature of this function is \[i32] -> \[i32], where the provided parameter
    /// is a length in bytes to allocate in the memory, and the returned parameter is
    /// a pointer to the location in memory.
    pub function_alloc: Option<String>,

    /// The function inside the module for the engine to use to clean up/deallocate
    /// data in its memory.
    /// 
    /// The signature of this function is \[i32, i32], where the provided parameters are
    /// (pointer to a buffer, length of buffer)
    pub function_dealloc: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct MistPackageLayout {
    apiVersion: String,
    kind: String,
    metadata: MistPackageLayoutMetadata,
    spec: MistPackageLayoutSpec,
}

#[derive(Serialize, Deserialize)]
struct MistPackageLayoutMetadata {
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    labels: Option<IndexMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
struct MistPackageLayoutSpec {
    functions: MistPackageLayoutSpecFunctions,
}

#[derive(Serialize, Deserialize)]
struct MistPackageLayoutSpecFunctions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    generate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    alloc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dealloc: Option<String>,
}

impl From<MistPackage> for MistPackageLayout {
    fn from(mhp: MistPackage) -> MistPackageLayout {
        MistPackageLayout {
            apiVersion: "mistletoe.dev/v1alpha1".to_string(),
            kind: "MistPackage".to_string(),
            metadata: MistPackageLayoutMetadata {
                name: mhp.name,
                labels: mhp.labels,
            },
            spec: MistPackageLayoutSpec {
                functions: MistPackageLayoutSpecFunctions {
                    generate: mhp.function_generate,
                    alloc: mhp.function_alloc,
                    dealloc: mhp.function_dealloc,
                }
            }
        }
    }
}

impl Into<MistPackage> for MistPackageLayout {
    fn into(self) -> MistPackage {
        MistPackage {
            name: self.metadata.name,
            labels: self.metadata.labels,

            function_generate: self.spec.functions.generate,
            function_alloc: self.spec.functions.alloc,
            function_dealloc: self.spec.functions.dealloc,
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
                mistletoe.dev/group: mistletoe-examples
            spec:
              functions:
                generate: __mistletoe_generate
                alloc: __mistletoe_alloc
                dealloc: __mistletoe_dealloc
        "};

        let mut labels = IndexMap::new();
        labels.insert("mistletoe.dev/group".to_string(), "mistletoe-examples".to_string());

        let mistpackage = MistPackage {
            name: "example-nginx".to_string(),
            labels: Some(labels),

            function_generate: Some("__mistletoe_generate".to_string()),
            function_alloc: Some("__mistletoe_alloc".to_string()),
            function_dealloc: Some("__mistletoe_dealloc".to_string()),
        };

        let yaml = serde_yaml::to_string(&mistpackage).unwrap();
        assert_eq!(expected_yaml, yaml, "left:\n{expected_yaml}\nright:\n{expected_yaml}");

        let mistpackage_parsed = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(mistpackage, mistpackage_parsed);
    }
}
