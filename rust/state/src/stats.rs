use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

use axum::http::StatusCode;

#[derive(Debug, Default)]
struct Inner {
    total: AtomicU64,
    active: AtomicI64,
    ok: AtomicU64,
    client_errors: AtomicU64,
    server_errors: AtomicU64,
}

/// 全局请求统计（无锁原子计数）
///
/// `Clone` 共享同一份计数器（内部 Arc）——这正是全局状态的语义：
/// 任意克隆体上的读写都是同一份数据。由 `track` 中间件自动维护，业务代码不要直接调 begin/end。
#[derive(Debug, Clone, Default)]
pub struct Stats {
    inner: Arc<Inner>,
}

impl Stats {
    /// 请求进入：总请求数 +1，进行中 +1
    pub fn begin(&self) {
        self.inner.total.fetch_add(1, Ordering::Relaxed);
        self.inner.active.fetch_add(1, Ordering::Relaxed);
    }

    /// 响应返回：进行中 -1，按状态码段归类（2xx / 4xx / 5xx）
    pub fn end(&self, status: StatusCode) {
        self.inner.active.fetch_sub(1, Ordering::Relaxed);
        match status.as_u16() {
            200..=299 => self.inner.ok.fetch_add(1, Ordering::Relaxed),
            400..=499 => self.inner.client_errors.fetch_add(1, Ordering::Relaxed),
            500..=599 => self.inner.server_errors.fetch_add(1, Ordering::Relaxed),
            _ => 0,
        };
    }

    /// 自启动以来的总请求数
    pub fn total(&self) -> u64 {
        self.inner.total.load(Ordering::Relaxed)
    }

    /// 进行中请求数（begin/end 严格配对，不会为负）
    pub fn active(&self) -> u64 {
        self.inner.active.load(Ordering::Relaxed).max(0) as u64
    }

    /// 2xx 响应数
    pub fn ok(&self) -> u64 {
        self.inner.ok.load(Ordering::Relaxed)
    }

    /// 4xx 响应数
    pub fn client_errors(&self) -> u64 {
        self.inner.client_errors.load(Ordering::Relaxed)
    }

    /// 5xx 响应数
    pub fn server_errors(&self) -> u64 {
        self.inner.server_errors.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
#[path = "stats_tests.rs"]
mod tests;
