use super::*;
use std::fs::TryLockError;

#[test]
fn writer_lock_refuses_second_owner_and_releases_on_drop() {
    let root = crate::TemporaryRoot::new().unwrap();
    let path = root.store_path("lock").unwrap();
    fs::create_dir(&path).unwrap();
    let owner = writer_lock(&path).unwrap();
    assert!(matches!(
        writer_lock(&path),
        Err(crate::Error::WriterLock(TryLockError::WouldBlock))
    ));
    drop(owner);
    let owner = writer_lock(&path).unwrap();
    assert!(matches!(
        writer_lock(&path),
        Err(crate::Error::WriterLock(TryLockError::WouldBlock))
    ));
    drop(owner);
}

#[cfg(target_os = "linux")]
fn lock_trace(file: &File) -> String {
    use std::os::{fd::AsRawFd, unix::fs::MetadataExt};
    let inode = file.metadata().unwrap().ino();
    let locks = fs::read_to_string("/proc/locks").unwrap();
    let matches: Vec<_> = locks
        .lines()
        .filter(|line| line.contains(&format!(":{inode} ")))
        .collect();
    format!(
        "pid={} inode={inode} fd={} locks={matches:?} fdinfo={}",
        std::process::id(),
        file.as_raw_fd(),
        fs::read_to_string(format!("/proc/self/fdinfo/{}", file.as_raw_fd())).unwrap()
    )
}

#[cfg(target_os = "linux")]
#[test]
fn writer_lock_releases_even_with_a_live_duplicate_description() {
    // dup and fork share the same open file description. Hold a safe try_clone
    // to deterministically reproduce the vulnerable lifetime without unsafe fork
    // callbacks in a multithreaded test runner. /proc records the actual flock.
    let root = crate::TemporaryRoot::new().unwrap();
    let path = root.store_path("dup-lock").unwrap();
    fs::create_dir(&path).unwrap();
    let old = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path.join("writer.lock"))
        .unwrap();
    old.try_lock().unwrap();
    let duplicate = old.try_clone().unwrap();
    drop(old); // The previous implementation: its clone still keeps the lock.
    assert!(matches!(
        writer_lock(&path),
        Err(crate::Error::WriterLock(TryLockError::WouldBlock))
    ));
    println!("OLD_CLOSE_ONLY {}", lock_trace(&duplicate));
    // Clean up the intentionally vulnerable control explicitly too: another
    // parallel test may currently be spawning and inheriting this descriptor.
    duplicate.unlock().unwrap();
    drop(duplicate);
    let owner = writer_lock(&path).unwrap();
    let duplicate = owner.0.try_clone().unwrap();
    println!("GUARD_HELD {}", lock_trace(&duplicate));
    drop(owner); // Explicit unlock releases the OFD lock even before clone close.
    println!("GUARD_DROPPED_DUP_LIVE {}", lock_trace(&duplicate));
    let next = writer_lock(&path).unwrap();
    assert!(matches!(
        writer_lock(&path),
        Err(crate::Error::WriterLock(TryLockError::WouldBlock))
    ));
    drop(duplicate); // Closing an old duplicate must not release the new owner.
    assert!(matches!(
        writer_lock(&path),
        Err(crate::Error::WriterLock(TryLockError::WouldBlock))
    ));
    drop(next);
}

#[cfg(target_os = "linux")]
#[test]
fn writer_lock_bounded_parallel_spawn_drop_reopen() {
    use std::sync::{Arc, Barrier};
    let root = crate::TemporaryRoot::new().unwrap();
    let path = root.store_path("spawn-lock").unwrap();
    fs::create_dir(&path).unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let child_barrier = barrier.clone();
    let worker = std::thread::spawn(move || {
        for _ in 0..128 {
            child_barrier.wait();
            let status = std::process::Command::new("/bin/true").status().unwrap();
            assert!(status.success());
            child_barrier.wait();
        }
    });
    let mut failures = Vec::new();
    for round in 0..128 {
        let owner = writer_lock(&path).unwrap();
        barrier.wait();
        drop(owner);
        // No retry/backoff: any acquisition failure fails the bounded test.
        match writer_lock(&path) {
            Ok(owner) => {
                assert!(matches!(
                    writer_lock(&path),
                    Err(crate::Error::WriterLock(TryLockError::WouldBlock))
                ));
                drop(owner);
            }
            Err(error) => {
                let file = File::open(path.join("writer.lock")).unwrap();
                failures.push(format!(
                    "round={round} error={error:?} {}",
                    lock_trace(&file)
                ));
            }
        }
        barrier.wait();
    }
    worker.join().unwrap();
    assert!(failures.is_empty(), "{failures:#?}");
    println!("LOCK_SPAWN rounds=128 acquisition_retries=0 single_writer_gate=passed");
}
