use super::*;

use base64::prelude::*;
use mistletoe_bind::random::get_random_bytes;
use k8s_openapi::api::core::v1::Secret;
use sha1::Digest;
use sha1::Sha1;

pub fn generate_secrets(inputs: &Inputs) -> MistResult {
    let output = MistOutput::new()
        .with_files_from_map(generate_secret_env(inputs)?.get_files())
        .with_files_from_map(generate_secret_scripts(inputs)?.get_files());

    Ok(output)
}

fn generate_secret_env(inputs: &Inputs) -> MistResult {
    let mut secret_values = IndexMap::new();

    if inputs.auto_upgrade {
        secret_values.insert("MARIADB_AUTO_UPGRADE".to_string(), "true".to_string());
    }

    if let Some(user) = inputs.users.get("root") {
        match &user.auth {
            Some(UserAuthValue::Hash { hash }) => {
                secret_values.insert("MARIADB_ROOT_PASSWORD_HASH".to_string(), hash.clone());
            },
            Some(UserAuthValue::Password { password }) => {
                secret_values.insert("MARIADB_ROOT_PASSWORD_HASH".to_string(), hash_into_mysql_native_password(password)?);
            },
            None => {
                secret_values.insert("MARIADB_RANDOM_ROOT_PASSWORD".to_string(), "true".to_string());
            },
        }
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
        .with_file("secret_env.yaml".to_string(), serde_yaml::to_string(&secret)?);

    Ok(output)
}

fn generate_secret_scripts(inputs: &Inputs) -> MistResult {
    let mut metadata = generate_base_metadata(inputs);
    metadata.name = Some(format!("{}-secret-scripts", metadata.name.unwrap()));
    metadata.labels.as_mut().unwrap()
        .insert("app.kubernetes.io/component".to_string(), "secret-scripts".to_string());

    let mut scripts = IndexMap::new();
    scripts.insert("001-users.sql".to_string(), generate_users_sql(inputs)?);

    let secret = Secret {
        metadata,
        string_data: Some(scripts.into_iter().collect()),
        ..Default::default()
    };

    let output = MistOutput::new()
        .with_file("secret_scripts.yaml".to_string(), serde_yaml::to_string(&secret)?);

    Ok(output)
}

fn generate_users_sql(inputs: &Inputs) -> anyhow::Result<String> {
    let mut sql = String::new();

    for (username, user) in &inputs.users {
        if username == "root" {
            continue;
        }

        sql.push_str(&format!("CREATE USER '{}'@'%' IDENTIFIED BY PASSWORD '{}';\n", username, match &user.auth {
            Some(UserAuthValue::Hash { hash }) => hash.clone(),
            Some(UserAuthValue::Password { password }) => hash_into_mysql_native_password(password)?,
            None => hash_into_mysql_native_password(&generate_random_base64(32))?,
        }));
    }

    Ok(sql)
}

fn hash_into_mysql_native_password(password: &str) -> anyhow::Result<String> {
    let mut hasher1 = Sha1::new();
    hasher1.update(password.as_bytes());
    let mut hasher2 = Sha1::new();
    hasher2.update(&hasher1.finalize());
    
    Ok(format!("*{}", base16::encode_upper(&hasher2.finalize()[..]).to_uppercase()))
}

fn generate_random_base64(len: usize) -> String {
    let bytes = get_random_bytes(len);
    BASE64_STANDARD.encode(&bytes)
}
