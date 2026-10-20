# 0001 The night

**Status:** implemented
**Date:** 2026-10-04

## Parked

Built on 2026-10-04 and put down the same night. The corridor works and the
game does not, for three reasons worth keeping rather than rediscovering.

**There are no guests.** A shut door is the only evidence anyone is in there.
This paradox is about everybody having to move and all of them still fitting,
and nobody is drawn.

**Nothing moves.** Applying a rule swings doors. The guests do not travel from
room n to room a·n + b, so the transformation is invisible and only its result
shows, which is the lesser half of it.

**The dials are a menu with extra steps.** They were meant to be a space to
explore instead of a list to pick from, and they are not. There are three right
answers, a stride of one with a shift past the coachload, a stride of two or
more for one endless coach, and the button for the rest. No setting makes a
stride of four better than a stride of two, so a player works the three cases
out in a couple of minutes and the dials stop mattering.

Two things would fix it and neither is small. Put guests in the rooms and walk
them, which is what the corridor was built for and is currently unused. And
give a rule a cost, so that disturbing fewest guests is worth something and a
stride of one with a shift of twelve beats a stride of two for a coach of
twelve. The second changes this spec.

Everything below is what was built. The crate's manifest is renamed to
`Cargo.toml.not-yet`, which takes it out of `list-repos` and so out of the gate,
the arcade and the listings. Rename it back to pick this up.

## Goal

You are on the desk at a hotel with a room for every counting number, and every
one of them is taken. Coaches keep arriving and each wants more than the last.
You make room by moving the guests who are already in, and you move them all at
once with one rule. The game is finding a rule that works.

## Behavior

### The corridor

Rooms are numbered 1, 2, 3, and on without end. Odd rooms are down the left side
and even rooms down the right, so the two sides alternate as you look along it.

That is not decoration. The rule that doubles everyone sends every guest into an
even room, which empties the entire left wall in one move. It is the clearest
picture this game has of freeing infinitely many rooms, and it only exists
because the sides are split odd and even.

The corridor runs away to a vanishing point. Perspective does the work a row of
doors cannot: near doors are large, far doors shrink, and forty of them read
where twelve in a row is already a fence. Nothing is drawn past the fortieth,
and nothing needs to be: the point of the hotel is that there is always more of
it than you can see.

A taken room is shut. A free one stands open and dark. Every door carries a
plaque with its number, white on black beside the frame rather than painted on
the door, because the engine draws text light on dark and a plaque is what that
looks like anyway.

### The rule

One rule moves everybody: **room n becomes room a·n + b**. You set `a` and `b`
on two dials and the corridor rearranges.

- `a = 1, b = 1` is everyone up one, which frees room 1.
- `a = 1, b = k` frees rooms 1 to k.
- `a = 2, b = 0` is everyone to double their room, which frees every odd room.
- `a = 3, b = 0` frees two rooms in every three.
- `a = 0` puts every guest in room b at once. The dial reaches it on purpose.

What a rule frees:

- `a = 0` frees nothing and loses the hotel. Every guest is in one room.
- `a = 1` frees exactly `b` rooms, and none at all when `b` is zero.
- `a` of two or more frees infinitely many: everything below `a + b`, and every
  room that is not `b` more than a multiple of `a`.

A rule is taken when it sends no two guests to the same room and frees at least
as many rooms as the arrival asks for. Otherwise it is refused and the coach
stays at the door. The refusal says which of the two went wrong, in the rule's
own terms: how many it frees against how many are asking.

### The one the dials cannot reach

A coach for every counting number, each with a seat for every counting number,
is refused by every setting of the dials.

The reason is not that there are too few rooms. A stride of two or more frees
endlessly many, and endlessly many is enough for any crowd you can count. The
trouble is the order they are filled in. The free rooms are handed out lowest
first, so you start on the first coach and never finish it, and the second coach
is never reached at all.

