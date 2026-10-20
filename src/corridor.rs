//! Where the rooms are and which of them are taken. See `specs/0001-the-night.md`.
//!
//! Odd rooms down the left and even rooms down the right, which is not
//! decoration: doubling everyone sends every guest into an even room, so the
//! whole left wall opens in one move. It is the clearest picture this game has
//! of freeing endlessly many rooms, and it exists only because the sides are
//! split that way.
//!
//! All arithmetic on a corridor, so it is checked without a window.

use glam::{vec3, Vec3};

use crate::rule::Rule;

/// How many rooms are drawn.
///
/// Forty, because perspective divides them rather than a fixed width doing it:
/// the near doors are large, the far ones shrink, and the corridor reads as
/// having no end. Nothing past the fortieth is drawn and nothing needs to be.
pub const DRAWN: u64 = 40;

/// How far apart two rooms on the same side are, down the corridor.
pub const APART: f32 = 1.5;

/// How wide the corridor is, and how tall.
pub const ACROSS: f32 = 3.2;
pub const TALL: f32 = 2.4;

/// A door's size.
pub const DOOR_WIDE: f32 = 1.05;
pub const DOOR_TALL: f32 = 1.9;
pub const DOOR_THICK: f32 = 0.07;

/// How far open a door swings, and how long it takes.
pub const SWINGS_TO: f32 = 1.75;
pub const SWING_SECONDS: f32 = 0.4;

/// The plaque beside a door: how big it is and how far from the frame.
///
/// Beside the door and not on it. The engine draws text light on dark, which is
/// what a plaque looks like and is not what a number painted on a pale door
/// looks like.
pub const PLAQUE_WIDE: f32 = 0.3;
pub const PLAQUE_TALL: f32 = 0.22;
pub const PLAQUE_BESIDE: f32 = 0.72;
pub const PLAQUE_UP: f32 = 1.35;

/// Which side of the corridor a room is on. Odd rooms are on the left.
pub fn is_left(room: u64) -> bool {
    room % 2 == 1
}

/// How far down the corridor a room is. Rooms 1 and 2 face each other.
fn along(room: u64) -> f32 {
    ((room - 1) / 2) as f32 * APART
}

/// The middle of a room's doorway, on its own wall.
pub fn doorway(room: u64) -> Vec3 {
    let side = if is_left(room) { -1.0 } else { 1.0 };

    vec3(side * ACROSS * 0.5, DOOR_TALL * 0.5, -along(room))
}

/// The edge a door hangs from, which is its far edge, so every door swings back
/// down the corridor and away from the desk.
pub fn hinge(room: u64) -> Vec3 {
    doorway(room) - Vec3::Z * DOOR_WIDE * 0.5
}

/// Where a door's middle is when it is `open` of the way open, from 0 to 1.
pub fn door(room: u64, open: f32) -> (Vec3, glam::Quat) {
    // a left door swings one way and a right door the other, so both open into
    // the corridor rather than into the wall behind them
    let way = if is_left(room) { 1.0 } else { -1.0 };
    let turn = glam::Quat::from_rotation_y(way * SWINGS_TO * open.clamp(0.0, 1.0));
    let reach = Vec3::Z * DOOR_WIDE * 0.5;

    (hinge(room) + turn * reach, turn)
}

/// The plaque beside a room's door, and which way it faces.
pub fn plaque(room: u64) -> Vec3 {
    let side = if is_left(room) { -1.0 } else { 1.0 };

    vec3(
        side * (ACROSS * 0.5 - 0.02),
        PLAQUE_UP,
        -along(room) + PLAQUE_BESIDE,
    )
}

/// The quad a plaque's number is drawn on, facing along positive x.
///
/// Its own mesh rather than a cube: a cube carries its texture whichever way
/// round each face pleases, which the arcade found out the hard way, and a
/// number is the one thing that cannot be turned on its side.
pub fn plaque_mesh() -> blitzkit::mesh::MeshData {
    use blitzkit::mesh::Vertex;

    let (half_z, half_y) = (PLAQUE_WIDE * 0.5, PLAQUE_TALL * 0.5);
    let out = [1.0, 0.0, 0.0];

    blitzkit::mesh::MeshData::new(
        vec![
            Vertex::new([0.0, half_y, half_z], out, [0.0, 0.0]),
            Vertex::new([0.0, half_y, -half_z], out, [1.0, 0.0]),
            Vertex::new([0.0, -half_y, -half_z], out, [1.0, 1.0]),
            Vertex::new([0.0, -half_y, half_z], out, [0.0, 1.0]),
        ],
        vec![0, 2, 1, 0, 3, 2],
    )
}

/// Where the desk stands and what it looks at.
pub fn eye() -> Vec3 {
    // far enough back that rooms one and two are both in frame, because the
    // shift dial frees the lowest rooms and they are the ones to watch
    vec3(0.0, TALL * 0.62, APART * 2.9)
}

