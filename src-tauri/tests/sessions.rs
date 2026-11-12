//! Q2-11 (audit 9): the listening session's running state.

use std::sync::atomic::Ordering;
use voxxa_lib::Sessions;

#[test]
fn a_failed_start_does_not_leave_already_running() {
    let s = Sessions::default();
    // AudioEngine::start fails, e.g. an unplugged microphone.
    let failed = s.begin(|| Err::<(), _>("device not found".into()));
    assert_eq!(failed.unwrap_err(), "device not found");
    assert!(!s.is_running());
    // The next Start works.
    let (_, run) = s.begin(|| Ok(())).unwrap();
    assert!(s.is_running() && run.load(Ordering::SeqCst));
    assert_eq!(s.begin(|| Ok(())).unwrap_err(), "Already running");
}

#[test]
fn a_slow_old_loop_cannot_end_a_new_session() {
    let s = Sessions::default();
    let (_, old) = s.begin(|| Ok(())).unwrap();
    s.stop();
    assert!(!old.load(Ordering::SeqCst), "stop ends the old loop");
    // Start again before the old loop (busy transcribing) has noticed.
    let (_, new) = s.begin(|| Ok(())).unwrap();
    // The old loop wakes up and exits.
    s.end(&old);
    assert!(new.load(Ordering::SeqCst), "the new loop keeps running");
    assert!(s.is_running());
    // The new loop's own exit (device dropout) does end the session.
    s.end(&new);
    assert!(!s.is_running());
}
