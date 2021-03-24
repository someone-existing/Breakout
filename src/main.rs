use graphics::{
    mesh::{Mesh, ShapeStyle},
    Color, DrawParams,
};
use hecs::{Entity, World};
use tetra::{
    graphics::{self, Rectangle},
    input::{is_key_down, Key},
    math::Vec2,
    window::quit,
    Context, ContextBuilder, State,
};

const WIN_W: f32 = 1280.0;
const WIN_H: f32 = 720.0;

const BRICK_COLS: f32 = 10.0;
const BRICK_ROWS: f32 = 6.0;
const BRICK_GAP: f32 = 10.0;

const BALL_RADIUS: f32 = 15.0;
const BALL_SPEED: f32 = 10.0;
const PADDLE_SPEED: f32 = 25.0;

struct Paddle;
struct Ball;
struct Brick;

fn paddle_movement_system(world: &mut World, ctx: &Context) {
    for (_id, (_paddle, pos, size)) in world.query_mut::<(&Paddle, &mut Position, &Size)>() {
        if is_key_down(ctx, Key::Right) && pos.x + size.w < WIN_W {
            pos.x += PADDLE_SPEED;
        }

        if is_key_down(ctx, Key::Left) && pos.x > 0.0 {
            pos.x -= PADDLE_SPEED;
        }
    }
}

fn ball_movement_system(world: &mut World) {
    for (_id, (_ball, pos, dir)) in world.query_mut::<(&Ball, &mut Position, &mut Direction)>() {
        pos.x += BALL_SPEED * dir.x;
        pos.y += BALL_SPEED * dir.y;
    }
}

fn ball_collision_system(world: &mut World, ball_id: Entity) {
    let ball_pos = world.get::<Position>(ball_id).unwrap();
    let ball_rect = Rectangle::new(
        ball_pos.x - BALL_RADIUS / 2.0,
        ball_pos.y - BALL_RADIUS / 2.0,
        BALL_RADIUS,
        BALL_RADIUS,
    );

    let mut x = false;
    let mut y = false;
    if ball_pos.x - BALL_RADIUS / 2.0 < 0.0 || ball_pos.x + BALL_RADIUS / 2.0 > WIN_W {
        x = true;
    }
    if ball_pos.y - BALL_RADIUS / 2.0 < 0.0 {
        y = true;
    }
    for (_id, (_paddle, pos, size)) in world.query::<(&Paddle, &Position, &Size)>().iter() {
        let paddle_rect = Rectangle::new(pos.x, pos.y, size.w, size.h);
        if ball_rect.intersects(&paddle_rect) {
            y = true;
        }
    }
    for (_id, (_brick, pos, size, to_delete)) in world
        .query::<(&Brick, &Position, &Size, &mut bool)>()
        .iter()
    {
        let brick_rect = Rectangle::new(pos.x, pos.y, size.w, size.h);
        if ball_rect.intersects(&brick_rect) {
            *to_delete = true;
            if ball_pos.x + BALL_RADIUS / 2.0 < pos.x
                && ball_pos.x - BALL_RADIUS / 2.0 > pos.x + size.w
            {
                x = true;
            } else {
                y = true;
            }
        }
    }
    if x == true {
        world.get_mut::<Direction>(ball_id).unwrap().x *= -1.0;
    }

    if y == true {
        world.get_mut::<Direction>(ball_id).unwrap().y *= -1.0;
    }
}

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}
struct Direction {
    x: f32,
    y: f32,
}
struct Size {
    w: f32,
    h: f32,
}

struct GameState {
    world: World,
    paddle: Entity,
    ball: Entity,
}

impl GameState {
    fn new(_ctx: &mut Context) -> tetra::Result<GameState> {
        let mut world = World::new(); // create world
        let paddle = world.spawn((
            Paddle,
            Position {
                x: WIN_W / 2.0 - 100.0,
                y: WIN_H - 50.0,
            },
            Size { w: 200.0, h: 20.0 },
        )); // spawn paddle into the world
        let ball = world.spawn((
            Ball,
            Position {
                x: WIN_W / 2.0,
                y: WIN_H - 100.0,
            },
            Direction { x: 0.5, y: -0.5 },
        )); // spawn ball into the world

        // spawn bricks into the world
        let width = (WIN_W - BRICK_GAP) / BRICK_COLS - BRICK_GAP; // brick width
        let height = (WIN_H / 2.0 - BRICK_GAP) / BRICK_ROWS - BRICK_GAP; // brick height
        for col in 0..BRICK_COLS as i32 {
            for row in 0..BRICK_ROWS as i32 {
                world.spawn((
                    Brick,
                    Position {
                        x: col as f32 * width + (col + 1) as f32 * BRICK_GAP,
                        y: row as f32 * height + (row + 1) as f32 * BRICK_GAP,
                    },
                    Size {
                        w: width,
                        h: height,
                    },
                    false,
                ));
            }
        }
        // return game state
        Ok(GameState {
            world,
            paddle,
            ball,
        })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::BLACK);

        for (_id, (_brick, pos, size)) in self.world.query::<(&Brick, &Position, &Size)>().iter() {
            let brick_mesh = Mesh::rectangle(
                ctx,
                ShapeStyle::Fill,
                Rectangle::new(pos.x, pos.y, size.w, size.h),
            )?;

            brick_mesh.draw(ctx, DrawParams::default().color(Color::WHITE));
        }

        let paddle_pos = self.world.get::<Position>(self.paddle).unwrap();
        let paddle_size = self.world.get::<Size>(self.paddle).unwrap();
        let paddle_mesh = Mesh::rectangle(
            ctx,
            ShapeStyle::Fill,
            Rectangle::new(paddle_pos.x, paddle_pos.y, paddle_size.w, paddle_size.h),
        )?;
        paddle_mesh.draw(ctx, DrawParams::default().color(Color::WHITE));

        let ball_pos = self.world.get::<Position>(self.ball).unwrap();
        let ball_mesh = Mesh::circle(
            ctx,
            ShapeStyle::Fill,
            Vec2::new(ball_pos.x, ball_pos.y),
            BALL_RADIUS,
        )?;
        ball_mesh.draw(ctx, DrawParams::default().color(Color::WHITE));

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        paddle_movement_system(&mut self.world, ctx);
        ball_movement_system(&mut self.world);
        ball_collision_system(&mut self.world, self.ball);

        let mut queue = vec![self.ball];
        queue.pop();
        for (id, (_brick, to_delete)) in self.world.query::<(&Brick, &bool)>().iter() {
            if *to_delete {
                queue.push(id);
            }
        }
        for i in 0..queue.len() {
            self.world.despawn(queue[i]).unwrap();
        }

        if self.world.get::<Position>(self.ball).unwrap().y >= WIN_H {
            quit(ctx);
        }
        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Breakout", WIN_W as i32, WIN_H as i32)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