pub fn looked_at() -> Vec3 {
    vec3(0.0, TALL * 0.45, -APART * 10.0)
}

/// Where the prime power rule puts the residents who are drawn.
///
/// Room n becomes room 2^n, so the drawn corridor ends up with guests in rooms
/// 2, 4, 8, 16 and 32 and nobody anywhere else. It is the most room anything in
/// this game frees and it looks like it.
pub fn primed() -> Vec<(u64, Option<u64>)> {
    (1..=DRAWN)
        .map(|room| {
            let landed = crate::rule::primes::resident(room as u32).filter(|to| *to <= DRAWN);
            (room, landed)
        })
        .collect()
}

/// Where the prime power rule puts the arrivals who land in the drawn corridor.
///
/// Seat s of coach c goes to room p^s, p being the c-th odd prime. Every coach
/// gets rooms of its own rather than queueing behind the one before it, and
/// this is the half of the trick that is worth seeing: the corridor does not
/// empty, it refills with somebody else.
pub fn arrived() -> Vec<u64> {
    let mut rooms = Vec::new();

    for coach in 1..=DRAWN as u32 {
        let Some(prime) = crate::rule::primes::odd(coach) else {
            break;
        };
        if prime > DRAWN {
            break;
        }

        for seat in 1..=DRAWN as u32 {
            match crate::rule::primes::guest(coach, seat) {
                Some(room) if room <= DRAWN => rooms.push(room),
                _ => break,
            }
        }
    }

    rooms.sort_unstable();
    rooms
}

/// Which of the drawn rooms the prime power rule leaves empty, once the
/// residents have moved and the coaches have come in.
pub fn free_after_primes() -> Vec<u64> {
    let mut taken: Vec<u64> = primed().into_iter().filter_map(|(_, to)| to).collect();
    taken.extend(arrived());

    (1..=DRAWN).filter(|room| !taken.contains(room)).collect()
}

/// Which of the drawn rooms a rule leaves empty.
pub fn free_rooms(rule: &Rule) -> Vec<u64> {
    let taken: Vec<u64> = moves(rule).into_iter().filter_map(|(_, to)| to).collect();

    (1..=DRAWN).filter(|room| !taken.contains(room)).collect()
}

