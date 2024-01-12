<p align="center">
  <img src="logo.png" alt="Mistletoe logo" width="200" height="200" />
</p>

# Mistletoe (WIP) - *the Polyglot Kubernetes Package Manager*

[Site](https://mistletoe.dev/) | [Blog](https://mistletoe.dev/blog/) | [Book](https://mistletoe.dev/book/)

**Mistletoe** is a Kubernetes package manager that goes about things in a different way than the rest of the ecosystem. It's built around a runtime where the packages are **WebAssembly** modules:

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

**The above is a simple package written in Rust.  If you install it with:**

```sh
mistctl install my-namespace -p mistletoe/examples/namespace-example:0.1.2
```

**The following will be installed to your cluster:**

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: my-namespace
  labels:
    mistletoe.dev/tied-to-install-name: my-namespace
    mistletoe.dev/tied-to-install-version: v0
```

This is one of a nearly-unlimited number of ways to write a package.  ***All you need is a language with a YAML parser that can compile to WebAssembly.***

# Roadmap

**Mistletoe** is still very early on in the development lifecycle, but we're moving quickly!

## Recent progress

1. The ability to **install to** clusters has landed!  (And we'll even throw in the ability to uninstall from clusters for free!)

2. We've implemented the **registry system** that's used by the above example -- you can add registries by Git URL and reference packages inside them by name now!  There's also a default ***"mistletoe"*** registry added that points to [mistletoe-registry](https://github.com/gsfraley/mistletoe-registry). There's only the example package up now, but more will be coming soon!

## Next milestones

That said, there are still a handful of things that need to be addressed before it could reasonably be opened up for use:

1. **Need documentation, documentation, and more documentation.**  The only way to guarantee adoption is to provide best-in-class documentation covering everything from general usage, to package development in multiple languages, to the nitty-gritty advanced information about the runtime for eager hackers.

2. Things could definitely **use some polish** -- the core of each function and feature that's being added is complete, but it's worth doing an overall pass to make sure that everything meshes well together and that the starting use cases have been well-tested.

3. We'll need some **starting packages.**  These should be stable, ready-to-use implementations of a couple popular services.  While really loading up the registries with packages to kick-start the ecosystem is important and being planned out, for now we'll want to keep the collection small to avoid rewriting a bunch of packages as things develop.

There's all that, as well as a number of less-important, but also-desired efforts worth evaluating:

1. **Dynamic dependencies!**  We can get a lot more value by adding a way for packages to declare and call dependencies outside of whatever is compiled into the package.  This gets really useful when you consider #3 below.

2. **Explicit support for other languages.**  While any WebAssembly-ready language will work, the project is written in Rust and provides Rust-first tooling.  For the next push, I'm looking to expand library support to TypeScript via the [QuickJS runtime](https://bellard.org/quickjs/).

3. Since the runtime is pretty open, we could theoretically **package other package managers.**  This means we could provide a system to install Helm packages as if they were any other Mistletoe package.

4. We could use some **package templates.**  The templating engine in Helm is a blessing and a curse -- it makes generating manifests pretty easy, but tends to get really overcomplicated really quickly depending on what logic is entailed in deploying the system.  **Mistletoe** addresses the curse pretty well, but we should provide some easy-build package templates to recreate the blessing.
