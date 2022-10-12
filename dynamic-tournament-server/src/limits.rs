//! Process limits of os resources
//!
use nix::sys::resource::{getrlimit, Resource};
use tokio::sync::{Semaphore, SemaphorePermit};

#[derive(Debug)]
pub struct Limits(LimitsInner);

impl Limits {
    #[inline]
    pub fn new() -> Self {
        Self(LimitsInner::new())
    }

    /// Acquires a single [`File`] descriptor. If one is not avaliable immediately this method
    /// will wait return once one is avaliable. The file descriptor is automatically released when
    /// [`File`] is dropped.
    #[inline]
    pub async fn acquire_file(&self) -> File<'_> {
        self.acquire_files(1).await
    }

    /// Acquires `n` [`File`] descriptors.
    #[inline]
    pub async fn acquire_files(&self, n: u32) -> File<'_> {
        self.0.acquire_files(n).await
    }

    /// Tries to acquire `n` [`File`] descriptors right now. Returns `None` if not enough
    /// are avaliable.
    #[inline]
    pub fn try_acquire_files(&self, n: u32) -> Option<File<'_>> {
        self.0.try_acquire_files(n)
    }
}

#[derive(Debug)]
pub struct File<'a> {
    #[cfg(feature = "limits")]
    permit: SemaphorePermit<'a>,
    #[cfg(not(feature = "limits"))]
    _marker: std::marker::PhantomData<'a>,
}

impl<'a> File<'a> {
    #[inline]
    pub fn forget(self) {
        #[cfg(feature = "limits")]
        self.permit.forget();
    }
}

#[derive(Debug)]
struct LimitsInner {
    nofile: Semaphore,
}

#[cfg(not(feature = "limits"))]
struct LimitsInner;

impl LimitsInner {
    fn new() -> Self {
        let (nofile_soft, _) = getrlimit(Resource::RLIMIT_NOFILE).unwrap();

        let nofile = Semaphore::new(nofile_soft as usize);

        Self { nofile }
    }

    #[inline]
    async fn acquire_files(&self, n: u32) -> File<'_> {
        log::trace!("{} free file descriptors", self.nofile.available_permits());

        File {
            permit: self.nofile.acquire_many(n).await.unwrap(),
        }
    }

    #[inline]
    fn try_acquire_files(&self, n: u32) -> Option<File<'_>> {
        Some(File {
            permit: self.nofile.try_acquire_many(n).ok()?,
        })
    }
}

#[cfg(not(feature = "limits"))]
impl LimitsInner {
    #[inline]
    fn new() -> Self {
        Self
    }

    #[inline]
    async fn acquire_files(&self) -> File<'_> {
        File {
            _marker: std::marker::PhantomData,
        }
    }

    fn try_acquire_files(&self) -> File<'_> {
        File {
            _marker: std::marker::PhantomData,
        }
    }
}
