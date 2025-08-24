#![allow(dead_code)]
/// Infrastructure layer - External concerns and implementations
/// 
/// This layer contains:
/// - Repository implementations (SQLite, file system, etc.)
/// - External service adapters
/// - Database connections and configurations
/// - Framework-specific code
/// 
/// The infrastructure layer implements the interfaces defined in the domain layer.

pub mod repositories;
pub mod factories;

// Re-export for convenience
// Infrastructure components are available through their modules
// Individual imports can be added as needed
