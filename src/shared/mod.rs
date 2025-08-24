/// Shared module - Common utilities and cross-cutting concerns
/// 
/// This module contains:
/// - Error types and handling
/// - Common constants and configurations
/// - Utility functions
/// - Cross-cutting concerns that are used across multiple layers

pub mod errors;

// Re-export commonly used types
// Only export what's currently used by legacy code
// Other error types available but not yet integrated
// pub use errors::{
//     ApplicationError,
//     Result,
// };

// Other error types are available but not currently used
// They can be imported individually as needed
