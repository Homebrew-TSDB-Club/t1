use std::{
    fmt,
    fmt::{Debug, Formatter},
    ops::{Add, Div, Mul, Sub},
    time::UNIX_EPOCH,
};

const MILLIS_PER_SEC: i64 = 1_000;

pub const EPOCH: Instant = Instant { millis: 0 };

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    millis: i64,
}

impl Duration {
    pub const SECOND: Self = Self {
        millis: MILLIS_PER_SEC,
    };

    #[inline]
    pub fn as_millis(&self) -> i64 {
        self.millis
    }

    #[inline]
    pub fn from_secs(secs: i64) -> Self {
        Self {
            millis: secs * 1000,
        }
    }
}

impl Mul<Duration> for isize {
    type Output = Duration;

    #[inline]
    fn mul(self, rhs: Duration) -> Self::Output {
        Duration {
            millis: rhs.millis * self as i64,
        }
    }
}

impl Mul<u32> for Duration {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            millis: self.millis * rhs as i64,
        }
    }
}

impl Mul<i64> for Duration {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: i64) -> Self::Output {
        Self {
            millis: self.millis * rhs,
        }
    }
}

impl Div<i64> for Duration {
    type Output = Self;

    #[inline]
    fn div(self, rhs: i64) -> Self::Output {
        Duration {
            millis: self.millis / rhs,
        }
    }
}

impl Div for Duration {
    type Output = i64;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.millis / rhs.millis
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    millis: i64,
}

impl Instant {
    pub fn now() -> Self {
        Self {
            millis: std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        }
    }

    #[inline]
    pub fn from_millis(m: i64) -> Self {
        Self { millis: m }
    }

    #[inline]
    pub fn as_millis(&self) -> i64 {
        self.millis
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let duration = std::time::Duration::from_millis(self.millis as u64);
        (std::time::UNIX_EPOCH + duration).fmt(f)
    }
}

impl Add<Duration> for Instant {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Duration) -> Self::Output {
        Self {
            millis: self.millis + rhs.millis,
        }
    }
}

impl Sub for Instant {
    type Output = Duration;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Duration {
            millis: self.millis - rhs.millis,
        }
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    #[inline]
    fn sub(self, rhs: Duration) -> Self::Output {
        Self {
            millis: self.millis - rhs.millis,
        }
    }
}

impl Div<Duration> for Instant {
    type Output = i64;

    #[inline]
    fn div(self, rhs: Duration) -> i64 {
        self.millis / rhs.millis
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    pub start: Option<Instant>,
    pub end: Option<Instant>,
}

impl Range {
    #[inline]
    pub fn is_empty(&self) -> bool {
        if let Some(start) = self.start {
            if let Some(end) = self.end {
                return end - start > Duration::from_secs(0);
            }
        }
        true
    }
}

impl std::ops::BitAnd for Range {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        let start = match self.start {
            Some(start) => match rhs.start {
                Some(rhs) => Some(if rhs > start { rhs } else { start }),
                None => Some(start),
            },
            None => rhs.start,
        };
        let end = match self.end {
            Some(end) => match rhs.end {
                Some(rhs) => Some(if rhs < end { rhs } else { end }),
                None => Some(end),
            },
            None => rhs.end,
        };
        Self { start, end }
    }
}
