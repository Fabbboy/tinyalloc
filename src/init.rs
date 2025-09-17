use std::sync::atomic::{
  AtomicBool,
  Ordering,
};

struct TeardownGuard;

impl Drop for TeardownGuard {
  fn drop(&mut self) {
    TEARING_DOWN.with(|flag| {
      flag.store(true, Ordering::SeqCst);
    });
  }
}

#[inline]
pub fn td_register() {
  _GUARD.with(|_| {});
}

#[inline]
pub fn is_td() -> bool {
  TEARING_DOWN
    .try_with(|flag| flag.load(Ordering::SeqCst))
    .unwrap_or(true)
}

thread_local! {
  static _GUARD: TeardownGuard = TeardownGuard {};
  pub static TEARING_DOWN: AtomicBool = AtomicBool::new(false);
}
