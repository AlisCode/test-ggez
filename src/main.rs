extern crate ggez;
//extern crate nphysics2d;
extern crate specs;

use ggez::*;
use ggez::event::*;
use ggez::graphics::{Point2, Color, Rect, DrawMode};

use specs::*;

struct MainState<'a, 'b> {
    world: World,
    logic_dispatcher: Dispatcher<'a, 'b>,
}

#[derive(Debug)]
struct Vel {
    value: Point2,
    speed: f32,
}

impl Vel {
    pub fn new(x: f32, y: f32, speed: f32) -> Self {
        Vel {
            value: Point2::new(x, y),
            speed,
        }
    }
}

impl Component for Vel {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct Pos {
    value: Point2,
}

impl Pos {
    pub fn new(x: f32, y: f32) -> Self {
        Pos {
            value: Point2::new(x, y),
        }
    }
}

impl Component for Pos {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct Drawable {
    rect: Rect,
    color: Color,
}

struct PlayerInput {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

impl PlayerInput {
    pub fn new() -> Self {
        PlayerInput {
            left: false,
            right: false,
            up: false,
            down: false,
        }
    }

    pub fn reset(&mut self) {
        self.left = false;
        self.right = false;
        self.up = false;
        self.down = false;
    }
}

struct Controlled;

impl Controlled {
    pub fn new() -> Self {
        Controlled {}
    }
}

impl Component for Controlled {
    type Storage = VecStorage<Self>;
}

struct SystemVelControlled;

impl<'a> System<'a> for SystemVelControlled {
    type SystemData = (WriteStorage<'a, Vel>, ReadStorage<'a, Controlled>, Fetch<'a, PlayerInput>);
    fn run(&mut self, (mut vel, controlled, player_input): Self::SystemData) {
        for (vel, controlled) in (&mut vel, &controlled).join() {
            vel.value.x = 0.0;
            vel.value.y = 0.0;
            if player_input.left {
                vel.value.x = -1.0;
            }
            if player_input.right {
                vel.value.x = 1.0;
            }
            if player_input.up {
                vel.value.y = -1.0;
            }
            if player_input.down {
                vel.value.y = 1.0;
            }
        }
    }
}

impl Drawable {
    pub fn new(width: i32, height: i32, color: Color) -> Self {
        Drawable {
            rect: Rect::new_i32(0, 0, width, height),
            color,
        }
    }

    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.rect.x = x;
        self.rect.y = y;
    }

    pub fn draw(&self, ctx: &mut Context) {
        graphics::set_color(ctx, self.color).unwrap();
        graphics::rectangle(ctx, DrawMode::Fill, self.rect).unwrap();
    }
}

impl Component for Drawable {
    type Storage = VecStorage<Self>;
}

struct SystemPosVel;

impl<'a> System<'a> for SystemPosVel {
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.value.x += vel.value.x * vel.speed;
            pos.value.y += vel.value.y * vel.speed;
        }
    }
}

struct SystemDrawable<'c> {
    ctx: &'c mut Context,
}

impl<'c> SystemDrawable<'c> {
    pub fn new(ctx: &'c mut Context) -> Self {
        SystemDrawable {
            ctx,
        }
    }
}

impl<'a, 'c> System<'a> for SystemDrawable<'c> {
    type SystemData = ReadStorage<'a, Drawable>;
    fn run(&mut self, drawable: Self::SystemData) {
        for drawable in (&drawable).join() {
            drawable.draw(self.ctx);
        }
    }
}

struct SystemPosDrawable;

impl<'a> System<'a> for SystemPosDrawable {
    type SystemData = (ReadStorage<'a, Pos>, WriteStorage<'a, Drawable>);

    fn run(&mut self, (pos, mut drawable): Self::SystemData) {
        for (pos, drawable) in (&pos, &mut drawable).join() {
            drawable.set_pos(pos.value.x, pos.value.y);
        }
    }
}

impl<'a, 'b> MainState<'a, 'b> {
    fn new(world: World) -> GameResult<MainState<'a, 'b>> {
        let logic_dispatcher: Dispatcher = DispatcherBuilder::new()
            .add(SystemVelControlled, "sys_vel_controlled", &[])
            .add(SystemPosVel, "sys_pos_vel", &["sys_vel_controlled"])
            .add(SystemPosDrawable, "sys_pos_draw", &["sys_pos_vel"])
            .build();

        let state = MainState {
            world,
            logic_dispatcher,
        };
        Ok(state)
    }
}


const THRESHOLD_JOYSTICK: i16 = 1000;

impl<'a, 'b> event::EventHandler for MainState<'a, 'b> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.logic_dispatcher.dispatch(&mut self.world.res);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        {
            let mut drawing_system = SystemDrawable::new(ctx);
            drawing_system.run_now(&mut self.world.res);
        }
        graphics::present(ctx);

        Ok(())
    }

    fn controller_axis_event(&mut self, _ctx: &mut Context, axis: Axis, value: i16, instance_id: i32) {
        let mut player_input = self.world.write_resource
            ::<PlayerInput>();

        match axis {
            Axis::LeftX if value >= THRESHOLD_JOYSTICK => {
                player_input.left = false;
                player_input.right = true
            }
            Axis::LeftX if value <= -THRESHOLD_JOYSTICK => {
                player_input.left = true;
                player_input.right = false;
            }
            Axis::LeftX if value > -THRESHOLD_JOYSTICK && value < THRESHOLD_JOYSTICK => {
                player_input.left = false;
                player_input.right = false;
            }
            Axis::LeftY if value >= THRESHOLD_JOYSTICK => {
                player_input.up = false;
                player_input.down = true
            }
            Axis::LeftY if value <= -THRESHOLD_JOYSTICK => {
                player_input.up = true;
                player_input.down = false;
            }
            Axis::LeftY if value > -THRESHOLD_JOYSTICK && value < THRESHOLD_JOYSTICK => {
                player_input.up = false;
                player_input.down = false;
            }
            _ => (),
        }
    }
}

fn main() {
    let c = conf::Conf::new();
    let context = &mut Context::load_from_conf("test-ggez-specs", "ggez", c).unwrap();

    let mut world = World::new();
    world.register::<Pos>();
    world.register::<Vel>();
    world.register::<Drawable>();
    world.register::<Controlled>();
    world.add_resource(PlayerInput::new());

    world.create_entity().with(Vel::new(0.0, 0.0, 5.0)).with(Pos::new(0.0, 0.0)).with(Drawable::new(20, 20, Color::new(1.0, 0.0, 0.0, 1.0))).with(Controlled::new()).build();
    world.create_entity().with(Vel::new(1.0, 1.0, 1.0)).with(Pos::new(50.0, 0.0)).with(Drawable::new(20, 20, Color::new(0.0, 1.0, 0.0, 1.0))).build();
    world.create_entity().with(Vel::new(1.0, 1.0, 1.0)).with(Pos::new(100.0, 0.0)).with(Drawable::new(20, 20, Color::new(0.0, 0.0, 1.0, 1.0))).build();

    let state = &mut MainState::new(world).unwrap();
    event::run(context, state).unwrap();
}
