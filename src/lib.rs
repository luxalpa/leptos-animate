//! Animation toolkit for Leptos
//!
//! Getting animations to work in Leptos is surprisingly difficult. This crate provides a set of utilities in order to make handling animations easier.
//!
//! Please note that there are various gotchas to handling animations in a CSS friendly way.
//!
//! This crate currently uses the Web Animations API, which means your animations need to be configured in code and not in CSS.
//!
//! Ensure using the `ssr` feature when building the ssr code, as web animations cannot be run on the server.

pub use animated_for::*;
pub use animated_layout::*;
pub use animated_show::*;
pub use animated_swap::*;
pub use animation_defs::*;
pub use position::*;
pub use size_transition::*;

mod animated_for;
mod animated_layout;
mod animated_show;
mod animated_swap;
mod animation_defs;
pub mod dynamics;
mod position;
mod size_transition;
