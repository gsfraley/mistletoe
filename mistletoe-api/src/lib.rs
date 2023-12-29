//! This is the main crate for the API objects that are passed back and forth between the engine
//! and the modules.  They roughly follow the same pattern as Kubernetes resource definititions.
//! Some examples of the output are provided below.
//! 
//! ## MistInput
//! 
//! This is passed into the module as the main input it receives when generating output:
//! 
//! ```yaml
//! apiVersion: mistletoe.dev/v1alpha1
//! kind: MistInput
//! data:
//!   name: my-nginx
//!   namespace: my-namespace
//! ```
//! 
//! The important part is the `data`, and the `data` is completely freeform.  This could be
//! considered roughly equivalent to Helm's values.  The objects provided by this package have
//! methods to convert the `data` into any Deserialize objects the module has defined.
//! 
//! ## MistPackage
//! 
//! ```yaml
//! apiVersion: mistletoe.dev/v1alpha1
//! kind: MistPackage
//! metadata:
//!   name: example-nginx
//!   labels:
//!     mistletoe.dev/group: mistletoe-examples
//! spec:
//!   functions:
//!     generate: __mistletoe_generate
//!     alloc: __mistletoe_alloc
//!     dealloc: __mistletoe_dealloc
//! ```
//! 
//! This is provided by the `info` method of the module getting called.  It contains some of the
//! usual metadata, notably the `name` and `labels`.  Some of the labels are used by
//! **Mistletoe** itself when returning information about the package to the end user.
//! 
//! ## MistResult
//! 
//! ```yaml
//! apiVersion: mistletoe.dev/v1alpha1
//! kind: MistResult
//! data:
//!   result: Ok
//!   message: 'warning: nothing went wrong' # This line is optional
//!   files:
//!     namespace.yaml: |
//!       apiVersion: v1
//!       kind: Namespace
//!       metadata:
//!         name: my-namespace
//!     resources/service.yaml: |
//!       apiVersion: v1
//!       kind: Service
//!       metadata:
//!         name: my-service
//!         labels:
//!           app: my-service
//!       spec:
//!         type: LoadBalancer
//!         selector:
//!           app: my-service
//!         ports:
//!         - port: 80
//!           containerPort: 80
//! ```
//!
//! Or...
//! 
//! ```yaml
//! apiVersion: mistletoe.dev/v1alpha1
//! kind: MistResult
//! data:
//!   result: Err
//!   message: 'error: something went wrong'
//! ```
//! 
//! This is what the module returns to the engine, and contains the output of the module execution.
//! The required fields when returning an error are `result: Err` as well as a module-supplied
//! `message` describing what went wrong.  The required fields when returning successful output
//! are `result: Ok` and a map of `files` that can be output in the form of a directory structure.
//! A `message` may also be provided in an `Ok` case if there's info to convey.
//! 
//! It's worth noting in either case, the `message` field may also be multiple lines long if there
//! is a lot of info the module wishes to provide to the end user.

/// Module containing API objects for the 0.1 version of the mistletoe-api.
/// The versions of the Kubernetes definitions are `mistletoe.dev/v1alpha1`.
pub mod v1alpha1;
