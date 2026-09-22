/// A small seeded generator. The original used QBasic's RND seeded from
/// TIMER. Matching its exact sequence is a non-goal, but a fixed seed
/// reproducing the same game is wanted, for the playthrough test.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        // Avoid the all zero state, which would stick.
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64star
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// FnRan(x) = INT(RND(1) * x) + 1, so the result is 1 to x inclusive.
    pub fn fn_ran(&mut self, x: i32) -> i32 {
        (self.next_f64() * x as f64) as i32 + 1
    }
}
