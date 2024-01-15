use super::*;

use k8s_openapi::api::core::v1::Secret;

pub fn generate_secret(inputs: &Inputs) -> MistResult {
    let mut secret_values = IndexMap::new();

    if let Some(user) = inputs.users.get("root") {
        match &user.auth {
            UserAuthValue::Hash { hash } => {
                secret_values.insert("MARIADB_ROOT_PASSWORD_HASH".to_string(), hash.clone());
            },
            UserAuthValue::Password { password } => {
                secret_values.insert("MARIADB_ROOT_PASSWORD".to_string(), password.clone());
            },
            UserAuthValue::Random => {
                secret_values.insert("MARIADB_RANDOM_ROOT_PASSWORD".to_string(), "true".to_string());
            },
        }
    } else {
        secret_values.insert("MARIADB_RANDOM_ROOT_PASSWORD".to_string(), "true".to_string());
    }

    let mut metadata = generate_base_metadata(inputs);
    metadata.name = Some(format!("{}-secret-env", metadata.name.unwrap()));
    metadata.labels.as_mut().unwrap()
        .insert("app.kubernetes.io/component".to_string(), "secret-env".to_string());

    let secret = Secret {
        metadata,
        string_data: Some(secret_values.into_iter().collect()),
        ..Default::default()
    };

    let output = MistOutput::new()
        .with_file("secret.yaml".to_string(), serde_yaml::to_string(&secret)?);

    Ok(output)
}

