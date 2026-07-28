//! Deterministic RNG streams.
//!
//! The simulation never uses a single global RNG.  Each subsystem gets its
//! own seeded stream so that:
//!
//! 1. Changing the world seed changes everything predictably.
//! 2. Adding a new subsystem does not alter the RNG sequence of existing ones.
//! 3. Debugging is easier: you can replay a single subsystem in isolation.
//!
//! All streams use `ChaCha8Rng` for speed and adequate entropy.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Subsystem identifier for RNG streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RngStream {
    World,
    Psychology,
    Behavior,
    Social,
    Economy,
    Ecology,
    Narrative,
}

impl RngStream {
    /// All defined streams, in a stable order.
    pub const ALL: &'static [RngStream] = &[
        RngStream::World,
        RngStream::Psychology,
        RngStream::Behavior,
        RngStream::Social,
        RngStream::Economy,
        RngStream::Ecology,
        RngStream::Narrative,
    ];
}

/// A collection of per-subsystem RNG streams, all seeded from one master seed.
///
/// RNG streams are NOT serialised.  To replay, re-seed from the master seed.
pub struct RngStreams {
    world: ChaCha8Rng,
    psychology: ChaCha8Rng,
    behavior: ChaCha8Rng,
    social: ChaCha8Rng,
    economy: ChaCha8Rng,
    ecology: ChaCha8Rng,
    narrative: ChaCha8Rng,
}

impl RngStreams {
    /// Create all streams from a single master seed.
    pub fn new(master_seed: u64) -> Self {
        // Each stream gets a distinct seed derived from the master.
        // We use different constants per stream to guarantee separation.
        Self {
            world: ChaCha8Rng::seed_from_u64(master_seed.wrapping_add(1)),
            psychology: ChaCha8Rng::seed_from_u64(master_seed.wrapping_add(2)),
            behavior: ChaCha8Rng::seed_from_u64(master_seed.wrapping_add(3)),
            social: ChaCha8Rng::seed_from_u64(master_seed.wrapping_add(4)),
            economy: ChaCha8Rng::seed_from_u64(master_seed.wrapping_add(5)),
            ecology: ChaCha8Rng::seed_from_u64(master_seed.wrapping_add(6)),
            narrative: ChaCha8Rng::seed_from_u64(master_seed.wrapping_add(7)),
        }
    }

    /// Get a mutable reference to a specific stream.
    pub fn get_mut(&mut self, stream: RngStream) -> &mut ChaCha8Rng {
        match stream {
            RngStream::World => &mut self.world,
            RngStream::Psychology => &mut self.psychology,
            RngStream::Behavior => &mut self.behavior,
            RngStream::Social => &mut self.social,
            RngStream::Economy => &mut self.economy,
            RngStream::Ecology => &mut self.ecology,
            RngStream::Narrative => &mut self.narrative,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn deterministic_streams() {
        let mut a = RngStreams::new(42);
        let mut b = RngStreams::new(42);

        let a_val: u32 = a.get_mut(RngStream::World).random();
        let b_val: u32 = b.get_mut(RngStream::World).random();
        assert_eq!(a_val, b_val);
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = RngStreams::new(42);
        let mut b = RngStreams::new(99);

        let a_val: u32 = a.get_mut(RngStream::World).random();
        let b_val: u32 = b.get_mut(RngStream::World).random();
        assert_ne!(a_val, b_val);
    }

    #[test]
    fn streams_are_independent() {
        let mut rng = RngStreams::new(42);
        let w1: u32 = rng.get_mut(RngStream::World).random();
        let s1: u32 = rng.get_mut(RngStream::Social).random();

        let mut rng2 = RngStreams::new(42);
        let w2: u32 = rng2.get_mut(RngStream::World).random();
        let s2: u32 = rng2.get_mut(RngStream::Social).random();

        assert_eq!(w1, w2);
        assert_eq!(s1, s2);
    }
}
