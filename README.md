# **Mistletoe** Package Manager

## Run WASM, Get Kubernetes Yaml

WebAssembly is a revolutionary technology that enables developers to run code in a wide array of languages in a sandboxed environment.  It was established to provide a near-native code target for languages that could run in a browser, but it's been heavily put to use in various other places as a portable and embedded runtime.

**Mistletoe** is a Kubernetes package manager, but with a slightly different goal than other established package managers and resource utilities.  For example, *Helm* is SO good 🤌.  Super well developed and stable pattern where you write Golang templates that take some input YAML and spits out a whole lot of output YAML.

But it has constraints in the package development lifecycle itself.  Creating large interconnections of services, whether you do it in one package or split it into a handful of dependent packages, is rough.  Some might relate, but I found myself trying to put *a lot* of logic in places that didn't fit to cover all sorts of edge-cases.

So what can I do?  I could write another package manager with a different templating language I fancy more, or try to find an existing solution that suits my problem better, but then I'm hunting down different tooling for different use cases in the same problem area and fracturing all my workflows.

The goal of **Mistletoe** is to eliminate **all** limitations on what can be done for package maintainers.  Fundamentally, the package logic of all the existing solutions is roughly the same.  Ya put configuration text in, ya get Kubernetes resource YAML text out.  You can probably see where this is going -- **Mistletoe** really just provides a thin runtime that sends this configuration text into an arbitrary WASM module that runs all its own maths and computations that hands the Kubernetes resource YAML text back.

So instead of packaging its *own* logic, **Mistletoe** really just calls *other* people's logic, eliminating any effort I have to spend writing *my own* (/s).

## So what does it look like to end users?

Well right now, it looks like nothing, I'm still working on it.  (TODO: Greg: please remove this line when it's done.)

But what I'm *hoping* it looks like could be explained with a few yet-to-be-written commands:

```sh
# Gotta make this work first
mistctl generate example-nginx.mist -f input.yaml --out-dir=./out

# Make be able to pull packages from a registry and deploy to a cluster
mistctl install example/nginx -f input.yaml

# Co-opt other package types in pluggable engines
mistctl install helm:wordpress -f input.yaml
```

## So what does it look like to developers?

That depends on what the developer wants to do.  I'll be providing support libraries to make writing modules in Rust dead simple.  Take a look at the [<100 line Nginx example](https://github.com/gsfraley/mistletoe/blob/main/examples/nginx/src/lib.rs), for instance.

But ultimately the process is simple enough that any language will do.  All you need is a language that has a library for parsing YAML and the rest of the module writes itself, there's not a whole lot of special sauce.  Take the language, write a `generate` method that takes a string and returns a string. Badaboom.
