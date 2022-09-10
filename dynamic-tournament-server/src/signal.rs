use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU8, Ordering};
use std::task::{Context, Poll};

use tokio::sync::futures::Notified;
use tokio::sync::{Notify, Semaphore};

static WG_STATE: AtomicU8 = AtomicU8::new(0);
static WG_NOTIFY: Notify = Notify::const_new();
static WG_COUNTER: ShutdownSemaphore = ShutdownSemaphore::new();

pub fn init() {
    #[cfg(target_family = "unix")]
    unix::init();
}

/// Initiates a server shutdown.
pub fn terminate() {
    WG_STATE.store(1, Ordering::Relaxed);
    WG_NOTIFY.notify_waiters();

    log::debug!("Awaiting {} shutdown listeners", WG_COUNTER.permits());
}

pub async fn await_shutdown() {
    WG_NOTIFY.notified().await;
    WG_COUNTER.empty().await;
}

/// A [`Future`] that resolves once a shutdown signal has been received.
///
/// The `ShutdownListener` will asynchronously block the main thread and prevent it from exiting
/// before the instace was dropped.
#[derive(Debug)]
pub struct ShutdownListener<'a> {
    future: Notified<'a>,
}

impl<'a> ShutdownListener<'a> {
    pub fn new() -> Self {
        WG_COUNTER.add();

        Self {
            future: WG_NOTIFY.notified(),
        }
    }

    pub fn is_active(&self) -> bool {
        WG_STATE.load(Ordering::Relaxed) == 1
    }
}

impl<'a> Default for ShutdownListener<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Future for ShutdownListener<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_active() {
            return Poll::Ready(());
        }

        let future = unsafe { self.map_unchecked_mut(|s| &mut s.future) };

        future.poll(cx)
    }
}

impl<'a> Clone for ShutdownListener<'a> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<'a> Unpin for ShutdownListener<'a> {}

impl<'a> Drop for ShutdownListener<'a> {
    fn drop(&mut self) {
        WG_COUNTER.take();
    }
}

pub struct ShutdownSemaphore {
    semaphore: Semaphore,
    notify: Notify,
}

impl ShutdownSemaphore {
    pub const fn new() -> Self {
        Self {
            semaphore: Semaphore::const_new(0),
            notify: Notify::const_new(),
        }
    }

    pub fn add(&self) {
        self.semaphore.add_permits(1);
    }

    pub fn take(&self) {
        self.semaphore.try_acquire().unwrap().forget();

        if self.semaphore.available_permits() == 0 {
            self.notify.notify_waiters();
        }
    }

    pub fn permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    pub async fn empty(&self) {
        if self.permits() != 0 {
            self.notify.notified().await
        }
    }
}

#[cfg(target_family = "unix")]
mod unix {
    use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};

    use super::terminate;

    pub(super) fn init() {
        let action = SigAction::new(
            SigHandler::Handler(handle_terminate),
            SaFlags::empty(),
            SigSet::empty(),
        );

        // SAFETY: The signal handler does not reference any local data and is always
        // safe to call in the lifetime of the program.
        unsafe {
            let _ = sigaction(Signal::SIGTERM, &action);
            let _ = sigaction(Signal::SIGINT, &action);
        }
    }

    extern "C" fn handle_terminate(_: i32) {
        terminate();
    }
}