For that there is one button: **everyone to a prime power**. Room n becomes room
2^n, and seat s of coach c goes to room p^s where p is the c-th odd prime. No
two of those are the same room, because a number factors one way only, and every
coach gets endlessly many rooms of its own rather than queueing behind the one
before it.

It is the only thing in the game that is not on the dials, and finding out that
the dials have a wall, and what kind of wall it is, is the point of having
them.

### The night

Arrivals keep coming and keep asking for more. A single guest, then coaches of a
dozen and of fifty, then a coach with a seat for every counting number, then a
coach for every counting number. The hardest kind comes round again rather than
ending the night.

You lose when a coach is turned away, which is a refusal you do not recover
from, or when a rule puts two guests in one room. The number at the end is how
many you housed.

### What is simulated and what is not

A rule is a function on the counting numbers and what it frees is worked out as
arithmetic. Nothing beyond the forty drawn rooms moves, and nothing needs to:
the proof that a rule works is the arithmetic, and the corridor is the part of
it you can watch.

Room numbers are computed with checked arithmetic. A guest whose new room cannot
be counted is further down the corridor than anyone can see, which is true of
this hotel regardless.

## Acceptance criteria

- Doubling frees every odd room. — `rule::tests::doubling_frees_the_odd_rooms`
- And the odd rooms are the whole left side. — `corridor::tests::the_left_side_is_the_odd_rooms`
- A stride of one frees exactly as many as the shift. — `rule::tests::a_shift_frees_what_it_shifts_by`
- A stride of one and no shift frees nothing. — `rule::tests::standing_still_frees_nothing`
- A stride past one frees without end. — `rule::tests::a_stride_past_one_frees_without_end`
- A stride of nothing puts everyone in one room. — `rule::tests::no_stride_is_a_collision`
- Every other stride keeps them apart. — `rule::tests::every_stride_past_nothing_keeps_them_apart`
- A room too large to count is past the end rather than wrong. — `rule::tests::a_room_too_far_is_past_the_end`
- No setting of the dials houses a coach for every counting number, and the
  reason is the order rooms are filled in rather than how many there are. —
  `rule::tests::the_dials_have_a_wall`
- Endlessly many free rooms do house one endless coach. — `rule::tests::endless_room_houses_one_endless_coach`
- Prime powers do. — `rule::tests::prime_powers_house_every_coach`
- And give no two guests the same room. — `rule::tests::no_two_prime_powers_are_the_same_room`
- A rule that frees too few is refused, and says so. — `night::tests::too_few_rooms_is_refused`
- A rule that doubles guests up is refused, and says so. — `night::tests::sharing_a_room_is_refused`
- A refusal leaves the coach at the door. — `night::tests::a_refusal_leaves_them_waiting`
- Turning one away ends the night. — `night::tests::a_coach_turned_away_ends_it`
- The arrivals grow. — `night::tests::each_arrival_asks_for_more`
- And the hardest kind comes round again. — `night::tests::the_hardest_kind_returns`
- Every room is taken when the night starts. — `corridor::tests::every_room_starts_taken`
- A rule moves each drawn guest where it says. — `corridor::tests::a_rule_moves_each_guest_where_it_says`
- A guest sent past the end leaves the view. — `corridor::tests::a_guest_sent_past_the_end_leaves`
- The rooms that come free are the ones nobody landed in. — `corridor::tests::the_free_rooms_are_the_ones_nobody_took`
- A plaque sits beside its own door at every depth. — `corridor::tests::a_plaque_sits_beside_its_door`

### Verified by hand

- The corridor reads as having no end.
- Doubling is the one to watch: the guests spread out down the corridor and the
  whole left wall opens behind them.
- Turning a dial says what it would free before you commit to it.

## Out of scope

Walking the corridor. You are on the desk.

A rule you write yourself. Two dials and one button is the whole vocabulary, and
a rule you can type is a different game.

Rooms with anything in them. A door, a number and whether it is taken is the
whole of a room here.

Any arrival that is not a count of guests. No awkward guests, no bookings, no
checkouts.
