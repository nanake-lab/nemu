mod common;
use common::*;
use core_runtime::clock::*;

#[test]
fn sys23_capture_once_completion_is_separate_and_does_not_retime_process() {
    let mut clock = Clock::start(sample(0), 10, bounds()).unwrap();
    let mut source = Source(sample(20), 0);
    let process = clock.capture(&mut source).unwrap();
    assert_eq!(source.1, 1);
    assert_eq!(process.ns(), 30);
    source.0 = sample(50);
    let completion = clock.completion(&process, &mut source).unwrap();
    assert_eq!(completion.wall_ns, 150);
    assert_eq!(process.ns(), 30);
    assert_eq!(source.1, 2);
}
#[test]
fn sys23_restart_uses_bounded_wall_time_and_refuses_unknown_downtime() {
    let before = sample(500);
    let mut after = sample(0);
    after.boot = "new-boot".into();
    after.wall_ns = 1600;
    let mut restored = Clock::restore(before.clone(), 100, after.clone(), bounds()).unwrap();
    let mut source = Source(after.clone(), 0);
    source.0.monotonic_ns = 10;
    source.0.wall_ns += 10;
    assert_eq!(restored.capture(&mut source).unwrap().ns(), 1110);
    after.uncertainty_ns = None;
    assert_eq!(
        Clock::restore(before, 100, after, bounds()).unwrap_err(),
        ClockFault::Untrusted
    );
}
#[test]
fn sys23_backward_jump_epoch_and_overflow_are_faults_without_plus_one_repair() {
    let start = sample(50);
    for (bad, expected) in [
        (sample(49), ClockFault::Backwards),
        (
            {
                let mut s = sample(60);
                s.wall_ns = 149;
                s
            },
            ClockFault::Backwards,
        ),
        (
            {
                let mut s = sample(60);
                s.wall_ns = 1000;
                s
            },
            ClockFault::Jump,
        ),
        (
            {
                let mut s = sample(60);
                s.epoch = "other".into();
                s
            },
            ClockFault::EpochChanged,
        ),
        (
            {
                let mut s = sample(60);
                s.uncertainty_ns = Some(2);
                s
            },
            ClockFault::Untrusted,
        ),
        (sample(100_000), ClockFault::Jump),
    ] {
        let mut clock = Clock::start(start.clone(), 50, bounds()).unwrap();
        assert_eq!(clock.capture(&mut Source(bad, 0)).unwrap_err(), expected);
        assert_eq!(clock.capture(&mut Source(sample(60), 0)).unwrap().ns(), 60);
    }
    let mut clock = Clock::start(sample(0), u64::MAX, bounds()).unwrap();
    assert_eq!(
        clock.capture(&mut Source(sample(1), 0)).unwrap_err(),
        ClockFault::Overflow
    );
}
