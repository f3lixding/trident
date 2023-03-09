use crate::rules::{generate_rules_from_examiner, Rule};
use alloc::{boxed::Box, vec::Vec};
use core::{marker::PhantomPinned, pin::Pin, ptr::NonNull};

extern "C" {
    fn turn_on_pump_for_duration(amount: i32);
}

static WATER_FACTOR: f32 = 1.5;

pub struct Examiner {
    water_count: i32,
    latest_humd: i32,
    threshold: i32,
    rules: NonNull<Vec<Box<dyn Rule>>>,
    _pin: PhantomPinned,
}

impl Examiner {
    pub fn get_water_count(self: Pin<&mut Self>) -> i32 {
        self.water_count
    }

    pub fn get_latest_humd(self: &Self) -> &i32 {
        &self.latest_humd
    }

    pub fn update_humd(self: Pin<&mut Self>, humd_reading: i32) {
        // unsafe {
        //     if Pin::get_unchecked_mut(self).latest_humd == 12 {
        //         turn_on_pump_for_duration(0);
        //     }
        // }
    }

    pub fn get_threshold(self: &Self) -> &i32 {
        &self.threshold
    }

    pub fn new(threshold: i32) -> Pin<Box<Self>> {
        // TODO
        // - bind action callback from C side
        let res = Self {
            water_count: 0,
            latest_humd: 12,
            threshold,
            rules: NonNull::dangling(),
            _pin: PhantomPinned,
        };
        let mut boxed = Box::pin(res);
        let rules = generate_rules_from_examiner(&mut boxed);
        let rules = Box::into_raw(Box::new(rules));

        // SAFETY: we are taking the reference of the raw pointer we just created to assign to an
        // examiner's field. We are not really dereferencing here
        unsafe {
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).rules = NonNull::from(&*rules);
        }
        boxed
    }

    fn determine_action(self: &mut Pin<&mut Self>) -> Action {
        Action::Noop
        // SAFETY: we know this is safe because determine_action is a private method
        // and we should only call this after properly initializing rules field
        // unsafe {
        //     match self
        //         .rules
        //         .as_mut()
        //         .iter_mut()
        //         .map(|rule| rule.evaluate())
        //         .find(|has_passed| !has_passed)
        //     {
        //         Some(_) => Action::Noop,
        //         None => {
        //             let water_amount = (self.threshold - self.latest_humd) as f32 * WATER_FACTOR;
        //             let water_amount = water_amount as i32;
        //             Action::Pump(water_amount)
        //         }
        //     }
        // }
    }

    pub fn handle_humd_input(
        mut self: Pin<&mut Self>,
        humd_input: i32,
    ) -> Result<i32, &'static str> {
        self.as_mut().update_humd(humd_input);
        match self.determine_action() {
            Action::Pump(amount) => {
                // water here
                // self.water_count += 1; // might need to think about overflow
                // unsafe {
                //     // TODO: implement real watering logic
                //     // right now it's just turning the lights on and off
                //     turn_on_pump_for_duration(1);

                //     // rule states clean up post watering
                //     self.rules
                //         .as_mut()
                //         .iter_mut()
                //         .for_each(|rule| rule.post_water());
                // };
                // unsafe { turn_on_pump_for_duration(1) };
                Ok(amount)
            }
            Action::Noop => {
                // TODO: implement real watering logic
                // right now it's just turning the lights on and off
                // unsafe { turn_on_pump_for_duration(0) };
                Ok(0)
            }
        }
    }
}

impl Drop for Examiner {
    // this is needed because we want to make sure the Target pointed to by the self-referential
    // NonNull pointer is cleaned up when the Examiner struct is deallocated
    fn drop(&mut self) -> () {
        unsafe { inner_drop(Pin::new_unchecked(self)) };
        fn inner_drop(this: Pin<&mut Examiner>) {
            let rules_ptr = this.rules.as_ptr();
            let _ = Box::from(rules_ptr);
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Noop,
    Pump(i32),
}

#[cfg(test)]
mod tests {
    use crate::rules;

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
        // let threshold = 10;
        // run_test(move |examiner: &mut Pin<Box<Examiner>>| {
        //     examiner.set_threshold(threshold);

        //     assert_eq!(
        //         *examiner.get_threshold(),
        //         threshold,
        //         "Did not return the correct threshold"
        //     );

        //     let over_saturated_input = 12;
        //     let under_saturated_input = 7;

        //     let result = examiner.as_mut().handle_humd_input(over_saturated_input);
        //     assert_eq!(result, Ok(0));

        //     let mut result = Ok(0);

        //     // Simulate 100 readings
        //     for _ in 0..rules::MIN_THRESHOLD_BREACH {
        //         result = examiner.as_mut().handle_humd_input(under_saturated_input);
        //     }
        //     let water_amount = (examiner.threshold - examiner.latest_humd) as f32 * WATER_FACTOR;
        //     let water_amount = water_amount as i32;
        //     assert_eq!(result, Ok(water_amount), "Water amount is incorrect");

        //     // Simulate 1 reading, which should not trigger a watering event
        //     let result = examiner.as_mut().handle_humd_input(under_saturated_input);
        //     // assert_eq!(result, Ok(0), "1 threshold breach should not trigger a watering event");
        // })
    }
}
