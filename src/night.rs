//! The coaches that keep coming, and what happens when you answer one. See
//! `specs/0001-the-night.md`.
//!
//! Arithmetic and a counter. Nothing here draws anything.

use rand::Rng;

use crate::rule::{Asking, Rule, Verdict};

/// How many a coachload is, at the smallest and the largest.
pub const COACH_FEWEST: u64 = 8;
pub const COACH_MOST: u64 = 60;

/// How the night goes: a single guest, then coachloads, then one endless coach,
/// then endlessly many of them, and round again from the coachloads.
///
/// The hardest kind comes back rather than ending the night, so the prime power
/// is a tool you keep needing and not a finale you reach once.
const SHAPE: [Kind; 5] = [
    Kind::One,
    Kind::Coachload,
    Kind::Coachload,
    Kind::ACoach,
    Kind::Coaches,
];

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Kind {
    One,
    Coachload,
    ACoach,
    Coaches,
}

/// What a refusal was about, said in the rule's own terms.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Refused {
    /// Two guests in the one room.
    Shared,
    /// It frees this many and they are asking for that many.
    TooFew,
    /// Endlessly many rooms, filled one coach at a time.
    OneAtATime,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Standing {
    /// A coach is at the door.
    Waiting,
    /// One was turned away. The night is over.
    Over,
}

pub struct Night {
    pub asking: Asking,
    pub housed: u32,
    pub standing: Standing,
    /// Why the last answer was refused, if it was.
    pub refused: Option<Refused>,
    /// How far along the shape we are. It wraps, and the wrap starts past the
    /// single guest, because one guest is the night opening and not a thing
    /// that happens again.
    at: usize,
}

impl Night {
    pub fn new(rng: &mut impl Rng) -> Self {
        let mut night = Self {
            asking: Asking::Guests(1),
            housed: 0,
            standing: Standing::Waiting,
            refused: None,
            at: 0,
        };
        night.asking = night.draw(rng);
        night
    }

    fn draw(&self, rng: &mut impl Rng) -> Asking {
        match SHAPE[self.at % SHAPE.len()] {
            Kind::One => Asking::Guests(1),
            Kind::Coachload => Asking::Guests(rng.gen_range(COACH_FEWEST..=COACH_MOST)),
            Kind::ACoach => Asking::ACoach,
            Kind::Coaches => Asking::Coaches,
        }
    }

    /// Answers with a rule off the dials.
    pub fn answer(&mut self, rule: Rule, rng: &mut impl Rng) -> Verdict {
        let verdict = rule.against(self.asking);
        self.settle(verdict, rng);
        verdict
    }

    /// Answers with the button, which houses anything.
    pub fn answer_with_primes(&mut self, rng: &mut impl Rng) -> Verdict {
        self.settle(Verdict::Taken, rng);
        Verdict::Taken
    }

