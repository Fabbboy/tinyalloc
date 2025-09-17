use std::sync::atomic::AtomicBool;

static TEARDOWN: AtomicBool = AtomicBool::new(false);
struct TeardownGuard {}

impl Drop for TeardownGuard {
  fn drop(&mut self) {

  }
}

thread_local! {
  static _GUARD: TeardownGuard = TeardownGuard {};
}