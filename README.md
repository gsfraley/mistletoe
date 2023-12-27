![Mistletoe logo](logo.jpg)

# Mistletoe (WIP) - Polyglot Kubernetes Package Manager

**Mistletoe** is a Kubernetes package manager that goes about things in a different way than the rest of the ecosystem.  There are some great package managers already out there that we'll have to catch up to, notably Helm.  The best way to distinguish the two is to talk about what the packages themselves look like.

## Run WASM, Get Kubernetes YAML

[Helm](https://github.com/helm/helm) Charts are YAML templates in the Golang templating language.  It's very similar to writing Kubernetes resources themselves -- you write the resources, receiving input configuration from the engine that you can use in the template logic.

**Mistletoe** packages are WebAssembly modules with one entrypoint.  The package developer writes a function in their language of choice that takes in a YAML input string and outputs Kubernetes resource YAML.  Essentially, the developer is welcome to do *anything* they want assuming it fits in the sandbox.

## What does it look like for users?

The command-line interface is relatively straightforward, and some examples are shown below.  More and better documentation will be provided soon, but it's simple enough to use at this point in time without much explanation.

```sh
# Show basic subcommands and usage.
mistctl --help


# Generate Kubernetes YAML output from a package.  Does not install, only outputs the resources.
mistctl generate my-nginx-installation \
    --package file:./examples/basic-nginx/pkg/mistletoe_example_basic_nginx_bg.wasm \
    --set "namespace: my-namespace" --output dir=./tmp/out
```

Currently only generation of output YAML is supported, but lifecycle management and installation to clusters directly from the utility are on the roadmap in the near future.

## What does it look like for developers?

That depends on what the developer wants to do.  It turns out that pretty much any language that has a YAML parsing library works almost perfectly.  Ultimately, all we're doing is taking a string and returning a string.

To give a more concrete example, here's a simple Rust package:

```rust
misthusk_headers! {"
  name: example-namespace
  labels:
    mistletoe.dev/group: mistletoe-examples
"}

#[derive(Deserialize)]
struct InputConfig {
    name: String,
}

pub fn generate(input_config: InputConfig) -> MistHuskResult {
    let output = MistHuskOutput::new()
        .with_file("namespace.yaml".to_string(), formatdoc!{"
            ---
            apiVersion: apps/v1
            kind: Namespace
            metadata:
              name: {0}
        ", input_config.name});

    Ok(output)
}
```

The above example takes a single-line input YAML, e.g. `name: my-namespace`, and spits out a Namespace with that name.  It makes use of some library support I've provided for Rust, and I'll be improving how that looks as time goes on, but its worth noting that there's not a whole lot I *can* provide given how simple it already is.

## Where are things at now?

The most basic functionality is done.  You can pretty easily write packages for Mistletoe, and Mistletoe can call them to generate installation YAML.  There are some glaring holes in the picture yet, so here are some must-haves and want-haves to be checked off before I start lobbying for usage of the toolset:

* Add support for pulling packages from a registry.  Right now packages must be downloaded to the local filesystem for use which is going to be a pain for any real use.
* Implement installation to clusters directly from the command.  We can generate the installation YAML now, but there isn't support for pushing it directly to the cluster.  We'll also need to consider lifecycle management and labeling of installed resources for later uninstallation.
* Want: increase the library of packages, packaging tools, and frameworks for writing and using Mistletoe packages.  We'll want TypeScript and maybe Go support, and examples of interfacing with other toolsets like cdk8s.
* Want: alternate package engines.  If we can package the relevant parts of Helm, or even the entirety of it, into an engine we can ship with the utility, then we could support Helm Charts as if they were any other Mistletoe package.
