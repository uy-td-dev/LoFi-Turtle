#![allow(dead_code)]
/// Application layer - Use cases and application services
/// 
/// This layer contains:
/// - Use cases (application-specific business rules)
/// - Application services (orchestration)
/// - DTOs (Data Transfer Objects)
/// - Workflow coordination
/// 
/// The application layer coordinates between the domain and infrastructure layers.

pub mod use_cases;
pub mod services;
pub mod dto;

// Re-export use cases for convenience
// Use cases are available through their modules
// Individual imports can be added as needed
