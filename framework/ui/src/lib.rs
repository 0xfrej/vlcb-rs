#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![deny(unsafe_code)]

use core::marker::PhantomData;
use embedded_simple_ui::{led::{effects::{blink, pulse, LedEffect}, Led}, switch::Switch};
use embedded_time::{duration::Milliseconds, Clock, Instant};
use vlcb_defs::ModuleMode;

pub mod config {
    pub const SW_LONG_HOLD_MS: u16 = 6000;
    pub const SW_SHORT_RANGE_HOLD_MS_LOW: u16 = 1000;
    pub const SW_SHORT_RANGE_HOLD_MS_HIGH: u16 = 2000;
    pub const SW_VERY_SHORT_HOLD_MS: u16 = 500;
    pub const SETUP_MODE_BLINK_RATE_HZ: u8 = 1;
    pub const ACTIVITY_PULSE_MS: u8 = 5;
}

pub trait VlcbUi<C: Clock> {
    /// Poll the UI for changes
    fn poll(&mut self, now: Instant<C>);

    /// Indicate whether the main switch is pressed
    fn is_main_sw_pressed(&self) -> bool;

    /// Indicate module activity
    ///
    /// Produces a short pulse on the green led.
    /// Module must wait for the next poll on the LED instance
    fn indicate_activity(&mut self);
}

pub struct HardwareUi<LED: Led<C>, SW: Switch<C>, C: Clock> {
    led_green: LED,
    led_yellow: LED,
    main_switch: SW,
    _clock: PhantomData<C>,
}

impl<LED: Led<C>, SW: Switch<C>, C: Clock> HardwareUi<LED, SW, C> {
    pub fn new(led_green: LED, led_yellow: LED, main_switch: SW) -> Self {
        let mut led_green = led_green;
        let mut led_yellow = led_yellow;
        led_green.clear_effect();
        led_green.turn_off();
        led_yellow.clear_effect();
        led_yellow.turn_off();

        Self {
            led_green,
            led_yellow,
            main_switch,
            _clock: PhantomData,
        }
    }

    pub fn indicate_mode(&mut self, mode: ModuleMode) {
        match mode {
            ModuleMode::Normal => {
                self.led_yellow.turn_on();
                self.led_green.turn_off();
            },
            ModuleMode::Uninitialized => {
                self.led_yellow.turn_off();
                self.led_green.turn_on();
            },
            ModuleMode::InSetup => {
                self.led_yellow.set_effect(LedEffect::new(blink::<C>(config::SETUP_MODE_BLINK_RATE_HZ)));
                self.led_green.turn_off();
            },
            _ => {},
        }
    }

    /// Indicate whether the user has requested a reset
    ///
    /// TODO: this should be either part of check_user_requested_action or something else
    pub fn is_reset_requested(&self) -> bool {
        // return pushButton.isPressed() && pushButton.getCurrentStateDuration() > SW_TR_HOLD;
        // TODO: the code must react with the switch still pressed, that should be a new feature in the library
        self.main_switch.pressed_for().map_or(false, |d| {
            d > Milliseconds::<C::T>::new(C::T::from(config::SW_LONG_HOLD_MS as u32))
        })
    }

    /// Indicate whether the main switch is pressed
    pub fn is_main_sw_pressed(&self) -> bool {
        todo!()
    }

    /// Check if user requested an action
    fn check_user_requested_action(&mut self) {
        if self.main_switch.has_changed() && self.main_switch.is_released() {
            let press_time = self.main_switch.prev_state_lasted_for();

            // TODO: these requests should be handled somehow probably instead of doing it this way we should have a flag and then the client
            // will "serve" the request and reset it?
            if press_time > Milliseconds::<C::T>::new(C::T::from(config::SW_LONG_HOLD_MS as u32)) {
                // controller->putAction(ACT_CHANGE_MODE);
                return
            }

            if press_time >= Milliseconds::<C::T>::new(C::T::from(config::SW_SHORT_RANGE_HOLD_MS_LOW as u32)) &&
                press_time < Milliseconds::<C::T>::new(C::T::from(config::SW_SHORT_RANGE_HOLD_MS_HIGH as u32)) {
                // controller->putAction(ACT_RENEGOTIATE);
                return
            }

            if press_time < Milliseconds::<C::T>::new(C::T::from(config::SW_VERY_SHORT_HOLD_MS as u32)) {
                // controller->putAction(ACT_START_CAN_ENUMERATION);
                return
            }
            todo!()
        }
    }
}

impl<LED: Led<C>, SW: Switch<C>, C: Clock> VlcbUi<C> for HardwareUi<LED, SW, C> {
    fn poll(&mut self, now: Instant<C>) {
        self.led_green.poll(now);
        self.led_yellow.poll(now);
        self.main_switch.poll(now);
    }

    fn is_main_sw_pressed(&self) -> bool {
        self.main_switch.is_pressed()
    }

    fn indicate_activity(&mut self) {
        self.led_green.set_effect(LedEffect::new(pulse::<C>(config::ACTIVITY_PULSE_MS as u16)));
    }
}
