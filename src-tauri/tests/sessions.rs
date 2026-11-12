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
    // The old loop wakes up and exits; it is not current, so it must not
    // tell the GUI that listening stopped.
    assert!(!s.end(&old), "a stale loop's end is not reported");
    assert!(new.load(Ordering::SeqCst), "the new loop keeps running");
    assert!(s.is_running());
    // The new loop's own exit (device dropout) does end the session.
    assert!(s.end(&new));
    assert!(!s.is_running());
}

#[test]
fn stop_marks_the_session_as_stopped_not_lost() {
    // Stop closes the audio channel too; the loop reads its own run flag to
    // tell a Stop (false) from a device loss (true).
    let s = Sessions::default();
    let (_, run) = s.begin(|| Ok(())).unwrap();
    assert!(
        run.load(Ordering::SeqCst),
        "a device loss now would be reported"
    );
    s.stop();
    assert!(
        !run.load(Ordering::SeqCst),
        "after Stop the closed channel is no loss"
    );
    assert!(!s.is_running());
}
