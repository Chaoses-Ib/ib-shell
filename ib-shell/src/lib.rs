/*!
See [ib-shell](https://github.com/Chaoses-Ib/ib-shell).

## Crate features
*/
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "doc", doc = document_features::document_features!())]

#[cfg(feature = "hook")]
pub mod hook;
#[cfg(feature = "item")]
pub use ib_shell_item as item;
#[cfg(feature = "verb")]
pub use ib_shell_verb as verb;
pub mod app;
