use indexmap::IndexMap;
use serde::{de, Serialize, Deserialize, Serializer, Deserializer};

#[derive(Clone, PartialEq, Debug)]
pub enum MistResult {
    Ok { files: MistResultFiles },
    Err { message: String },
}

#[derive(Clone, PartialEq, Debug)]
pub struct MistResultFiles {
    files: IndexMap<String, String>,
}

impl MistResultFiles {
    pub fn new() -> Self {
        Self {
            files: IndexMap::new(),
        }
    }

    pub fn add_file(mut self, name: String, content: String) -> Self {
        self.files.insert(name, content);
        self
    }
}

impl Into<MistResult> for MistResultFiles {
    fn into(self) -> MistResult {
        MistResult::Ok {
            files: self
        }
    }
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct MistResultLayout {
    apiVersion: String,
    kind: String,
    data: MistResultLayoutData,
}

#[derive(Serialize, Deserialize)]
struct MistResultLayoutData {
    result: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    files: Option<IndexMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl From<MistResult> for MistResultLayout {
    fn from(mr: MistResult) -> Self {
        let mrld = match mr {
            MistResult::Ok { files } => MistResultLayoutData {
                result: "Ok".to_string(),
                files: Some(files.files),
                message: None,
            },
            MistResult::Err { message } => MistResultLayoutData {
                result: "Err".to_string(),
                files: None,
                message: Some(message),
            },
        };

        Self {
            apiVersion: "mistletoe.dev/v1alpha1".to_string(),
            kind: "MistResult".to_string(),
            data: mrld,
        }
    }
}

impl TryInto<MistResult> for MistResultLayout {
    type Error = String;

    fn try_into(self) -> Result<MistResult, Self::Error> {
        match self.data.result.as_str() {
            "Ok" => {
                match self.data.files {
                    Some(files) => Ok(MistResult::Ok { files: MistResultFiles { files } }),
                    None => Err("\"files\" must be defined on \"Ok\" results".to_string()),
                }
            },
            "Err" => {
                match self.data.message {
                    Some(message) => Ok(MistResult::Err { message }),
                    None => Err("\"message\" must be defined on \"Err\" results".to_string()),
                }
            },
            result => Err(format!("unexpected \"result\" value {}", result))
        }
    }
}

impl Serialize for MistResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        MistResultLayout::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MistResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let mrl = MistResultLayout::deserialize(deserializer)?;
        Ok(mrl.try_into().map_err(|e| de::Error::custom(e))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use serde_yaml;

    #[test]
    fn test_mistresult_err() {
        let expected_yaml = indoc! {"
            apiVersion: mistletoe.dev/v1alpha1
            kind: MistResult
            data:
              result: Err
              message: something failed
        "};

        let mistresult = MistResult::Err { message: "something failed".to_string() };
        let yaml = serde_yaml::to_string(&mistresult).unwrap();
        assert_eq!(expected_yaml, yaml);

        let mistresult_parsed = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(mistresult, mistresult_parsed);
    }

    #[test]
    fn test_mistresult_ok() {
        let expected_yaml = indoc! {"
            apiVersion: mistletoe.dev/v1alpha1
            kind: MistResult
            data:
              result: Ok
              files:
                namespace.yaml: |
                  apiVersion: v1
                  kind: Namespace
                  metadata:
                    name: my-namespace
                resources/service.yaml: |
                  apiVersion: v1
                  kind: Service
                  metadata:
                    name: my-nginx
                  spec:
                    type: LoadBalancer
                    selector:
                      app: my-nginx
                    ports:
                    - name: http
                      port: 80
                      containerPort: http
        "};

        let files = MistResultFiles::new()
            .add_file("namespace.yaml".to_string(), indoc! {"
                apiVersion: v1
                kind: Namespace
                metadata:
                  name: my-namespace
            "}.to_string())
            .add_file("resources/service.yaml".to_string(), indoc! {"
                apiVersion: v1
                kind: Service
                metadata:
                  name: my-nginx
                spec:
                  type: LoadBalancer
                  selector:
                    app: my-nginx
                  ports:
                  - name: http
                    port: 80
                    containerPort: http
            "}.to_string());

        let mistresult = MistResult::Ok { files };
        let yaml = serde_yaml::to_string(&mistresult).unwrap();
        assert_eq!(expected_yaml, yaml);

        let mistresult_parsed = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(mistresult, mistresult_parsed);
    }
}
