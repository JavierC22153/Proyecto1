use std::f32::consts::PI;
use nalgebra_glm::Vec2;
use minifb::{Window, Key};

pub struct Player {
    pub pos: Vec2,
    pub a: f32,
    pub fov: f32,
    pub mouse_sensitivity: f32,
    pub last_mouse_x: f32,
}

pub fn process_event(window: &Window, player: &mut Player) {
    const SPEED: f32 = 5.0;
    const ROTATION_SPEED: f32 = PI / 10.0;

    // Rotación con las teclas izquierda y derecha
    if window.is_key_down(Key::Left) {
        player.a -= ROTATION_SPEED;
    }
    if window.is_key_down(Key::Right) {
        player.a += ROTATION_SPEED;
    }

    // Movimiento hacia adelante y hacia atrás
    let (dx, dy) = (SPEED * player.a.cos(), SPEED * player.a.sin());

    if window.is_key_down(Key::Up) {
        player.pos.x += dx;
        player.pos.y += dy;
    }
    if window.is_key_down(Key::Down) {
        player.pos.x -= dx;
        player.pos.y -= dy;
    }

    // Rotación con el mouse
    if let Some(mouse_pos) = window.get_mouse_pos(minifb::MouseMode::Discard) {
        let mouse_x = mouse_pos.0 as f32;
        let mouse_sensitivity = player.mouse_sensitivity;

        // Calcular el cambio en la posición del mouse
        let delta_x = mouse_x - player.last_mouse_x;

        // Ajustar el ángulo basado en el cambio del mouse
        player.a -= delta_x * mouse_sensitivity;

        // Actualizar la última posición del mouse
        player.last_mouse_x = mouse_x;

        // Opcional: Mantén el ángulo en el rango [0, 2π)
        player.a = player.a % (2.0 * PI);
    }
}
