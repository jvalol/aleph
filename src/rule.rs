//! The one rule that moves everybody, and what it frees. See
//! `specs/0001-the-night.md`.
//!
//! All of it is arithmetic on the counting numbers, so it is checked without a
//! window. What a rule frees is proved here rather than enacted: the corridor
//! shows the part of it you can see, and the part you cannot is this.

/// Room n becomes room `stride` times n plus `shift`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Rule {
    pub stride: u64,
    pub shift: u64,
}

/// How far the dials go.
///
/// The stride reaches nothing on purpose: a stride of nothing puts every guest
/// in one room, and a dial that cannot do the wrong thing teaches nothing.
pub const STRIDE_MOST: u64 = 6;
pub const SHIFT_MOST: u64 = 24;

/// How many rooms a rule leaves empty.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Freed {
    None,
    /// This many, and no more.
    Counted(u64),
    /// Without end.
    Endless,
}

/// What a coach at the door is asking for.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Asking {
    /// A number of guests you could write down.
    Guests(u64),
    /// One coach with a seat for every counting number.
    ACoach,
    /// A coach for every counting number, each with a seat for every counting
    /// number.
    Coaches,
}

/// What happens when a rule meets an arrival.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Verdict {
    Taken,
    /// Two guests would end up in the one room.
    Shared,
    /// It frees rooms, and not enough of them.
    TooFew,
    /// It frees rooms without end, and they are filled one coach at a time, so
    /// the second coach is never reached.
    ///
    /// This is the wall the dials have. It is not that there are too few rooms:
    /// endlessly many free rooms is enough for any countable crowd. It is that
    /// filling them in order means starting with the first coach and never
    /// finishing it.
    OneAtATime,
}

impl Rule {
    pub fn new(stride: u64, shift: u64) -> Self {
        Self { stride, shift }
    }

    /// Where the guest in `room` ends up, or nothing when the number is too
    /// large to count. Such a guest is further down the corridor than anyone
    /// can see, which is true of this hotel regardless.
    pub fn lands(&self, room: u64) -> Option<u64> {
        self.stride.checked_mul(room)?.checked_add(self.shift)
    }

    /// Whether it sends every guest somewhere of their own.
    pub fn keeps_them_apart(&self) -> bool {
        self.stride >= 1
    }

    /// How many rooms it leaves empty.
    ///
    /// A stride of one slides everybody along, so the rooms below the shift are
    /// the only ones freed. A stride of two or more spreads them out, and what
    /// is left is endless: everything below the first landing, and every room
    /// that is not the shift more than a multiple of the stride.
    pub fn frees(&self) -> Freed {
        match self.stride {
            0 => Freed::None,
            1 if self.shift == 0 => Freed::None,
            1 => Freed::Counted(self.shift),
            _ => Freed::Endless,
        }
    }

    /// Whether this rule houses what is at the door, and what is wrong with it
    /// when it does not.
    pub fn against(&self, asking: Asking) -> Verdict {
        if !self.keeps_them_apart() {
            return Verdict::Shared;
        }

        match (self.frees(), asking) {
            (Freed::None, _) => Verdict::TooFew,
            (Freed::Counted(rooms), Asking::Guests(many)) => {
                if rooms >= many {
                    Verdict::Taken
                } else {
                    Verdict::TooFew
                }
            }
            (Freed::Counted(_), _) => Verdict::TooFew,
            (Freed::Endless, Asking::Coaches) => Verdict::OneAtATime,
            (Freed::Endless, _) => Verdict::Taken,
        }
    }
}

/// The rule that is not on the dials: everyone to a prime power.
///
/// A resident in room n goes to room 2^n. Seat s of coach c goes to room p^s,
/// where p is the c-th odd prime. No two of those are the same room, because a
/// number factors one way only, and every coach gets endlessly many rooms of
/// its own rather than waiting behind the one before it.
pub mod primes {
    /// Where a resident goes.
    pub fn resident(room: u32) -> Option<u64> {
        2u64.checked_pow(room)
    }

    /// Where seat `seat` of coach `coach` goes. Both are counted from one.
    pub fn guest(coach: u32, seat: u32) -> Option<u64> {
        odd(coach)?.checked_pow(seat)
    }

    /// The `which`th odd prime, counted from one: 3, 5, 7, 11, and so on.
    pub fn odd(which: u32) -> Option<u64> {
        if which == 0 {
            return None;
        }

        let mut found = 0;
        let mut at: u64 = 1;

        while found < which {
            at += 2;
            if is_prime(at) {
                found += 1;
            }
        }

        Some(at)
    }

