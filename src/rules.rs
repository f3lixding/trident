use crate::threshold_examiner::Examiner;
use alloc::vec;
use alloc::{boxed::Box, vec::Vec};
use core::mem;
use core::{pin::Pin, ptr::NonNull};

pub static MIN_THRESHOLD_BREACH: i32 = 1000;
/// This trait is here to make runtime polymorphism happen.
/// It's here to make the invocation more ergonomics.
/// But it does not necessarily make it easier to be extensible.
pub trait Rule {
    fn evaluate(&mut self) -> bool;
    fn post_water(&mut self) {}
}

/// Rule for examining moisture level
pub struct MoistureRule<'a> {
    threshold: &'a i32,
    latest_humd: &'a i32,
    threshold_breach_count: i32,
}

impl<'a> Rule for MoistureRule<'a> {
    fn evaluate(&mut self) -> bool {
        if self.threshold > self.latest_humd {
            self.threshold_breach_count += 1;
            if self.threshold_breach_count >= MIN_THRESHOLD_BREACH {
                return true;
            }
        }
        false
    }

    fn post_water(&mut self) {
        self.threshold_breach_count = 0;
    }
}

/// Rule for examining if it's time allowed for watering
pub struct TimeRule {
    window_start: i32,
    window_end: i32,
}

impl Rule for TimeRule {
    /// This is relatively complicated since the utime on the pico resets every time on boot
    /// A better way to do this would be to ask the onboard server to make a call on startup to get
    /// the real time and count from there
    fn evaluate(&mut self) -> bool {
        true
    }
}

/// This function needs to be called from the top level **after** Examiner has been instantiated
pub fn generate_rules_from_examiner<'a>(
    examiner: *mut Examiner,
) -> Vec<Box<dyn Rule + 'a>> {
    let examiner_mut_ref = unsafe {
        &mut *examiner
    };
    let threshold_ref = examiner_mut_ref.get_threshold();
    let latest_humd_ref = examiner_mut_ref.get_latest_humd();
    vec![
        Box::new(MoistureRule {
            threshold: threshold_ref,
            latest_humd: latest_humd_ref,
            threshold_breach_count: 0,
        }),
        Box::new(TimeRule {
            window_start: 9,
            window_end: 21,
        }),
    ]
}
