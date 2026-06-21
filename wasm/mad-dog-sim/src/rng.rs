//! Mulberry32 PRNG — matches `makeRng` in `src/sim/quantum.ts`.

pub struct Rng {
    state: u32,
}

/// JavaScript `>>>` (unsigned right shift) on a signed 32-bit value.
fn ushr(x: i32, bits: u32) -> i32 {
    (x as u32).wrapping_shr(bits) as i32
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> f64 {
        let a = (self.state.wrapping_add(0x6d2b79f5)) as i32;
        self.state = a as u32;

        let mut t = i32::wrapping_mul(a ^ ushr(a, 15), a | 1);
        t = i32::wrapping_add(
            t,
            i32::wrapping_mul(t ^ ushr(t, 7), t | 61),
        ) ^ t;
        let tu = t as u32;
        ((tu ^ (tu >> 14)) as f64) / 4294967296.0
    }
}
