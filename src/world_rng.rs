use bevy::prelude::Resource;
use rand::{SeedableRng, Rng as RandRng, rngs::{StdRng, OsRng}};

// Store number generator as a resource so tests can reuse seed.
#[derive(Resource)]
pub struct Rng(pub StdRng);

// NOTE: It's costly to instantiate an instance so only do this infrequently.
// This might be a bottleneck in testing. If it is, then it's possible to go back to using StdRng
// but will need to use NonSend: https://bevy-cheatbook.github.io/programming/non-send.html
impl Default for Rng {
    fn default() -> Self {
        Self(StdRng::seed_from_u64(OsRng {}.gen()))
    }
}
