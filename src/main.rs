use raylib::{ffi::GetFrameTime, prelude::*};

pub struct Engine {
    rl: RaylibHandle,
    thread: RaylibThread,
}

trait UpdateSystem {
    fn update_system(world: &mut World, dt: f32);
}

trait DrawSystem {
    fn draw_system(world: &mut World, d: &mut RaylibDrawHandle);
}

type EntityID = u64;

enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,

    CenterLeft,
    Center,
    CenterRight,

    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Anchor {
    fn values() -> Vec<Anchor> {
        vec![
            Anchor::TopLeft,
            Anchor::TopCenter,
            Anchor::TopRight,
            Anchor::CenterLeft,
            Anchor::Center,
            Anchor::CenterRight,
            Anchor::BottomLeft,
            Anchor::BottomCenter,
            Anchor::BottomRight,
        ]
    }
}

#[derive(Debug, Clone, Copy)]
struct BoundingBox2D {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl BoundingBox2D {
    fn new(x: f32, y: f32, w: f32, h: f32) -> BoundingBox2D {
        BoundingBox2D {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    fn new_v(pos: Vector2, size: Vector2) -> BoundingBox2D {
        BoundingBox2D::new(pos.x, pos.y, size.x, size.y)
    }

    fn width(&self) -> f32 {
        return self.x2 - self.x1;
    }

    fn height(&self) -> f32 {
        return self.y2 - self.y1;
    }

    fn center(&self) -> Vector2 {
        return Vector2::new(self.x1 + self.width() * 0.5, self.y1 + self.height() * 0.5);
    }

    fn calc(&self, anchor: Anchor) -> Vector2 {
        match anchor {
            Anchor::TopLeft => Vector2::new(self.x1, self.y1),
            Anchor::TopCenter => Vector2::new(self.x1 + self.width() * 0.5, self.y1),
            Anchor::TopRight => Vector2::new(self.x2, self.y1),
            Anchor::CenterLeft => Vector2::new(self.x1, self.y1 + self.height() * 0.5),
            Anchor::Center => self.center(),
            Anchor::CenterRight => Vector2::new(self.x2, self.y1 + self.height() * 0.5),
            Anchor::BottomLeft => Vector2::new(self.x1, self.y2),
            Anchor::BottomCenter => Vector2::new(self.x1 + self.width() * 0.5, self.y2),
            Anchor::BottomRight => Vector2::new(self.x2, self.y2),
        }
    }
}

impl Into<ffi::Rectangle> for BoundingBox2D {
    fn into(self) -> ffi::Rectangle {
        return ffi::Rectangle {
            x: self.x1,
            y: self.y1,
            width: self.x2 - self.x1,
            height: self.y2 - self.y1,
        };
    }
}

struct Base2D {
    name: String,
    bounds: BoundingBox2D,
    tint: Color,
    visible: bool,
}

impl Base2D {
    fn new(pos: Vector2, size: Vector2) -> Base2D {
        Base2D {
            name: "Unnamed".to_string(),
            bounds: BoundingBox2D::new_v(pos, size),
            tint: Color::WHITE,
            visible: true,
        }
    }
}

impl DrawSystem for Base2D {
    fn draw_system(world: &mut World, d: &mut RaylibDrawHandle) {
        // draw bases outlines
        world.base_components.iter().for_each(|b| {
            let base = &b.1;
            d.draw_rectangle_lines_ex(base.bounds, 1, base.tint);

            // draw all points
            for anchor in Anchor::values() {
                d.draw_circle_v(base.bounds.calc(anchor), 2.0, Color::RED);
            }
        });
    }
}

enum UIBarStyle {
    HIDDEN,
    INLINE,
    BOSS,
}

struct Health {
    max_health: u16,
    health: u16,
    bar_style: UIBarStyle,
}

impl Health {
    fn new(health: u16) -> Health {
        Health {
            max_health: health,
            health,
            bar_style: UIBarStyle::INLINE,
        }
    }

    fn take_damage(&mut self, amount: u16) {
        self.health -= amount;
    }
}

impl DrawSystem for Health {
    fn draw_system(world: &mut World, d: &mut RaylibDrawHandle) {
        world
            .base_components
            .iter()
            .filter_map(
                |b| match world.health_components.iter().find(|h| b.0 == h.0) {
                    Some(h) => Some((&b.1, &h.1)),
                    None => None,
                },
            )
            .for_each(|(b, h)| match h.bar_style {
                UIBarStyle::INLINE => {
                    let w = 80.0;
                    let h = 10.0;
                    let mut top_center = b.bounds.calc(Anchor::TopCenter);
                    top_center -= Vector2::new(0.0, 20.0);

                    let rect = Rectangle::new(top_center.x - w * 0.5, top_center.y - h * 0.5, w, h);
                    d.draw_rectangle_lines_ex(rect, 1, Color::WHITE);
                }
                UIBarStyle::BOSS => todo!(),
                _ => (),
            });
    }
}

struct World {
    last_entity: EntityID,
    base_components: Vec<(EntityID, Base2D)>,
    health_components: Vec<(EntityID, Health)>,
}

impl World {
    fn new() -> World {
        World {
            last_entity: 0,
            base_components: Vec::new(),
            health_components: Vec::new(),
        }
    }

    fn new_entity(&mut self) -> EntityID {
        self.last_entity += 1;
        return self.last_entity;
    }
}

const WIDTH: i32 = 640;
const HEIGHT: i32 = 480;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("SpaceGame")
        .build();

    let mut world = World::new();

    let player = world.new_entity();
    world.base_components.push((
        player,
        Base2D::new(Vector2::new(100.0, 280.0), Vector2::new(36.0, 48.0)),
    ));
    world.health_components.push((player, Health::new(20)));

    let god = world.new_entity();
    let mut base2d = Base2D::new(Vector2::new(400.0, 380.0), Vector2::new(76.0, 48.0));
    base2d.tint = Color::YELLOW;
    world.base_components.push((god, base2d));

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        let mut dt = 0.0;
        unsafe {
            dt = GetFrameTime();
        }

        Base2D::draw_system(&mut world, &mut d);
        Health::draw_system(&mut world, &mut d);

        d.clear_background(Color::BLACK);
        d.draw_fps(10, 10);
    }
}
