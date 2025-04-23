use core::sync::atomic::{AtomicU8, Ordering};

const NONE_VALUE: u8 = 255;

pub struct OptionalAtomicU8 {
    inner: AtomicU8,
}

impl OptionalAtomicU8 {
    pub const fn new(val: Option<u8>) -> Self {
        let raw = match val {
            Some(v) => v,
            None => NONE_VALUE,
        };
        Self {
            inner: AtomicU8::new(raw),
        }
    }

    pub fn load(&self) -> Option<u8> {
        let val = self.inner.load(Ordering::SeqCst);
        if val == NONE_VALUE {
            None
        } else {
            Some(val)
        }
    }

    pub fn store(&self, val: Option<u8>) {
        self.inner
            .store(val.unwrap_or(NONE_VALUE), Ordering::SeqCst);
    }
}
