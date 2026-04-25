pub mod allocate_account_ledger;
pub mod allocate_slab_allocator;

// Re-export allocation handlers for entrypoint dispatch.
pub use allocate_account_ledger::*;
pub use allocate_slab_allocator::*;