    fn settle(&mut self, verdict: Verdict, rng: &mut impl Rng) {
        if self.standing == Standing::Over {
            return;
        }

        match verdict {
            Verdict::Taken => {
                self.housed += 1;
                self.refused = None;
                // the shape wraps past the single guest rather than back to
                // it, because one guest is the night opening and not a thing
                // that comes round
                self.at += 1;
                if self.at.is_multiple_of(SHAPE.len()) {
                    self.at += 1;
                }
                self.asking = self.draw(rng);
            }
            Verdict::Shared => {
                self.refused = Some(Refused::Shared);
                self.standing = Standing::Over;
            }
            Verdict::TooFew => {
                self.refused = Some(Refused::TooFew);
                self.standing = Standing::Over;
            }
            Verdict::OneAtATime => {
                self.refused = Some(Refused::OneAtATime);
                self.standing = Standing::Over;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn rng() -> StdRng {
        StdRng::seed_from_u64(9)
    }

    /// A rule that houses anything the dials can.
    const WIDE: Rule = Rule {
        stride: 2,
        shift: 0,
    };

    #[test]
    fn a_night_opens_on_one_guest() {
        let night = Night::new(&mut rng());

        assert_eq!(night.asking, Asking::Guests(1));
        assert_eq!(night.housed, 0);
        assert_eq!(night.standing, Standing::Waiting);
    }

    #[test]
    fn each_arrival_asks_for_more() {
        let mut rng = rng();
        let mut night = Night::new(&mut rng);
        let mut seen = Vec::new();

        for _ in 0..4 {
            seen.push(night.asking);
            night.answer(WIDE, &mut rng);
        }

        assert_eq!(seen[0], Asking::Guests(1));
        assert!(matches!(seen[1], Asking::Guests(many) if many >= COACH_FEWEST));
        assert!(matches!(seen[2], Asking::Guests(many) if many >= COACH_FEWEST));
        assert_eq!(seen[3], Asking::ACoach);
    }

    #[test]
    fn the_hardest_kind_returns() {
        let mut rng = rng();
        let mut night = Night::new(&mut rng);
        let mut hardest = 0;

        for _ in 0..40 {
            if night.asking == Asking::Coaches {
                hardest += 1;
                night.answer_with_primes(&mut rng);
            } else {
                night.answer(WIDE, &mut rng);
            }
            assert_eq!(night.standing, Standing::Waiting);
        }

        assert!(hardest >= 2, "it came round {} times in forty", hardest);
    }

    #[test]
    fn one_guest_is_the_opening_and_does_not_come_round() {
        let mut rng = rng();
        let mut night = Night::new(&mut rng);
        night.answer(WIDE, &mut rng);

        for _ in 0..40 {
            assert_ne!(night.asking, Asking::Guests(1), "the night started over");
            if night.asking == Asking::Coaches {
                night.answer_with_primes(&mut rng);
            } else {
                night.answer(WIDE, &mut rng);
            }
        }
    }

    #[test]
    fn too_few_rooms_is_refused() {
        let mut rng = rng();
        let mut night = Night::new(&mut rng);
        night.answer(WIDE, &mut rng);

        // a coachload, against a rule that frees one room
        let verdict = night.answer(Rule::new(1, 1), &mut rng);

        assert_eq!(verdict, Verdict::TooFew);
        assert_eq!(night.refused, Some(Refused::TooFew));
    }

    #[test]
    fn sharing_a_room_is_refused() {
        let mut rng = rng();
        let mut night = Night::new(&mut rng);

        let verdict = night.answer(Rule::new(0, 3), &mut rng);

        assert_eq!(verdict, Verdict::Shared);
        assert_eq!(night.refused, Some(Refused::Shared));
    }

    #[test]
    fn the_wall_is_refused_for_what_it_is() {
        let mut rng = rng();
        let mut night = Night::new(&mut rng);

        while night.asking != Asking::Coaches {
            night.answer(WIDE, &mut rng);
        }
        let verdict = night.answer(WIDE, &mut rng);

        assert_eq!(verdict, Verdict::OneAtATime);
        assert_eq!(night.refused, Some(Refused::OneAtATime));
    }

    #[test]
    fn a_coach_turned_away_ends_it() {
        let mut rng = rng();
        let mut night = Night::new(&mut rng);
        night.answer(Rule::new(1, 0), &mut rng);

        assert_eq!(night.standing, Standing::Over);

        // and nothing moves afterwards
        let housed = night.housed;
        night.answer(WIDE, &mut rng);
        assert_eq!(night.housed, housed);
        assert_eq!(night.standing, Standing::Over);
    }

    #[test]
    fn a_refusal_leaves_them_waiting() {
        let mut rng = rng();
        let mut night = Night::new(&mut rng);
        let asking = night.asking;

        night.answer(Rule::new(1, 0), &mut rng);

        assert_eq!(night.asking, asking, "the coach at the door changed");
    }

    #[test]
    fn housing_them_counts() {
        let mut rng = rng();
        let mut night = Night::new(&mut rng);

        for _ in 0..3 {
            if night.asking == Asking::Coaches {
                night.answer_with_primes(&mut rng);
            } else {
                night.answer(WIDE, &mut rng);
            }
        }

        assert_eq!(night.housed, 3);
    }

    #[test]
    fn the_button_houses_anything() {
        let mut rng = rng();
        let mut night = Night::new(&mut rng);

        for _ in 0..20 {
            assert_eq!(night.answer_with_primes(&mut rng), Verdict::Taken);
            assert_eq!(night.standing, Standing::Waiting);
        }
    }
}
