use crate::rules::{generate_rules_from_examiner, Rule};
use alloc::vec;
use alloc::{boxed::Box, vec::Vec};
use core::{pin::Pin, ptr::NonNull};

extern "C" {
    fn turn_on_pump_for_duration(amount: i32);
}

static WATER_FACTOR: f32 = 1.5;

pub struct Examiner {
    water_count: i32,
    latest_humd: i32,
    threshold: i32,
    rules: NonNull<Vec<Box<dyn Rule>>>,
}

impl Examiner {
    pub fn get_water_count(&self) -> i32 {
        self.water_count
    }

    pub fn get_latest_humd(&self) -> &i32 {
        &self.latest_humd
    }

    pub fn set_threshold(&mut self, threshold: i32) {
        self.threshold = threshold;
    }

    pub fn get_threshold(&self) -> &i32 {
        &self.threshold
    }

    pub fn new(threshold: i32) -> Pin<Box<Self>> {
        // TODO
        // - bind action callback from C side
        let res = Self {
            water_count: 0,
            latest_humd: 0,
            threshold,
            rules: NonNull::dangling(),
        };
        let mut boxed = Box::pin(res);
        let examiner = NonNull::from(&boxed);
        let rules = generate_rules_from_examiner(examiner);
        let rules = Box::into_raw(Box::new(rules));

        // SAFETY: we are taking the reference of the raw pointer we just created to assign to an
        // examiner's field. We are not really dereferencing here
        unsafe { boxed.rules = NonNull::from(&*rules) };
        boxed
    }

    fn determine_action(&mut self) -> Action {
        // SAFETY: we know this is safe because determine_action is a private method
        // and we should only call this after properly initializing rules field
        unsafe {
            match self
                .rules
                .as_mut()
                .iter_mut()
                .map(|rule| rule.evaluate())
                .find(|has_passed| !has_passed)
            {
                Some(_) => Action::Noop,
                None => {
                    let water_amount = (self.threshold - self.latest_humd) as f32 * WATER_FACTOR;
                    let water_amount = water_amount as i32;
                    Action::Pump(water_amount)
                }
            }
        }
    }

    pub fn handle_humd_input(&mut self, humd_input: i32) -> Result<i32, &'static str> {
        self.latest_humd = humd_input;
        match self.determine_action() {
            Action::Pump(amount) => {
                // water here
                self.water_count += 1; // might need to think about overflow
                unsafe {
                    turn_on_pump_for_duration(amount);

                    // rule states clean up post watering
                    self.rules
                        .as_mut()
                        .iter_mut()
                        .for_each(|rule| rule.post_water());
                };
                Ok(amount)
            }
            Action::Noop => Ok(0),
        }
    }
}

impl Drop for Examiner {
    // this is needed because we want to make sure the Target pointed to by the self-referential
    // NonNull pointer is cleaned up when the Examiner struct is deallocated
    fn drop(&mut self) -> () {
        // deref the rules pointer
        let rules_ptr = self.rules.as_ptr();
        // and then drop it
        let _ = Box::from(rules_ptr);
    }
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Noop,
    Pump(i32),
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::panic;
    static TEST_THRESHOLD: i32 = 10;

    // mocking extern function for test
    #[no_mangle]
    fn turn_on_pump_for_duration(amount: i32) {
        println!("pump got turned on for {}", amount);
    }

    fn run_test<T>(test: T)
    where
        T: FnOnce(&mut Pin<Box<Examiner>>) -> () + panic::UnwindSafe,
    {
        let mut examiner = Examiner::new(TEST_THRESHOLD);
        test(&mut examiner);
    }

    #[test]
    fn test_process_humd_input() {
        let threshold = 10;
        run_test(move |examiner: &mut Pin<Box<Examiner>>| {
            examiner.set_threshold(threshold);

            assert_eq!(
                *examiner.get_threshold(),
                threshold,
                "Did not return the correct threshold"
            );

            let over_saturated_input = 12;
            let under_saturated_input = 7;

            let result = examiner.handle_humd_input(over_saturated_input);
            assert_eq!(result, Ok(0));

            let mut result = Ok(0);

            // Simulate 100 readings
            for i in 0 .. 100 {
                result = examiner.handle_humd_input(under_saturated_input);
            }
            let water_amount = (examiner.threshold - examiner.latest_humd) as f32 * WATER_FACTOR;
            let water_amount = water_amount as i32;
            assert_eq!(result, Ok(water_amount), "Water amount is incorrect");

            // Simulate 1 reading, which should not trigger a watering event
            let result = examiner.handle_humd_input(under_saturated_input);
            assert_eq!(result, Ok(0), "1 threshold breach should not trigger a watering event");
        })
    }
}
