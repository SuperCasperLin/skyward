//! The lock screen's code, worked out away from the screen's own thread:
//! stretching a code with scrypt takes long enough to be felt as a stutter
//! if the frame wait is spent on it, so it goes to a thread of its own and
//! the answer comes back through a pipe, as decryption's does.

use anyhow::{Context, Result};
use skyward_crypto::pin;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

/// What the thread was asked for, and what it found.
pub enum Done {
    /// The code the code page took, as the line to keep, or why it is not
    /// one to keep.
    Saved(Result<String, String>),
    /// Whether what was typed on the lock screen is the code kept.
    Checked(bool),
}

pub struct Worker {
    done: Receiver<Done>,
    /// Readable once the thread has finished.
    wake: OwnedFd,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Keep `code`, hashed.
    pub fn save(code: String) -> Result<Worker> {
        Worker::start(move || Done::Saved(pin::hash(&code).map_err(|e| format!("{e:#}"))))
    }

    /// Check `typed` against the line kept for the code.
    pub fn check(recorded: String, typed: String) -> Result<Worker> {
        Worker::start(move || Done::Checked(pin::check(&recorded, &typed)))
    }

    fn start(work: impl FnOnce() -> Done + Send + 'static) -> Result<Worker> {
        let mut fds = [0; 2];
        if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC | libc::O_NONBLOCK) } != 0 {
            return Err(std::io::Error::last_os_error()).context("make the code thread's wake pipe");
        }
        let (wake, signal) = unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) };
        let (tx, done) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("pin".into())
            .spawn(move || {
                let answer = work();
                // The screen only stops listening once it has let the
                // thread go.
                let _ = tx.send(answer);
                unsafe { libc::write(signal.as_raw_fd(), [1u8].as_ptr().cast(), 1) };
            })
            .context("start the code thread")?;
        Ok(Worker { done, wake, thread: Some(thread) })
    }

    pub fn fd(&self) -> RawFd {
        self.wake.as_raw_fd()
    }

    /// What the thread found, once it has.
    pub fn ended(&self) -> Option<Done> {
        let mut drain = [0u8; 8];
        while unsafe { libc::read(self.wake.as_raw_fd(), drain.as_mut_ptr().cast(), drain.len()) } > 0 {}
        self.done.try_recv().ok()
    }
}

impl Drop for Worker {
    /// The screen lets the thread go once its work has ended.
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            thread.join().expect("the code thread does not panic");
        }
    }
}
