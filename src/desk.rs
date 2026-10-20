//! What is happening at the desk: where the dials are set, what the corridor is
//! showing, and what a key does about it. See `specs/0001-the-night.md`.
//!
//! Kept apart from the drawing, so a night can be played without a window.

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::corridor;
use crate::night::{Night, Refused, Standing};
use crate::rule::{Asking, Freed, Rule, Verdict, SHIFT_MOST, STRIDE_MOST};

/// How fast a door swings, as a fraction of the way per second.
pub const SWINGS_AT: f32 = 1.0 / corridor::SWING_SECONDS;

/// Which rule the corridor is showing.
///
/// The button is not on the dials, so pressing it has to change what you see or
/// it is a key that claims to move everybody and moves nobody.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Showing {
    Dials,
    Primes,
}

pub struct Desk {
    pub rule: Rule,
    pub showing: Showing,
    pub night: Night,
    /// How far open each drawn room's door is, room 1 first. The corridor shows
    /// the dials as they are set rather than as they were last committed, so
    /// turning one says what it would free before you answer with it.
    open: Vec<f32>,
    rng: StdRng,
}

impl Desk {
    pub fn new(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let night = Night::new(&mut rng);

        Self {
            // standing still, so the night opens on a full hotel
            rule: Rule::new(1, 0),
            showing: Showing::Dials,
            night,
            open: vec![0.0; corridor::DRAWN as usize],
            rng,
        }
    }

    /// Whether a drawn room is empty under whatever is being shown.
    pub fn is_free(&self, room: u64) -> bool {
        match self.showing {
            Showing::Dials => corridor::free_rooms(&self.rule).contains(&room),
            Showing::Primes => corridor::free_after_primes().contains(&room),
        }
    }

    /// How far open a door is, from 0 to 1.
    pub fn open_by(&self, room: u64) -> f32 {
        self.open
            .get(room as usize - 1)
            .copied()
            .unwrap_or_default()
    }

    /// How far open a door should be, which is all the way when its room is
    /// empty and not at all when it is not.
    fn wants(&self, room: u64) -> f32 {
        if self.is_free(room) {
            1.0
        } else {
            0.0
        }
    }

    /// Walks every door towards where the dials say it should be.
    pub fn tick(&mut self, dt: f32) {
        let step = SWINGS_AT * dt;
        let wants: Vec<f32> = (1..=corridor::DRAWN).map(|room| self.wants(room)).collect();

        for (open, want) in self.open.iter_mut().zip(wants) {
            *open = if *open < want {
                (*open + step).min(want)
            } else {
                (*open - step).max(want)
            };
        }
    }

    /// Whether the doors have finished moving, so a picture of them is settled.
    pub fn settled(&self) -> bool {
        (1..=corridor::DRAWN).all(|room| (self.open_by(room) - self.wants(room)).abs() < 1e-3)
    }

    pub fn turn_stride(&mut self, by: i64) {
        self.showing = Showing::Dials;
        self.rule.stride = step(self.rule.stride, by, 0, STRIDE_MOST);
    }

    pub fn turn_shift(&mut self, by: i64) {
        self.showing = Showing::Dials;
        self.rule.shift = step(self.rule.shift, by, 0, SHIFT_MOST);
    }

    /// Answers the coach at the door with the dials.
    pub fn answer(&mut self) -> Verdict {
        self.night.answer(self.rule, &mut self.rng)
    }

    /// Answers it with the one rule that is not on the dials, and shows it.
    pub fn answer_with_primes(&mut self) -> Verdict {
        self.showing = Showing::Primes;
        self.night.answer_with_primes(&mut self.rng)
    }

    pub fn is_over(&self) -> bool {
        self.night.standing == Standing::Over
    }

    /// Starts a fresh night, which is the only thing a key does once one is
    /// over.
    pub fn again(&mut self) {
        self.night = Night::new(&mut self.rng);
        self.rule = Rule::new(1, 0);
        self.showing = Showing::Dials;
    }

