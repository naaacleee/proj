//! Простой лимитёр запросов в памяти процесса.
//!
//! Используется для защиты auth-эндпоинтов от перебора паролей.
//! Без внешних зависимостей (Redis-лимиты — когда понадобится масштаб).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    window: Duration,
    max: usize,
    hits: Mutex<HashMap<String, Vec<Instant>>>,
}

impl RateLimiter {
    pub fn new(max: usize, window: Duration) -> Self {
        Self {
            window,
            max,
            hits: Mutex::new(HashMap::new()),
        }
    }

    /// `true` — запрос разрешён, `false` — лимит исчерпан.
    pub fn allow(&self, key: &str) -> bool {
        // Если мьютекс «отравлен» — не блокируем работу сервиса.
        let Ok(mut map) = self.hits.lock() else {
            return true;
        };
        let now = Instant::now();
        // Заодно чистим устаревшие записи — память ограничена.
        map.retain(|_, times| {
            times.retain(|t| now.duration_since(*t) < self.window);
            !times.is_empty()
        });
        let times = map.entry(key.to_string()).or_default();
        if times.len() >= self.max {
            return false;
        }
        times.push(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_max_in_window() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        assert!(limiter.allow("k"));
        assert!(limiter.allow("k"));
        assert!(!limiter.allow("k"));
    }

    #[test]
    fn keys_are_independent() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        assert!(limiter.allow("a"));
        assert!(!limiter.allow("a"));
        assert!(limiter.allow("b"));
    }
}