    fn is_prime(of: u64) -> bool {
        if of < 2 {
            return false;
        }

        let mut by = 2;
        while by * by <= of {
            if of.is_multiple_of(by) {
                return false;
            }
            by += 1;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubling_frees_the_odd_rooms() {
        let doubling = Rule::new(2, 0);

        for room in 1..=200u64 {
            let landed = doubling.lands(room).expect("it fits");
            assert_eq!(landed % 2, 0, "room {} did not land on an even one", room);
        }

        // which leaves the odd ones empty, half of any stretch of corridor
        let landed: Vec<u64> = (1..=40).filter_map(|room| doubling.lands(room)).collect();
        let free: Vec<u64> = (1..=40u64).filter(|room| !landed.contains(room)).collect();

        assert!(free.iter().all(|room| room % 2 == 1), "{:?}", free);
        assert_eq!(free.len(), 20, "half of forty rooms should come free");
    }

    #[test]
    fn a_shift_frees_what_it_shifts_by() {
        for shift in 1..=SHIFT_MOST {
            let sliding = Rule::new(1, shift);

            assert_eq!(sliding.frees(), Freed::Counted(shift));

            let upto = SHIFT_MOST * 2;
            let landed: Vec<u64> = (1..=upto).filter_map(|room| sliding.lands(room)).collect();
            let free = (1..=upto).filter(|room| !landed.contains(room)).count();
            assert_eq!(free as u64, shift);
        }
    }

    #[test]
    fn standing_still_frees_nothing() {
        let still = Rule::new(1, 0);

        assert_eq!(still.frees(), Freed::None);
        assert!((1..=40).all(|room| still.lands(room) == Some(room)));
    }

    #[test]
    fn a_stride_past_one_frees_without_end() {
        for stride in 2..=STRIDE_MOST {
            for shift in 0..=4 {
                assert_eq!(Rule::new(stride, shift).frees(), Freed::Endless);
            }
        }
    }

    #[test]
    fn no_stride_is_a_collision() {
        let heap = Rule::new(0, 7);

        assert!(!heap.keeps_them_apart());
        assert_eq!(heap.against(Asking::Guests(1)), Verdict::Shared);

        // every guest in the one room, which is what the dial reaching nothing
        // is there to let you find out
        for room in 1..=50 {
            assert_eq!(heap.lands(room), Some(7));
        }
    }

    #[test]
    fn every_stride_past_nothing_keeps_them_apart() {
        for stride in 1..=STRIDE_MOST {
            for shift in 0..=SHIFT_MOST {
                let rule = Rule::new(stride, shift);
                assert!(rule.keeps_them_apart());

                let mut landed: Vec<u64> = (1..=300).filter_map(|room| rule.lands(room)).collect();
                let before = landed.len();
                landed.sort_unstable();
                landed.dedup();

                assert_eq!(landed.len(), before, "{:?} put two in one room", rule);
            }
        }
    }

    #[test]
    fn a_room_too_far_is_past_the_end() {
        let doubling = Rule::new(2, 0);

        assert_eq!(doubling.lands(u64::MAX), None);
        assert!(doubling.lands(1_000_000).is_some());
    }

    #[test]
    fn the_dials_have_a_wall() {
        // every setting of them, against a coach for every counting number
        for stride in 0..=STRIDE_MOST {
            for shift in 0..=SHIFT_MOST {
                let verdict = Rule::new(stride, shift).against(Asking::Coaches);
                assert_ne!(
                    verdict,
                    Verdict::Taken,
                    "{} and {} housed them",
                    stride,
                    shift
                );
            }
        }

        // and the wall is not that there are too few rooms. Endlessly many is
        // enough for any countable crowd; the trouble is filling them in order
        assert_eq!(
            Rule::new(2, 0).against(Asking::Coaches),
            Verdict::OneAtATime
        );
    }

    #[test]
    fn endless_room_houses_one_endless_coach() {
        assert_eq!(Rule::new(2, 0).against(Asking::ACoach), Verdict::Taken);
        assert_eq!(Rule::new(1, 9).against(Asking::ACoach), Verdict::TooFew);
    }

    #[test]
    fn a_counted_rule_houses_what_it_counts() {
        let sliding = Rule::new(1, 10);

        assert_eq!(sliding.against(Asking::Guests(10)), Verdict::Taken);
        assert_eq!(sliding.against(Asking::Guests(11)), Verdict::TooFew);
    }

    #[test]
    fn prime_powers_house_every_coach() {
        // every coach gets rooms of its own rather than waiting behind the one
        // before it, which is the whole of what the dials could not do
        for coach in 1..=6u32 {
            for seat in 1..=8u32 {
                assert!(primes::guest(coach, seat).is_some(), "{} {}", coach, seat);
            }
        }
    }

    #[test]
    fn no_two_prime_powers_are_the_same_room() {
        let mut rooms: Vec<u64> = Vec::new();

        for room in 1..=20u32 {
            rooms.push(primes::resident(room).expect("it fits"));
        }
        for coach in 1..=6u32 {
            for seat in 1..=8u32 {
                rooms.push(primes::guest(coach, seat).expect("it fits"));
            }
        }

        let before = rooms.len();
        rooms.sort_unstable();
        rooms.dedup();

        assert_eq!(rooms.len(), before, "a number factored two ways");
    }

    #[test]
    fn the_odd_primes_come_in_order() {
        let want = [3u64, 5, 7, 11, 13, 17, 19, 23];

        for (n, prime) in want.iter().enumerate() {
            assert_eq!(primes::odd(n as u32 + 1), Some(*prime));
        }
        assert_eq!(primes::odd(0), None);
    }

    #[test]
    fn a_prime_power_too_far_is_past_the_end() {
        assert_eq!(primes::resident(64), None);
        assert!(primes::resident(63).is_some());
    }
}