    /// What the dials say, in words.
    pub fn rule_says(&self) -> String {
        if self.showing == Showing::Primes {
            return String::from(
                "room n becomes room 2 to the n, and seat s of coach c becomes p to the s",
            );
        }

        let moving = format!(
            "room n becomes room {}n + {}",
            self.rule.stride, self.rule.shift
        );

        let frees = match self.rule.frees() {
            Freed::None if self.rule.stride == 0 => {
                String::from("and puts every guest in the one room")
            }
            Freed::None => String::from("and frees nothing"),
            Freed::Counted(1) => String::from("and frees 1 room"),
            Freed::Counted(rooms) => format!("and frees {} rooms", rooms),
            Freed::Endless if self.rule.stride == 2 => {
                String::from("and frees every odd room, which is endlessly many")
            }
            Freed::Endless => format!(
                "and frees {} rooms in every {}, which is endlessly many",
                self.rule.stride - 1,
                self.rule.stride
            ),
        };

        format!("{} {}", moving, frees)
    }

    /// What is at the door, in words.
    pub fn asking_says(&self) -> String {
        match self.night.asking {
            Asking::Guests(1) => String::from("One guest is at the door."),
            Asking::Guests(many) => format!("A coach of {} is at the door.", many),
            Asking::ACoach => {
                String::from("A coach with a seat for every counting number is at the door.")
            }
            Asking::Coaches => String::from(
                "A coach for every counting number, each with a seat for every counting number.",
            ),
        }
    }

    /// How the last answer went, in words.
    pub fn standing_says(&self) -> String {
        match self.night.refused {
            None => String::from("Set the dials, then enter to answer. P moves everyone to a prime power."),
            Some(Refused::Shared) => String::from(
                "Two guests in the one room. They are still in the car park. Enter starts another night.",
            ),
            Some(Refused::TooFew) => String::from(
                "Not enough rooms for them. They are still in the car park. Enter starts another night.",
            ),
            Some(Refused::OneAtATime) => String::from(
                "Rooms enough, but handed out lowest first you never finish the first coach. Enter starts another night.",
            ),
        }
    }
}

