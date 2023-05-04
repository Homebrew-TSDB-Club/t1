use std::future::Future;

use uuid::Uuid;

#[macro_export]
macro_rules! try_yield {
    ($cx:expr) => {
        if $cx.take() {
            $cx.yield_now().await;
        }
    };
}

#[derive(Debug)]
pub struct Context {
    #[allow(unused)]
    session_id: Uuid,
    quota: usize,
    n: usize,
}

impl Context {
    #[inline]
    pub fn new(quota: usize) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            quota,
            n: quota,
        }
    }

    #[inline]
    pub fn take(&mut self) -> bool {
        self.n -= 1;
        self.n == 0
    }

    #[inline]
    pub fn yield_now(&mut self) -> impl Future<Output = ()> {
        self.n = self.quota;
        futures_lite::future::yield_now()
    }

    #[inline]
    pub fn copy_from(ctx: &Context) -> Self {
        Self {
            session_id: ctx.session_id,
            quota: ctx.quota,
            n: ctx.quota,
        }
    }
}
