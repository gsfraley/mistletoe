![Mistletoe logo](logo.png)

# Mistletoe (WIP) - the Polyglot Kubernetes Package Manager

[Site](https://mistletoe.dev/) | [Blog](https://mistletoe.dev/blog/) | [Book](https://mistletoe.dev/book/)

**Mistletoe** is a Kubernetes package manager that goes about things in a different way than the rest of the
ecosystem. It's built around a runtime where the packages are **WebAssembly** modules:

```rust
mistletoe_package! {"
  name: namespace-example
  labels:
    mistletoe.dev/group: mistletoe-examples
"}

#[derive(Deserialize)]
pub struct Inputs {
    name: String,
}

pub fn generate(inputs: Inputs) -> MistResult {
    let name = inputs.name;

    let output = MistOutput::new()
        .with_file("namespace.yaml".to_string(), formatdoc!("
            apiVersion: v1
            kind: Namespace
            metadata:
              name: {name}
        "));

    Ok(output)
}
```

**The above is a simple package written in Rust.  If you run it with:**

```sh
mistctl generate my-namespace -p mistletoe/examples/namespace-example:0.1.1
```

**You get the following YAML back:**

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: my-namespace
```

This is one of a nearly-unlimited number of ways to write a package.  ***All you need is a language with a
YAML parser that can compile to WebAssembly.***

# Roadmap

**Mistletoe** is still very early on in the development lifecycle.  The above example works without issue, but
there are a handful of things that need to be addressed before it could reasonably be opened up for use:

1. The `mistctl` utility currently only supports YAML generation -- it should be expanded to **support
installation to clusters.**  This also implies all the usual goodies around lifecycle management, notably
upgrades and uninstallation.
2. The package manager is currently confined to packages that exist on the local filesystem.  We'll need to
**attach a registry system** for users to pull and run packages by ref, not to mention some starting packages
to demonstrate practical development and use.
3. **Need documentation, documentation, and more documentation.**  The only way to guarantee adoption is to
provide best-in-class documentation covering everything from general usage, to package development in
multiple languages, to the nitty-gritty advanced information about the runtime for eager hackers.

There's all that, as well as a number of less-important, but also-desired efforts worth evaluating:

1. **Dependencies!**  We can get a lot more value by adding a way for packages to declare and call dependencies
outside of whatever is compiled into the package.  This gets really useful when you consider #3 below.
2. **Explicit support for other languages.**  While any WebAssembly-ready language will work, the project is
written in Rust and provides Rust-first tooling.  For the next push, I'm looking to expand library support to
TypeScript via the [QuickJS runtime](https://bellard.org/quickjs/).
3. Since the runtime is pretty open, we could theoretically **package other package managers.**  This means we
could provide a system to install Helm packages as if they were any other Mistletoe package.
4. We could use some **package templates.**  The templating engine in Helm is a blessing and a curse -- it
makes generating manifests pretty easy, but tends to get really overcomplicated really quickly
depending on what logic is entailed in deploying the system.  **Mistletoe** addresses the curse pretty well,
but we should provide some easy-build package templates to recreate the blessing.
