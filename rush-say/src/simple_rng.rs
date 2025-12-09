use std::hash::{Hash, Hasher};
use std::time::SystemTime;

#[derive(Clone, Copy)]
pub struct SimpleRng(u64);

impl Default for SimpleRng {
    fn default() -> Self {
        Self(Self::entropy())
    }
}

impl SimpleRng {
    /// 尽量“全平台可用”的种子收集（非安全），永不 panic
    pub fn new() -> Self {
        Self::default()
    }

    /// 可显式指定种子
    pub fn with_seed(seed: u64) -> Self {
        Self(if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed })
    }

    /// 产生 [0, max) 的 usize，使用拒绝采样避免取模偏差；max==0 返回 0
    pub fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        let m = max as u64;
        // 最大可接受的上界（小于此值再取模不会偏）
        let bound = u64::MAX / m * m;
        loop {
            let x = self.next_u64();
            if x < bound {
                return (x % m) as usize;
            }
        }
    }

    /// 产生下一个 u64（xorshift64*）
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    // -------------------- 内部工具 --------------------

    #[inline]
    fn mix64(mut z: u64) -> u64 {
        // splitmix64 的一次混洗，用来把弱熵打散
        z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut x = z;
        x ^= x >> 30;
        x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^ (x >> 31)
    }

    fn entropy() -> u64 {
        // 1) 系统时间（失败则取 0）
        let t = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        // 2) 进程 id（并非所有目标都有意义，std 会尽量给）
        let pid = std::process::id() as u64;

        // 3) 线程 id（转字符串再哈希）
        let tid_hash = {
            let tid = std::thread::current().id();
            let s = format!("{tid:?}");
            let mut h = std::collections::hash_map::DefaultHasher::new();
            s.hash(&mut h);
            h.finish()
        };

        // 4) 栈地址 + 函数地址（一些低熵）
        let stack_addr = {
            let x = 0u8;
            (&x as *const u8 as usize) as u64
        };
        fn tag() {}
        let fn_addr = (tag as usize) as u64;

        // 5) 综合混洗
        let seed0 = t ^ (pid.rotate_left(7)) ^ (tid_hash.rotate_left(13)) ^ (stack_addr.rotate_left(29)) ^ (fn_addr.rotate_left(47));

        let s1 = Self::mix64(seed0);
        let s2 = Self::mix64(seed0 ^ 0xA5A5_A5A5_A5A5_A5A5);
        let seed = s1 ^ s2;

        if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed }
    }
}
