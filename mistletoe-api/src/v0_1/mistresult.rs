use anyhow::anyhow;
use indexmap::IndexMap;
use serde::{Serialize, Deserialize};

/// This is the type that's returned from the module to the engine.
/// 
/// The error case can be any error (thanks to "anyhow"), and the success case is a
/// [MistOutput]. It can be serialized with [serialize_result] and deserialized with
/// [deserialize_result] (this is because we don't own the [Result] type).
pub type MistResult = anyhow::Result<MistOutput>;

/// Serialized the result to a YAML string.
pub fn serialize_result(result: MistResult) -> Result<String, serde_yaml::Error> {
    serde_yaml::to_string(&MistResultLayout::from(result))
}

/// Deserialized the result from a YAML string.
pub fn deserialize_result(result_str: &str) -> MistResult {
    serde_yaml::from_str::<MistResultLayout>(result_str)?.into()
}

/// This is the successful output of a module.
#[derive(Clone, PartialEq, Debug)]
pub struct MistOutput {
    message: Option<String>,

    /// This is the map of output files.
    /// 
    /// Each key is a relative path in the output directory that the content will be
    /// rendered to, and the keys are the content.
    files: IndexMap<String, String>,
}

impl MistOutput {
    /// Creates a new output object.
    pub fn new() -> Self {
        Self {
            message: None,
            files: IndexMap::new(),
        }
    }

    /// Sets the optional message in the output that the module can print out, in case
    /// there's additional info the module wishes to provide to the end user.
    pub fn set_message(&mut self, message: String) {
        self.message = Some(message);
    }


    /// Sets the optional message in the output that the module can print out, in case
    /// there's additional info the module wishes to provide to the end user.
    /// 
    /// This is the same as `set_message` but can be used in chaining.
    pub fn with_message(mut self, message: String) -> Self {
        self.set_message(message);
        self
    }

    /// Adds a file to the output that will be rendered to the output directory.
    pub fn add_file(&mut self, filename: String, content: String) {
        self.files.insert(filename, content);
    }

    /// Adds a file to the output that will be rendered to the output directory.
    /// 
    /// This is the same as `add_file` but can be used in chaining.
    pub fn with_file(mut self, filename: String, content: String) -> Self {
        self.add_file(filename, content);
        self
    }

    /// Retrieves the attached message on this object.
    pub fn get_message(&self) -> &Option<String> {
        &self.message
    }

    /// Retrieves the map of files stored by this object.
    pub fn get_files(&self) -> &IndexMap<String, String> {
        &self.files
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
    message: Option<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    files: IndexMap<String, String>,
}

impl From<MistResult> for MistResultLayout {
    fn from(result: MistResult) -> Self {
        Self {
            apiVersion: "mistletoe.dev/v1alpha1".to_string(),
            kind: "MistResult".to_string(),
            data: match result {
                Ok(output) => MistResultLayoutData {
                    result: "Ok".to_string(),
                    message: output.message,
                    files: output.files,
                },
                Err(e) => MistResultLayoutData {
                    result: "Err".to_string(),
                    message: Some(e.to_string()),
                    files: IndexMap::new(),
                },
            }
        }
    }
}

impl Into<MistResult> for MistResultLayout {
    fn into(self) -> MistResult {
        match self.data.result.as_str() {
            "Ok" => MistResult::Ok(MistOutput {
                message: self.data.message,
                files: self.data.files,
            }),
            "Err" => MistResult::Err(match self.data.message {
                Some(message) => anyhow!(message),
                None => anyhow!("module failed without a message"),
            }),
            s => MistResult::Err(anyhow!("module result format error: `data.result` must either be \"Ok\" or \"Err\", found {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_mistresult_ok() {
        let expected_yaml = indoc!{"
            apiVersion: mistletoe.dev/v1alpha1
            kind: MistResult
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

        let mistoutput = MistOutput::new()
            .with_message("warning: nothing went wrong".to_string())
            .with_file("namespace.yaml".to_string(), indoc!("
                apiVersion: v1
                kind: Namespace
                metadata:
                  name: my-namespace
            ").to_string());

        let yaml = serialize_result(Ok(mistoutput.clone())).unwrap();
        assert_eq!(expected_yaml, yaml);

        let mistresult_parsed = deserialize_result(&yaml).unwrap();
        assert_eq!(mistoutput, mistresult_parsed);
    }

    #[test]
    fn test_mistresult_err() {
        let expected_yaml: &str = indoc!{"
            apiVersion: mistletoe.dev/v1alpha1
            kind: MistResult
            data:
              result: Err
              message: 'error: something went wrong'
        "};

        let err_string = "error: something went wrong";
        let yaml = serialize_result(Err(anyhow!(err_string.to_string()))).unwrap();
        assert_eq!(expected_yaml, yaml);

        let mistresult_parsed = deserialize_result(&yaml);
        assert_eq!(err_string, mistresult_parsed.err().unwrap().to_string());
    }
}
