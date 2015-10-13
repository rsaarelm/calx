use std::mem;
use rand::{Rng, SeedableRng};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::Error;
use to_log_odds;

/// Additional methods for random number generators.
pub trait RngExt {
    /// Return true with 50 % probability.
    fn coinflip(&mut self) -> bool;

    /// Return true with probability 1 / n.
    fn one_chance_in(&mut self, n: u32) -> bool;

    /// Return true with p probability.
    fn with_chance(&mut self, p: f32) -> bool;

    /// Return a log odds deciban score that corresponds to a random
    /// probability from [0, 1].
    ///
    fn log_odds(&mut self) -> f32;

    /// Return true with the probability corresponding to the log odds with
    /// the given deciban value.
    fn with_log_odds(&mut self, db: f32) -> bool;
}

impl<T: Rng> RngExt for T {
    fn coinflip(&mut self) -> bool {
        self.gen_weighted_bool(2)
    }

    fn one_chance_in(&mut self, n: u32) -> bool {
        self.gen_weighted_bool(n)
    }

    fn with_chance(&mut self, p: f32) -> bool {
        self.gen_range(0.0, 1.0) < p
    }

    fn log_odds(&mut self) -> f32 {
        to_log_odds(self.gen_range(0.0, 1.0))
    }

    fn with_log_odds(&mut self, db: f32) -> bool {
        db > self.log_odds()
    }
}

/// A wrapper that makes a Rng implementation encodable.
///
/// For games that want to store the current Rng state as a part of the save
/// game. Works by casting the Rng representation into a binary blob, will
/// crash and burn if the Rng struct is not plain-old-data.
pub struct EncodeRng<T> {
    inner: T,
}

impl<T: Rng+'static> EncodeRng<T> {
    pub fn new(inner: T) -> EncodeRng<T> {
        EncodeRng { inner: inner }
    }
}

impl<T: SeedableRng<S>+Rng+'static, S> SeedableRng<S> for EncodeRng<T> {
    fn reseed(&mut self, seed: S) {
        self.inner.reseed(seed);
    }

    fn from_seed(seed: S) -> EncodeRng<T> {
        EncodeRng::new(SeedableRng::from_seed(seed))
    }
}

impl<T: Rng> Rng for EncodeRng<T> {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }
}

impl<T: Rng+'static> Serialize for EncodeRng<T> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        let mut vec = Vec::new();
        unsafe {
            let view = self as *const _ as *const u8;
            for i in 0..(mem::size_of::<T>()) {
                vec.push(*view.offset(i as isize));
            }
        }
        vec.serialize(serializer)
    }
}

impl<T: Rng+'static> Deserialize for EncodeRng<T> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let blob: Vec<u8> = try!(Deserialize::deserialize(deserializer));
        unsafe {
            if blob.len() == mem::size_of::<T>() {
                Ok(EncodeRng::new(mem::transmute_copy(&blob[0])))
            } else {
                Err(Error::syntax("Bad RNG blob length"))
            }
        }
    }
}

#[cfg(test)]
mod test {
use rand::{Rng, XorShiftRng, SeedableRng};
use super::EncodeRng;

    #[test]
    fn test_serialize_rng() {
        use bincode::{serde, SizeLimit};

        let mut rng: EncodeRng<XorShiftRng> = SeedableRng::from_seed([1, 2, 3, 4]);

        let saved = serde::serialize(&rng, SizeLimit::Infinite).expect("Serialization failed");
        let mut rng2 = serde::deserialize::<EncodeRng<XorShiftRng>>(&saved).expect("Deserialization failed");

        assert!(rng.next_u32() == rng2.next_u32());
    }
}
