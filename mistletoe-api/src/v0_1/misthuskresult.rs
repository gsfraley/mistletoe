use anyhow::anyhow;
use indexmap::IndexMap;
use serde::{Serialize, Deserialize};

pub type MistHuskResult = anyhow::Result<MistHuskOutput>;

pub fn serialize_result(result: MistHuskResult) -> Result<String, serde_yaml::Error> {
    serde_yaml::to_string(&MistHuskResultLayout::from(result))
}

pub fn deserialize_result(result_str: &str) -> Result<MistHuskResult, serde_yaml::Error> {
    Ok(serde_yaml::from_str::<MistHuskResultLayout>(result_str)?.into())
}

#[derive(Clone, PartialEq, Debug)]
pub struct MistHuskOutput {
    message: Option<String>,
    files: IndexMap<String, String>,
}

impl MistHuskOutput {
    pub fn new() -> Self {
        Self {
            message: None,
            files: IndexMap::new(),
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    pub fn with_file(mut self, filename: String, content: String) -> Self {
        self.files.insert(filename, content);
        self
    }
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct MistHuskResultLayout {
    apiVersion: String,
    kind: String,
    data: MistHuskResultLayoutData,
}

#[derive(Serialize, Deserialize)]
struct MistHuskResultLayoutData {
    result: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    files: IndexMap<String, String>,
}

impl From<MistHuskResult> for MistHuskResultLayout {
    fn from(result: MistHuskResult) -> Self {
        Self {
            apiVersion: "mistletoe.dev/v1alpha1".to_string(),
            kind: "MistHuskResult".to_string(),
            data: match result {
                Ok(output) => MistHuskResultLayoutData {
                    result: "Ok".to_string(),
                    message: output.message,
                    files: output.files,
                },
                Err(e) => MistHuskResultLayoutData {
                    result: "Err".to_string(),
                    message: Some(e.to_string()),
                    files: IndexMap::new(),
                },
            }
        }
    }
}

impl Into<MistHuskResult> for MistHuskResultLayout {
    fn into(self) -> MistHuskResult {
        match self.data.result.as_str() {
            "Ok" => MistHuskResult::Ok(MistHuskOutput {
                message: self.data.message,
                files: self.data.files,
            }),
            "Err" => MistHuskResult::Err(match self.data.message {
                Some(message) => anyhow!(message),
                None => anyhow!("module failed without a message"),
            }),
            s => MistHuskResult::Err(anyhow!("module result format error: `data.result` must either be \"Ok\" or \"Err\", found {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_misthuskresult_ok() {
        let expected_yaml = indoc!{"
            apiVersion: mistletoe.dev/v1alpha1
            kind: MistHuskResult
            data:
              result: Ok
              message: 'warning: nothing went wrong'
              files:
                namespace.yaml: |
                  apiVersion: v1
                  kind: Namespace
                  metadata:
                    name: my-namespace
        "};

        let misthuskoutput = MistHuskOutput::new()
            .with_message("warning: nothing went wrong".to_string())
            .with_file("namespace.yaml".to_string(), indoc!("
                apiVersion: v1
                kind: Namespace
                metadata:
                  name: my-namespace
            ").to_string());

        let yaml = serialize_result(Ok(misthuskoutput.clone())).unwrap();
        assert_eq!(expected_yaml, yaml);

        let misthuskresult_parsed = deserialize_result(&yaml).unwrap();
        assert_eq!(misthuskoutput, misthuskresult_parsed.unwrap());
    }

    #[test]
    fn test_misthuskresult_err() {
        let expected_yaml: &str = indoc!{"
            apiVersion: mistletoe.dev/v1alpha1
            kind: MistHuskResult
            data:
              result: Err
              message: 'error: something went wrong'
        "};

        let err_string = "error: something went wrong";
        let yaml = serialize_result(Err(anyhow!(err_string.to_string()))).unwrap();
        assert_eq!(expected_yaml, yaml);

        let misthuskresult_parsed = deserialize_result(&yaml).unwrap();
        assert_eq!(err_string, misthuskresult_parsed.err().unwrap().to_string());
    }
}