fn step(from: u64, by: i64, least: u64, most: u64) -> u64 {
    let next = from as i64 + by;

    next.clamp(least as i64, most as i64) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desk() -> Desk {
        Desk::new(4)
    }

    #[test]
    fn a_night_opens_on_a_full_hotel() {
        let desk = desk();

        assert_eq!(desk.rule, Rule::new(1, 0));
        assert!((1..=corridor::DRAWN).all(|room| !desk.is_free(room)));
        assert!(desk.settled());
    }

    #[test]
    fn the_dials_stop_at_both_ends() {
        let mut desk = desk();

        for _ in 0..40 {
            desk.turn_stride(-1);
            desk.turn_shift(-1);
        }
        assert_eq!(desk.rule, Rule::new(0, 0));

        for _ in 0..80 {
            desk.turn_stride(1);
            desk.turn_shift(1);
        }
        assert_eq!(desk.rule, Rule::new(STRIDE_MOST, SHIFT_MOST));
    }

    #[test]
    fn turning_a_dial_opens_the_doors_it_frees() {
        let mut desk = desk();
        desk.turn_stride(1);

        // the dials show what they would do before the coach is answered
        assert!(desk.is_free(1), "the first room did not come free");
        assert!(!desk.is_free(2));
        assert!(!desk.settled(), "the doors should still be swinging");

        desk.tick(corridor::SWING_SECONDS * 1.2);

        assert!(desk.settled());
        assert_eq!(desk.open_by(1), 1.0);
        assert_eq!(desk.open_by(2), 0.0);
    }

    #[test]
    fn a_door_swings_rather_than_jumping() {
        let mut desk = desk();
        desk.turn_stride(1);
        desk.tick(corridor::SWING_SECONDS * 0.5);

        let half = desk.open_by(1);
        assert!(half > 0.0 && half < 1.0, "it jumped to {}", half);
    }

    #[test]
    fn a_door_shuts_again_when_the_dial_goes_back() {
        let mut desk = desk();
        desk.turn_stride(1);
        desk.tick(corridor::SWING_SECONDS * 1.2);
        assert_eq!(desk.open_by(1), 1.0);

        desk.turn_stride(-1);
        desk.tick(corridor::SWING_SECONDS * 1.2);

        assert_eq!(desk.open_by(1), 0.0);
    }

    #[test]
    fn answering_well_houses_them_and_brings_the_next() {
        let mut desk = desk();
        desk.turn_shift(1);

        assert_eq!(desk.answer(), Verdict::Taken);
        assert_eq!(desk.night.housed, 1);
        assert!(!desk.is_over());
    }

    #[test]
    fn answering_badly_ends_the_night() {
        let mut desk = desk();

        assert_eq!(desk.answer(), Verdict::TooFew);
        assert!(desk.is_over());
        assert!(desk.standing_says().contains("car park"));
    }

    #[test]
    fn the_button_answers_anything() {
        let mut desk = desk();

        for _ in 0..12 {
            assert_eq!(desk.answer_with_primes(), Verdict::Taken);
            assert!(!desk.is_over());
        }
        assert_eq!(desk.night.housed, 12);
    }

    #[test]
    fn another_night_puts_everyone_back() {
        let mut desk = desk();
        desk.turn_stride(2);
        desk.answer();
        desk.again();

        assert_eq!(desk.rule, Rule::new(1, 0));
        assert_eq!(desk.night.housed, 0);
        assert!(!desk.is_over());
    }

    #[test]
    fn the_dials_say_what_they_do() {
        let mut desk = desk();
        assert!(desk.rule_says().contains("frees nothing"));

        desk.turn_shift(3);
        assert!(desk.rule_says().contains("frees 3 rooms"));

        desk.turn_shift(-2);
        assert!(
            desk.rule_says().contains("frees 1 room"),
            "{}",
            desk.rule_says()
        );

        desk.turn_shift(-1);
        desk.turn_stride(1);
        assert!(desk.rule_says().contains("every odd room"));

        desk.turn_stride(1);
        assert!(desk.rule_says().contains("2 rooms in every 3"));

        desk.turn_stride(-3);
        assert!(desk.rule_says().contains("every guest in the one room"));
    }

    #[test]
    fn the_button_changes_what_the_corridor_shows() {
        let mut desk = desk();
        desk.turn_stride(1);
        desk.tick(corridor::SWING_SECONDS * 1.5);
        assert!(desk.is_free(3), "doubling frees the odd rooms");

        desk.answer_with_primes();

        assert_eq!(desk.showing, Showing::Primes);

        // the residents take the powers of two and the coaches take the powers
        // of every odd prime, so the corridor refills rather than emptying
        assert!(!desk.is_free(2), "2 is 2 to the 1");
        assert!(!desk.is_free(3), "3 is the first coach's");
        assert!(!desk.is_free(5), "5 is the second coach's");
        assert!(desk.is_free(1), "nobody is in room 1");
        assert!(desk.is_free(6), "6 is no prime's power");
    }

    #[test]
    fn turning_a_dial_takes_the_corridor_back_off_the_button() {
        let mut desk = desk();
        desk.answer_with_primes();
        assert_eq!(desk.showing, Showing::Primes);

        desk.turn_shift(1);

        assert_eq!(desk.showing, Showing::Dials);
    }

    #[test]
    fn the_button_says_what_it_does() {
        let mut desk = desk();
        desk.answer_with_primes();

        assert!(
            desk.rule_says().contains("2 to the n"),
            "{}",
            desk.rule_says()
        );
    }
}
