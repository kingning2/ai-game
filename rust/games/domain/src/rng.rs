//! 确定性随机数 —— xorshift64 + Fisher–Yates（seed 相同 ⇒ 序列相同，发牌可复现）。
//!
//! 从 `werewolf/engine.rs` 原样搬移（Bevy ECS 迁移阶段 3）：
//! 旧引擎与 game-ecs 发牌共用此实现，改动语义前先确认两侧调用方。

/// 随机数发生器：splitmix64 雪崩混淆种子，xorshift64 产流
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rng(u64);

impl Rng {
    /// splitmix64 雪崩混淆：相邻 seed（如 7/8）也能产生完全不同的序列
    pub fn from_seed(seed: u64) -> Self {
        let mut z = seed;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Self((z ^ (z >> 31)) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Fisher–Yates 洗牌
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = (self.next_u64() % (i as u64 + 1)) as usize;
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::from_seed(42);
        let mut b = Rng::from_seed(42);
        for _ in 0..16 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn adjacent_seeds_diverge() {
        let mut a = Rng::from_seed(7);
        let mut b = Rng::from_seed(8);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn shuffle_is_permutation() {
        let mut items: Vec<u32> = (0..64).collect();
        Rng::from_seed(1).shuffle(&mut items);
        items.sort_unstable();
        assert_eq!(items, (0..64).collect::<Vec<_>>());
    }
}
