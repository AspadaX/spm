// Package module - exports all components related to package management
pub mod creator;
pub mod dependency;
pub mod manager;
pub mod metadata;
pub mod std_lib;

// Re-export key structures and functions for external use
pub use manager::PackageManager;
pub use metadata::{Package, PackageMetadata};

pub use metadata::is_inside_a_package;
