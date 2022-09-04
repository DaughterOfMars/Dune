pub mod setup;
pub mod storm;

use super::*;

// TODO:
// - Use Spawn/Despawn over Insert/Remove
// - Convert Lerp to it's own entity with a reference to the subject
// - Add/modify an action queue
// - Add action queue entity for any series of sequential actions
// - Action queues can spawn entities and wait for end conditions (despawned/signal/none/???)
