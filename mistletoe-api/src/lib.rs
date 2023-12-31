//! This is the main crate for the API objects that are passed back and forth between the engine
//! and the packages.  They roughly follow the same pattern as Kubernetes resource definititions.
//! Some examples of the output are provided below.
//! 
//! ## MistPackage
//! 
//! ```yaml
//! apiVersion: mistletoe.dev/v1alpha1
//! kind: MistPackage
//! metadata:
//!   name: nginx-example
//!   labels:
//!     mistletoe.dev/group: mistletoe-examples
//! ```
//! 
//! This is provided by the `info` method of the package getting called.  It contains some of the
//! usual metadata, notably the `name` and `labels`.  Some of the labels are used by
//! **Mistletoe** itself when returning information about the package to the end user.
//! 
//! ## MistInput
//! 
//! This is passed into the package as the main input it receives when generating output:
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
//! methods to convert the `data` into any Deserialize objects the package has defined.
//! 
//! ## MistResult
//! 
//! ```yaml
//! apiVersion: mistletoe.dev/v1alpha1
//! kind: MistResult
//! data:
//!   result: Ok
//!   message: 'nothing went wrong' # This line is optional
//!   files:
//!     namespace.yaml: |
//!       apiVersion: v1
//!       kind: Namespace
//!       metadata:
//!         name: my-namespace
//! ```
//!
//! Or...
//! 
//! ```yaml
//! apiVersion: mistletoe.dev/v1alpha1
//! kind: MistResult
//! data:
//!   result: Err
//!   message: 'something went wrong'
//! ```
//! 
//! This is what the package returns to the engine, and contains the output of the package execution.
//! The required fields when returning an error are `result: Err` as well as a package-supplied
//! `message` describing what went wrong.  The required fields when returning successful output
//! are `result: Ok` and a map of `files` that can be output in the form of a directory structure.
//! A `message` may also be provided in an `Ok` case if there's info to convey.
//! 
//! It's worth noting in either case, the `message` field may also be multiple lines long if there
//! is a lot of info the package wishes to provide to the end user.

/// Module containing API objects for the 0.1 version of the mistletoe-api.
/// The versions of the Kubernetes definitions are `mistletoe.dev/v1alpha1`.
pub mod v1alpha1;
