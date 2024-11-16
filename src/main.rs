use flecs_ecs::prelude::*;
use raylib::prelude::*;

#[derive(Component, Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Velocity {
    x: i32,
    y: i32,
}

#[derive(Component, Default)]
struct Sprite {
    color: Color,
    x: i32,
    y: i32,
}

#[derive(Component, Default)]
struct InputState {
    move_up: bool,
    move_down: bool,
    move_left: bool,
    move_right: bool,
}

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Stats {
    move_speed: u32,
}

#[derive(Component, Debug)]
struct Pickable;

#[derive(Component)]
struct Collider {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

fn main() {
    // Inicia la ventana con raylib
    let (mut rl, thread) = raylib::init().size(1280, 720).title("Hello, World").build();
    rl.set_target_fps(60);
    // Crear un nuevo mundo Flecs
    let world = World::new();
    world.set(InputState::default());

    // Crear entidades y asignarles posiciones
    for i in 0..3 {
        let e = world
            .entity()
            .set(Position {
                x: 10 + (i * 30),
                y: 10,
            })
            .set(Velocity { x: 0, y: 0 })
            .set(Stats { move_speed: 1 })
            .set(Collider {
                x: -8,
                y: -18,
                w: 16,
                h: 18,
            })
            .set(Sprite {
                color: Color::RED,
                x: 0,
                y: -10,
            });

        if i == 1 {
            e.add::<Player>();
            e.set(Sprite {
                color: Color::GREEN,
                x: 0,
                y: -10,
            })
            .set_name("player");
        };
    }

    let coin_entity = world
        .entity()
        .set(Position { x: 10, y: 40 })
        .set(Sprite {
            color: Color::YELLOW,
            x: 0,
            y: -10,
        })
        .set(Collider {
            x: -8,
            y: -18,
            w: 16,
            h: 18,
        })
        .set_name("coin")
        .add::<Pickable>();

    let q = world.query::<(&mut Position, &Velocity, &Stats)>().build();
    let query_draw_ordered = world
        .query::<(&Position, &Sprite)>()
        .order_by::<Position>(|_, p1: &Position, _, p2: &Position| {
            (p1.y > p2.y) as i32 - (p1.y < p2.y) as i32
        })
        .build();
    let player_query = world.query::<(&Position, &mut Velocity, &Player)>().build();
    let coin_query = world.query::<(&Position, &Pickable)>().build();
    let query_collider_draw = world.query::<(&Collider, &Position)>().build();
    let query_origin_draw = world.query::<&Position>().build();

    // Creates REST server on default port (27750)
    world.set(flecs::rest::Rest::default());

    let mut camera = Camera2D {
        target: Vector2 { x: 0.0, y: 0.0 },
        offset: Vector2 { x: 400.0, y: 300.0 },
        rotation: 0.0,
        zoom: 3.0, // Inicializar sin zoom
    };

    // Runs the system serving up REST requests

    while !rl.window_should_close() && world.progress() {
        // INPUT --------------------------------------------------------------
        let wheel_move = rl.get_mouse_wheel_move();
        camera.zoom += wheel_move * 0.1;
        camera.zoom = camera.zoom.clamp(2.0, 5.0);
        world.set(InputState {
            move_right: rl.is_key_down(KeyboardKey::KEY_D),
            move_left: rl.is_key_down(KeyboardKey::KEY_A),
            move_up: rl.is_key_down(KeyboardKey::KEY_W),
            move_down: rl.is_key_down(KeyboardKey::KEY_S),
        });

        world.get::<&InputState>(|state| {
            player_query.each(|(_, vel, _)| {
                let input_up = if state.move_up { -1 } else { 0 };
                let input_down = if state.move_down { 1 } else { 0 };

                vel.y = input_up + input_down; // Move up y move down

                let input_left = if state.move_left { -1 } else { 0 };
                let input_right = if state.move_right { 1 } else { 0 };

                vel.x = input_left + input_right;
            });
        });

        q.each(|(pos, vel, stats)| {
            pos.x += vel.x * stats.move_speed as i32;
            pos.y += vel.y * stats.move_speed as i32;
        });

        world.defer_begin();

        coin_query.each_entity(|entity, (coin_pos, _)| {
            player_query.each_entity(|_, (player_pos, _, _)| {
                // Cálculo de los límites de los coliders
                let player_rect = (
                    player_pos.x + player_pos.x, // Posición X del jugador
                    player_pos.y + player_pos.y, // Posición Y del jugador
                    16, // Ancho del collider del jugador (suponiendo que sea 16)
                    18, // Alto del collider del jugador (suponiendo que sea 18)
                );
                let coin_rect = (
                    coin_pos.x + coin_pos.x, // Posición X de la moneda
                    coin_pos.y + coin_pos.y, // Posición Y de la moneda
                    16,                      // Ancho del collider de la moneda
                    18,                      // Alto del collider de la moneda
                );

                // Comprobación de colisión
                if player_rect.0 < coin_rect.0 + coin_rect.2
                    && player_rect.0 + player_rect.2 > coin_rect.0
                    && player_rect.1 < coin_rect.1 + coin_rect.3
                    && player_rect.1 + player_rect.3 > coin_rect.1
                {
                    println!("+1");
                    entity.destruct(); // Destruir la moneda si hay colisión
                }
            });
        });

        world.defer_end();

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);
        let mut draw = d.begin_mode2D(camera);

        query_draw_ordered.each(|(pos, spr)| {
            draw.draw_circle(pos.x as i32 + spr.x, pos.y as i32 + spr.y, 10.0, spr.color);
        });

        query_origin_draw.each(|pos| {
            draw.draw_circle(pos.x as i32, pos.y as i32, 1.0, Color::RED);
        });

        query_collider_draw.each(|(col, pos)| {
            draw.draw_rectangle(
                pos.x + col.x,
                pos.y + col.y,
                col.w as i32,
                col.h as i32,
                Color::new(255, 0, 0, 128),
            );
        });

        // colliders_draw.each(|(col)| {
        //     d.draw_rectangle(
        //         // pos.x + col.x,
        //         // pos.y + col.y,
        //         col.x,
        //         col.y,
        //         col.w as i32,
        //         col.h as i32,
        //         Color::new(255, 0, 0, 128),
        //     );
        // });
    }
}
