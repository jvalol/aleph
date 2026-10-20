//! aleph: a hotel with a room for every counting number, and all of them taken.
//! See `specs/`.

mod corridor;
mod desk;
mod night;
mod rule;

use blitzkit::camera::Camera;
use blitzkit::geometry::Geometry;
use blitzkit::keyboard::{KeyboardInput, KeyboardKey, KeyboardKeyState};
use blitzkit::lighting::PointLight;
use blitzkit::mesh::{MeshData, Transform};
use blitzkit::notice;
use blitzkit::renderer::render_text::{RenderText, TextRenderer};
use blitzkit::renderer::scene::{MeshId, Scene, TextureId};
use blitzkit::renderer::Renderer;
use blitzkit::sound::SoundSystem;
use blitzkit::{start, Game};
use glam::{vec2, vec3, vec4, Quat, Vec2, Vec3, Vec4};

use desk::Desk;

/// Whether this run is only here to be photographed, for `refresh-screenshots`
/// in the project above.
///
/// An opening frame is a corridor of shut doors, which says nothing. The
/// picture wants everyone doubled up and half the doors standing open.
fn staged() -> bool {
    std::env::args().any(|arg| arg == "--screenshot")
}

const FLOOR: Vec4 = vec4(0.13, 0.12, 0.14, 1.0);
const WALL: Vec4 = vec4(0.30, 0.27, 0.26, 1.0);
const CEILING: Vec4 = vec4(0.09, 0.09, 0.11, 1.0);
const DOOR: Vec4 = vec4(0.46, 0.30, 0.20, 1.0);
const ROOM_DARK: Vec4 = vec4(0.02, 0.02, 0.03, 1.0);
/// The plaque's ground. The number on it is white in the texture, so the colour
/// here is what the unlit part of the plaque comes out as.
const PLAQUE: Vec4 = vec4(0.85, 0.86, 0.92, 1.0);

const MATTE: f32 = 52.0;

/// How far past the last drawn room the corridor itself runs, so it leaves the
/// frame rather than ending in a wall.
const BEYOND: f32 = 14.0;

/// The lamps down the corridor, and how far apart they hang.
const LAMPS: usize = 5;
const LAMP_EVERY: f32 = corridor::APART * 3.0;
const LAMP: Vec3 = vec3(1.0, 0.92, 0.78);

const HUD_LEFT: f32 = 26.0;
const HUD_TOP: f32 = 22.0;
const HUD_APART: f32 = 22.0;
const HUD_SIZE: f32 = 14.0;

struct Aleph {
    desk: Desk,
    slab: Option<MeshId>,
    plaque: Option<MeshId>,
    /// One texture per drawn room, carrying its number.
    numbers: Vec<TextureId>,
    window: Vec2,
    held: [bool; 5],
    staged: bool,
    posed: bool,
    quitting: bool,
}

impl Aleph {
    fn new() -> Self {
        Self {
            desk: Desk::new(0x1E0),
            slab: None,
            plaque: None,
            numbers: Vec::new(),
            window: vec2(900.0, 600.0),
            held: [false; 5],
            staged: staged(),
            posed: false,
            quitting: false,
        }
    }

    /// Sets the dials for the camera and lets the doors finish swinging.
    fn pose(&mut self) {
        self.desk.turn_stride(1);

        // until the doors have stopped, rather than for a guessed while
        while !self.desk.settled() {
            self.desk.tick(corridor::SWING_SECONDS * 0.25);
        }
    }

    fn readout(&self) -> Vec<String> {
        vec![
            format!("{} housed", self.desk.night.housed),
            self.desk.asking_says(),
            self.desk.rule_says(),
            self.desk.standing_says(),
            String::from("up and down set the stride, left and right the shift, escape quits"),
        ]
    }
}

impl Default for Aleph {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for Aleph {
    fn load(&mut self, renderer: &mut Renderer) {
        self.slab = Some(renderer.add_mesh(&MeshData::cube()));
        self.plaque = Some(renderer.add_mesh(&corridor::plaque_mesh()));

        // a plaque per drawn room, its number drawn into a texture. White on
        // black is what the engine gives and what a plaque looks like.
        self.numbers = (1..=corridor::DRAWN)
            .map(|room| renderer.add_texture(&blitzkit::text::drawn(&room.to_string(), 48.0)))
            .collect();
    }

    fn initialize(
        &mut self,
        _geometry: &mut Geometry,
        _text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
        window_size: (f32, f32),
    ) {
        self.resized(window_size);
    }

    fn resized(&mut self, window_size: (f32, f32)) {
        self.window = vec2(window_size.0, window_size.1);
    }

    fn update(
        &mut self,
        dt: f32,
        geometry: &mut Geometry,
        text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
    ) {
        if self.staged && !self.posed {
            self.posed = true;
            self.pose();
        }

        self.desk.tick(dt);

        geometry.reset();
        text_renderer.reset();

        let lines: Vec<RenderText> = self
            .readout()
            .into_iter()
            .enumerate()
            .map(|(n, text)| RenderText {
                position: vec2(HUD_LEFT, HUD_TOP + n as f32 * HUD_APART),
                color: vec4(1.0, 1.0, 1.0, 1.0),
                size: HUD_SIZE,
                text,
                ..Default::default()
            })
            .collect();

        // the readout goes on a panel, so it reads over the corridor rather
        // than into it. See blitzkit's spec 0038.
        if let Some(frame) = notice::framing_all(&lines) {
            for quad in frame.iter() {
                geometry.push_quad(quad);
            }
        }
        for line in lines {
            text_renderer.push_render_text(line);
        }
    }

