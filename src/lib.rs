use wasmer::{Store, Module, Memory, Instance};

struct MistHuskPackage {
    store: Store,
    module: Module,
    instance: Instance,
    memory: Memory,
}
