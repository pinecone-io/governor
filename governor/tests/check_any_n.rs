use governor::{
    clock::{Clock, FakeRelativeClock},
    Quota, RateLimiter,
};
use nonzero_ext::nonzero;
use std::time::Duration;

#[test]
fn check_any_n_admits_available() {
    let clock = FakeRelativeClock::default();
    let lb = RateLimiter::direct_with_clock(Quota::per_second(nonzero!(10u32)), clock.clone());

    let (actual, _) = lb.check_any_n(nonzero!(10u32)).unwrap();
    assert_eq!(actual, 10);

    assert!(lb.check_any_n(nonzero!(5u32)).is_err());

    clock.advance(Duration::from_millis(500));
    let (actual, _) = lb.check_any_n(nonzero!(10u32)).unwrap();
    assert_eq!(actual, 5);
}

#[test]
fn check_any_n_partial_vending() {
    let clock = FakeRelativeClock::default();
    let lb = RateLimiter::direct_with_clock(Quota::per_second(nonzero!(10u32)), clock.clone());

    let (actual, _) = lb.check_any_n(nonzero!(7u32)).unwrap();
    assert_eq!(actual, 7);

    // Ask for 10, only 3 available
    let (actual, _) = lb.check_any_n(nonzero!(10u32)).unwrap();
    assert_eq!(actual, 3);

    assert!(lb.check_any_n(nonzero!(10u32)).is_err());
}

#[test]
fn check_any_n_never_exceeds_burst() {
    let clock = FakeRelativeClock::default();
    let lb = RateLimiter::direct_with_clock(Quota::per_second(nonzero!(5u32)), clock.clone());

    let (actual, _) = lb.check_any_n(nonzero!(100u32)).unwrap();
    assert_eq!(actual, 5);

    assert!(lb.check_any_n(nonzero!(1u32)).is_err());
}

#[test]
fn check_any_n_single_token_request() {
    let clock = FakeRelativeClock::default();
    let lb = RateLimiter::direct_with_clock(Quota::per_second(nonzero!(10u32)), clock);

    let (actual, _) = lb.check_any_n(nonzero!(1u32)).unwrap();
    assert_eq!(actual, 1);
}

#[test]
fn check_any_n_gradual_refill() {
    let clock = FakeRelativeClock::default();
    let lb = RateLimiter::direct_with_clock(Quota::per_second(nonzero!(10u32)), clock.clone());

    let (actual, _) = lb.check_any_n(nonzero!(10u32)).unwrap();
    assert_eq!(actual, 10);

    // At 10 tokens/sec, we get 1 token per 100ms
    for _ in 0..5 {
        clock.advance(Duration::from_millis(100));
        let (actual, _) = lb.check_any_n(nonzero!(10u32)).unwrap();
        assert_eq!(actual, 1);
    }
}

#[test]
fn check_any_n_keyed_partial() {
    let clock = FakeRelativeClock::default();
    let lb = RateLimiter::dashmap_with_clock(Quota::per_second(nonzero!(10u32)), clock.clone());

    let (actual, _) = lb.check_key_any_n(&"user1", nonzero!(7u32)).unwrap();
    assert_eq!(actual, 7);

    let (actual, _) = lb.check_key_any_n(&"user1", nonzero!(10u32)).unwrap();
    assert_eq!(actual, 3);

    // user2 has independent capacity
    let (actual, _) = lb.check_key_any_n(&"user2", nonzero!(10u32)).unwrap();
    assert_eq!(actual, 10);
}

#[test]
fn check_any_n_returns_not_until_when_rate_limited() {
    let clock = FakeRelativeClock::default();
    let lb = RateLimiter::direct_with_clock(Quota::per_second(nonzero!(10u32)), clock.clone());

    let _ = lb.check_any_n(nonzero!(10u32)).unwrap();

    let result = lb.check_any_n(nonzero!(1u32));
    assert!(result.is_err());

    let not_until = result.unwrap_err();
    let wait_time = not_until.wait_time_from(clock.now());
    assert!(wait_time > Duration::ZERO);

    clock.advance(wait_time);
    assert!(lb.check_any_n(nonzero!(1u32)).is_ok());
}