/// Where the guest in each drawn room ends up. A guest whose new room is past
/// the drawn ones has walked off down the corridor and is not coming back.
pub fn moves(rule: &Rule) -> Vec<(u64, Option<u64>)> {
    (1..=DRAWN)
        .map(|room| {
            let landed = rule.lands(room).filter(|landed| *landed <= DRAWN);
            (room, landed)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_left_side_is_the_odd_rooms() {
        for room in 1..=DRAWN {
            assert_eq!(is_left(room), room % 2 == 1, "room {}", room);
            assert_eq!(doorway(room).x < 0.0, is_left(room), "room {}", room);
        }

        // which is what makes doubling empty a whole wall
        let doubling = Rule::new(2, 0);
        assert!(free_rooms(&doubling).iter().all(|room| is_left(*room)));
    }

    #[test]
    fn two_rooms_face_each_other() {
        for pair in 0..DRAWN / 2 {
            let left = pair * 2 + 1;
            let right = pair * 2 + 2;

            assert!((doorway(left).z - doorway(right).z).abs() < 1e-5);
            assert!(doorway(left).x < 0.0 && doorway(right).x > 0.0);
        }
    }

    #[test]
    fn the_corridor_runs_away_from_the_desk() {
        let eye = eye();

        for room in 1..=DRAWN {
            assert!(doorway(room).z < eye.z, "room {} is behind you", room);
        }
        assert!(doorway(DRAWN).z < doorway(1).z, "it does not recede");
    }

    #[test]
    fn a_shut_door_fills_its_doorway() {
        let (middle, turn) = door(3, 0.0);

        assert!((middle - doorway(3)).length() < 1e-5);
        assert!(turn.to_axis_angle().1.abs() < 1e-5);
    }

    #[test]
    fn an_open_door_swings_back_down_the_corridor() {
        for room in [1u64, 2, 7, 8] {
            let shut = door(room, 0.0).0;
            let open = door(room, 1.0).0;

            assert!(open.z < shut.z, "room {} swung towards the desk", room);
            assert!(
                open.x.abs() < shut.x.abs(),
                "room {} swung into the wall",
                room
            );
        }
    }

    #[test]
    fn a_door_hangs_off_its_hinge_the_whole_way() {
        let hinge = hinge(5);

        for step in 0..=10 {
            let open = step as f32 / 10.0;
            let reach = door(5, open).0 - hinge;

            assert!(
                (reach.length() - DOOR_WIDE * 0.5).abs() < 1e-4,
                "it came off its hinge at {}",
                open
            );
        }
    }

    #[test]
    fn a_plaque_sits_beside_its_door() {
        for room in 1..=DRAWN {
            let plaque = plaque(room);
            let doorway = doorway(room);

            assert_eq!(plaque.x < 0.0, is_left(room), "room {} crossed over", room);
            assert!(
                (plaque.x.abs() - ACROSS * 0.5).abs() < 0.05,
                "room {} floated off its wall",
                room
            );
            assert!(plaque.z > doorway.z, "room {} is behind its door", room);
            assert!(plaque.y > 0.0 && plaque.y < TALL);
        }
    }

    #[test]
    fn a_plaque_clears_the_door_beside_it() {
        // the plaque sits towards the desk from its doorway, and the door
        // swings the other way, so one can never stand in front of the other
        for room in [1u64, 2, 9, 10] {
            let plaque = plaque(room);
            for step in 0..=10 {
                let door = door(room, step as f32 / 10.0).0;
                assert!(plaque.z > door.z, "room {} was covered at {}", room, step);
            }
        }
    }

    #[test]
    fn every_room_starts_taken() {
        // nobody has moved, so nothing is free
        assert!(free_rooms(&Rule::new(1, 0)).is_empty());
    }

    #[test]
    fn a_rule_moves_each_guest_where_it_says() {
        let rule = Rule::new(2, 1);

        for (room, landed) in moves(&rule) {
            match landed {
                Some(landed) => assert_eq!(landed, room * 2 + 1),
                None => assert!(room * 2 + 1 > DRAWN, "room {} vanished early", room),
            }
        }
    }

    #[test]
    fn a_guest_sent_past_the_end_leaves() {
        let doubling = Rule::new(2, 0);
        let gone: Vec<u64> = moves(&doubling)
            .into_iter()
            .filter(|(_, landed)| landed.is_none())
            .map(|(room, _)| room)
            .collect();

        assert!(!gone.is_empty(), "nobody walked off");
        assert!(gone.iter().all(|room| room * 2 > DRAWN));
    }

    #[test]
    fn the_free_rooms_are_the_ones_nobody_took() {
        let rule = Rule::new(3, 2);
        let landed: Vec<u64> = moves(&rule).into_iter().filter_map(|(_, to)| to).collect();

        for room in free_rooms(&rule) {
            assert!(!landed.contains(&room), "room {} was taken", room);
        }
        for room in landed {
            assert!(!free_rooms(&rule).contains(&room), "room {} was free", room);
        }
    }

    #[test]
    fn a_plaque_is_one_quad_facing_into_the_corridor() {
        let mesh = plaque_mesh();

        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.triangle_count(), 2);
        assert!(mesh.vertices.iter().all(|at| at.position[0] == 0.0));
        assert!(mesh.vertices.iter().all(|at| at.normal == [1.0, 0.0, 0.0]));
    }

    #[test]
    fn a_plaque_reads_the_right_way_up() {
        let mesh = plaque_mesh();
        let top: Vec<&blitzkit::mesh::Vertex> = mesh
            .vertices
            .iter()
            .filter(|at| at.position[1] > 0.0)
            .collect();

        // the two upper corners carry the upper edge of the picture, which is
        // v nothing, because the engine draws text from the top down
        assert_eq!(top.len(), 2);
        assert!(top.iter().all(|at| at.uv[1] == 0.0));
    }

    #[test]
    fn prime_powers_move_the_residents_to_the_powers_of_two() {
        let taken: Vec<u64> = primed().into_iter().filter_map(|(_, to)| to).collect();

        assert_eq!(taken, vec![2, 4, 8, 16, 32], "{:?}", taken);
    }

    #[test]
    fn every_coach_gets_rooms_of_its_own() {
        let arrived = arrived();

        // the first coach takes the powers of three, the second the powers of
        // five, and neither waits for the other to finish
        for room in [3u64, 9, 27] {
            assert!(arrived.contains(&room), "the first coach missed {}", room);
        }
        for room in [5u64, 25] {
            assert!(arrived.contains(&room), "the second coach missed {}", room);
        }
        assert!(arrived.contains(&7), "the third coach got nothing");
    }

    #[test]
    fn nobody_shares_after_the_prime_powers() {
        let mut taken: Vec<u64> = primed().into_iter().filter_map(|(_, to)| to).collect();
        taken.extend(arrived());

        let before = taken.len();
        taken.sort_unstable();
        taken.dedup();

        assert_eq!(taken.len(), before, "a room was handed out twice");
    }

    #[test]
    fn the_corridor_refills_rather_than_emptying() {
        let free = free_after_primes();

        // room 1 is nobody's: the residents start at two to the one
        assert!(free.contains(&1));
        assert!(!free.contains(&2) && !free.contains(&3) && !free.contains(&5));

        // and what is left is the rooms that are no prime's power at all
        assert!(free.contains(&6) && free.contains(&10) && free.contains(&12));

        // nineteen of the first forty rooms are a prime's power, so the
        // corridor comes out about half full rather than empty
        assert_eq!(free.len(), 21, "{:?}", free);
    }
}