    fn process_keyboard(&mut self, input: KeyboardInput) {
        let down = input.state == KeyboardKeyState::Pressed;
        let slot = match input.key {
            KeyboardKey::Left => Some(0),
            KeyboardKey::Right => Some(1),
            KeyboardKey::Up => Some(2),
            KeyboardKey::Down => Some(3),
            KeyboardKey::Return => Some(4),
            _ => None,
        };

        if let Some(slot) = slot {
            // one press per press, so a held dial does not run to its stop
            let was = self.held[slot];
            self.held[slot] = down;
            if !down || was {
                return;
            }
        } else if !down {
            return;
        }

        if self.desk.is_over() {
            match input.key {
                KeyboardKey::Return => self.desk.again(),
                KeyboardKey::Escape => self.quitting = true,
                _ => (),
            }
            return;
        }

        match input.key {
            KeyboardKey::Up => self.desk.turn_stride(1),
            KeyboardKey::Down => self.desk.turn_stride(-1),
            KeyboardKey::Right => self.desk.turn_shift(1),
            KeyboardKey::Left => self.desk.turn_shift(-1),
            KeyboardKey::Return => {
                self.desk.answer();
            }
            KeyboardKey::P => {
                self.desk.answer_with_primes();
            }
            KeyboardKey::Escape => self.quitting = true,
            _ => (),
        }
    }

    fn draw(&mut self, scene: &mut Scene, camera: &mut Camera) {
        let (slab, plaque) = match (self.slab, self.plaque) {
            (Some(slab), Some(plaque)) => (slab, plaque),
            _ => return,
        };

        camera.position = corridor::eye();
        camera.target = corridor::looked_at();

        scene.light.direction = vec3(0.0, -1.0, -0.2).normalize();
        scene.light.intensity = 0.3;
        scene.light.ambient = Vec3::splat(0.1);

        // past the desk at one end and past the last drawn room at the other,
        // so the corridor leaves the frame rather than ending in a wall
        let near = corridor::APART * 4.0;
        let far = -(corridor::APART * (corridor::DRAWN / 2) as f32 + BEYOND);
        let deep = near - far;
        let middle = (near + far) * 0.5;

        scene.push_material(
            slab,
            &Transform::at(vec3(0.0, -0.05, middle)).with_scale(vec3(corridor::ACROSS, 0.1, deep)),
            FLOOR,
            MATTE,
        );
        scene.push_material(
            slab,
            &Transform::at(vec3(0.0, corridor::TALL, middle)).with_scale(vec3(
                corridor::ACROSS,
                0.1,
                deep,
            )),
            CEILING,
            MATTE,
        );

        for side in [-1.0f32, 1.0] {
            scene.push_material(
                slab,
                &Transform::at(vec3(
                    side * (corridor::ACROSS * 0.5 + 0.1),
                    corridor::TALL * 0.5,
                    middle,
                ))
                .with_scale(vec3(0.2, corridor::TALL, deep)),
                WALL,
                MATTE,
            );
        }

        // lamps down the middle, so the corridor falls away rather than ending
        for lamp in 0..LAMPS {
            scene.push_light(PointLight::new(
                vec3(0.0, corridor::TALL * 0.92, -(lamp as f32) * LAMP_EVERY),
                LAMP,
                1.5,
                LAMP_EVERY * 1.6,
            ));
        }

        for room in 1..=corridor::DRAWN {
            let open = self.desk.open_by(room);
            let doorway = corridor::doorway(room);
            let side = if corridor::is_left(room) { -1.0 } else { 1.0 };

            // the dark of the room behind, which is what an open door shows
            scene.push_material(
                slab,
                &Transform::at(doorway + Vec3::X * side * 0.12).with_scale(vec3(
                    0.18,
                    corridor::DOOR_TALL,
                    corridor::DOOR_WIDE,
                )),
                ROOM_DARK,
                MATTE,
            );

            let (at, turn) = corridor::door(room, open);
            scene.push_material(
                slab,
                &Transform::at(at).with_rotation(turn).with_scale(vec3(
                    corridor::DOOR_THICK,
                    corridor::DOOR_TALL,
                    corridor::DOOR_WIDE,
                )),
                DOOR,
                MATTE,
            );

            // the number beside it. A left plaque faces along positive x, which
            // is how the mesh is built; a right one is turned to face back.
            if let Some(number) = self.numbers.get(room as usize - 1) {
                let facing = if corridor::is_left(room) {
                    Quat::IDENTITY
                } else {
                    Quat::from_rotation_y(std::f32::consts::PI)
                };

                scene.push_textured(
                    plaque,
                    *number,
                    &Transform::at(corridor::plaque(room)).with_rotation(facing),
                    PLAQUE,
                    MATTE,
                );
            }
        }
    }

    fn focus_changed(&mut self, _focus: bool) {}

    fn is_quitting(&self) -> bool {
        self.quitting
    }
}

fn main() {
    start("aleph", Box::new(Aleph::new()));
}
