extern crate mistletoe_api;

use std::sync::atomic::AtomicPtr;
use mistletoe_api::v0_1::MistResult;
use mistletoe_macros::misthusk_headers;

use indexmap::IndexMap;
use indoc::formatdoc;
use once_cell::sync::Lazy;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

misthusk_headers! {"
  name: example-nginx
  labels:
    mistletoe.dev/group: mistletoe-examples
"}

#[derive(Deserialize)]
struct InputConfig {
    name: String,
    namespace: String,
}

pub fn generate(input: &str) -> String {
    let input_config: InputConfig = serde_yaml::from_str(input).unwrap();

    let mut files = IndexMap::new();
    files.insert("deployment.yaml".to_string(), formatdoc!{"
        ---
        apiVersion: apps/v1
        kind: Deployment
        metadata:
          name: {0}
          namespace: {1}
          labels:
            app: {0}
        spec:
          replicas: 1
          selector:
            matchLabels:
              app: {0}
          template:
            metadata:
              labels:
                app: {0}
            spec:
              containers:
              - image: nginx
                name: nginx
                ports:
                - name: http
                  containerPort: 80
    ", input_config.name, input_config.namespace});
    files.insert("service.yaml".to_string(), formatdoc!{"
        ---
        apiVersion: v1
        kind: Service
        metadata:
          name: {0}
          namespace: {1}
          labels:
            app: {0}
        spec:
          selector:
            app: {0}
          ports:
          - name: http
            port: 80
            protocol: TCP
            targetPort: http
    ", input_config.name, input_config.namespace});

    serde_yaml::to_string(&MistResult::Ok { files }).unwrap()
}
