use rand::{Rng, SeedableRng};
use std::hash::Hash;
use vec_map::VecMap;
use Deciban;

/// Seed a RNG from any hashable value.
pub fn seeded_rng<I, O>(seed: &I) -> O
where
    I: Hash,
    O: SeedableRng<Seed = [u8; 16]>,
{
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let hash = hasher.finish().to_be();
    // XorShift seed mustn't be all-0.
    let hash = if hash == 0 { 1 } else { hash };

    let seed = unsafe { ::std::mem::transmute::<[u64; 2], [u8; 16]>([hash, hash]) };
    O::from_seed(seed)
}

/// Additional methods for random number generators.
pub trait RngExt {
    /// Return true with 50 % probability.
    fn coinflip(&mut self) -> bool;

    /// Return true with probability 1 / n.
    fn one_chance_in(&mut self, n: u32) -> bool;

    /// Return true with p probability.
    fn with_chance(&mut self, p: f32) -> bool;

    /// Return true with the probability corresponding to the log odds with
    /// the given deciban value.
    fn with_log_odds(&mut self, db: Deciban) -> bool;
}

impl<T: Rng> RngExt for T {
    fn coinflip(&mut self) -> bool { self.gen_bool(1.0 / 2.0) }

    fn one_chance_in(&mut self, n: u32) -> bool { self.gen_bool(1.0 / n as f64) }

    fn with_chance(&mut self, p: f32) -> bool { self.gen_range(0.0, 1.0) < p }

    fn with_log_odds(&mut self, db: Deciban) -> bool { db > self.gen::<Deciban>() }
}

/// Lazily evaluated random permutation.
pub struct RandomPermutation<'a, R: Rng + 'static> {
    remain: usize,
    shuffle: VecMap<usize>,
    rng: &'a mut R,
}

impl<'a, R: Rng + 'static> RandomPermutation<'a, R> {
    pub fn new(rng: &'a mut R, n: usize) -> RandomPermutation<'a, R> {
        RandomPermutation {
            remain: n,
            shuffle: VecMap::new(),
            rng: rng,
        }
    }
}

impl<'a, R: Rng + 'static> Iterator for RandomPermutation<'a, R> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.remain == 0 {
            return None;
        }

        let swap_idx = self.rng.gen_range(0, self.remain);
        self.remain -= 1;

        let head = *self.shuffle.get(self.remain).unwrap_or(&self.remain);
        Some(self.shuffle.insert(swap_idx, head).unwrap_or(swap_idx))
    }
}

/// Independent sampling trait.
///
/// Implemented as a convenience struct since `rand::distributions::IndependentSample` always wants
/// a boilerplate implementation for `rand::distributions::Sample` as well.
pub trait IndependentSample<Support>: Sized {
    /// Sample a single value from the distribution.
    fn ind_sample<R: Rng>(&self, rng: &mut R) -> Support;

    /// Create an endless iterator sampling values from the distribution.
    fn iter<'a, 'b, R: Rng + 'a>(
        &'b self,
        rng: &'a mut R,
    ) -> SampleIterator<'a, 'b, R, Support, Self> {
        SampleIterator {
            rng,
            sample: self,
            phantom: ::std::marker::PhantomData,
        }
    }
}

pub struct SampleIterator<'a, 'b, R: Rng + 'a, Support, S: IndependentSample<Support> + 'b> {
    rng: &'a mut R,
    sample: &'b S,
    phantom: ::std::marker::PhantomData<Support>,
}

impl<'a, 'b, R: Rng + 'a, Support, S: IndependentSample<Support> + 'b> Iterator
    for SampleIterator<'a, 'b, R, Support, S>
{
    type Item = Support;

    fn next(&mut self) -> Option<Self::Item> { Some(self.sample.ind_sample(self.rng)) }
}
