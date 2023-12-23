use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "value")]
pub enum MistOutput {
    Ok(String),
    Err(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_output_ok() {
        let ok_case_value = indoc! {"
        ---
        apiVersion: v1
        kind: Namespace
        metadata:
          name: my-namespace
        "};
        
        let ok_case = MistOutput::Ok(ok_case_value.into());

        let expected_ok_case = indoc! {"
        type: Ok
        value: |
          ---
          apiVersion: v1
          kind: Namespace
          metadata:
            name: my-namespace
        "};

        assert_eq!(expected_ok_case.to_owned(), serde_yaml::to_string(&ok_case).unwrap());
    }

    #[test]
    fn test_output_err() {
        let err_case_value = "failure message";
        
        let err_case = MistOutput::Err(err_case_value.into());

        let expected_err_case = indoc! {"
        type: Err
        value: failure message
        "};

        assert_eq!(expected_err_case.to_owned(), serde_yaml::to_string(&err_case).unwrap());
    }
}
