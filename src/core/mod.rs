// core/mod.rs — AUDIT system, LLM orchestration, context management,
// autonomous build loop, permissions, agent switching
// Phases 4, 7, 12

pub mod audit;
pub mod spec_writer;

// Phase 7
pub mod progress;
pub mod autonomous;

// Stubs for later phases
pub mod llm {}
pub mod context {}
pub mod permissions {}
pub mod agents {}
