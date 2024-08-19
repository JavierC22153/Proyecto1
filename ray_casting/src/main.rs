mod framebuffer;
mod color;
mod bmp;
mod maze;
mod player;
mod caster;

use std::time::{Instant, Duration};
use minifb::{Window, WindowOptions, Key};
use nalgebra_glm::Vec2;
use crate::framebuffer::FrameBuffer;
use crate::color::Color;
use crate::maze::load_maze;
use crate::caster::cast_ray;
use crate::player::{Player, process_event};
use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, Sink};

const WIDTH: usize = 1000;
const HEIGHT: usize = 900;

fn render2d(framebuffer: &mut FrameBuffer, player: &mut Player) {
    let maze = match load_maze("maze.txt") {
        Ok(maze) => maze,
        Err(e) => {
            eprintln!("Failed to load maze: {}", e);
            return;
        }
    };

    let block_size = WIDTH / maze[0].len(); // Tamaño del bloque basado en el ancho del framebuffer
    let maze_height = HEIGHT / maze.len(); // Tamaño del bloque basado en la altura del framebuffer

    // Dibuja el laberinto
    for row in 0..maze.len() {
        for col in 0..maze[row].len() {
            if maze[row][col] != ' ' {
                let x = col * block_size;
                let y = row * block_size;

                if x + block_size <= framebuffer.width && y + block_size <= framebuffer.height {
                    framebuffer.draw_rectangle(x, y, block_size, block_size);
                }
            }
        }
    }

    // Dibuja el jugador
    framebuffer.set_current_color(Color::new(255, 0, 0));
    let player_x = (player.pos.x / block_size as f32).round() as usize * block_size;
    let player_y = (player.pos.y / block_size as f32).round() as usize * block_size;
    framebuffer.set_pixel(player_x, player_y);

    // Dibuja los rayos
    let num_rays = 50;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        cast_ray(framebuffer, &maze, &player, a, block_size, true);
    }
}

fn render3d(framebuffer: &mut FrameBuffer, player: &Player) {
    let maze = match load_maze("maze.txt") {
        Ok(maze) => maze,
        Err(e) => {
            eprintln!("Failed to load maze: {}", e);
            return;
        }
    };

    let block_size = WIDTH / maze[0].len(); // Tamaño del bloque basado en el ancho del framebuffer
    let num_rays = framebuffer.width;
    let hh = framebuffer.height as f32 / 2.0;
    let distance_to_projection_plane = 100.0;

    // Crear un buffer temporal para almacenar las alturas de las paredes
    let mut heights = vec![0; num_rays];

    // Calcula la altura de cada rayo
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);

        let distance_to_wall = intersect.distance as f32; // Convertir a f32
        let stake_height = (hh / distance_to_wall) * distance_to_projection_plane;
        heights[i] = stake_height as usize;
    }

    // Dibuja las paredes en una sola pasada
    framebuffer.set_current_color(Color::new(255, 0, 0)); // Color de las paredes, ajusta según sea necesario

    for i in 0..num_rays {
        let stake_height = heights[i];
        let stake_top = (hh - (stake_height as f32 / 2.0)) as usize;
        let stake_bottom = (hh + (stake_height as f32 / 2.0)) as usize;
        framebuffer.draw_rectangle(i, stake_top, 1, stake_bottom - stake_top);
    }
}


fn calculate_fps(last_update: &mut Instant, frame_count: &mut u32) -> f32 {
    *frame_count += 1;
    let duration = last_update.elapsed();

    if duration >= Duration::from_secs(1) {
        let fps = *frame_count as f32 / duration.as_secs_f32();
        *frame_count = 0;
        *last_update = Instant::now();
        fps
    } else {
        -1.0
    }
}


fn main() {
    // Inicializar framebuffer
    let framebuffer_width = WIDTH;
    let framebuffer_height = HEIGHT;
    let mut framebuffer = FrameBuffer::new(framebuffer_width, framebuffer_height);
    framebuffer.set_current_color(Color::new(50, 50, 100));

    // Inicializar ventana
    let mut window = match Window::new(
        "Rust Graphics - Render Loop Example",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    ) {
        Ok(win) => win,
        Err(e) => {
            eprintln!("Error creating window: {}", e);
            return;
        }
    };

    // Inicializar jugador
    // Asegúrate de que la posición inicial del jugador sea coherente con el tamaño del laberinto
    let initial_mouse_pos = window.get_mouse_pos(minifb::MouseMode::Discard).unwrap_or((0.0, 0.0));
    let mut player = Player {
        pos: Vec2::new(250.0, 150.0), // Ajusta esto según el tamaño de bloque y la posición deseada
        a: std::f32::consts::PI / 3.0,
        fov: std::f32::consts::PI / 3.0,
        mouse_sensitivity: 0.005,
        last_mouse_x: initial_mouse_pos.0 as f32,
    };

    let mut mode = "2D";

    // Variables para el cálculo de FPS
    let mut last_update = Instant::now();
    let mut frame_count = 0;

    // Inicializar sistema de audio
    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok((stream, handle)) => (stream, handle),
        Err(e) => {
            eprintln!("Error initializing audio system: {}", e);
            return;
        }
    };

    // Cargar y reproducir música de fondo
    let file = match File::open("assets/background_music.mp3") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening audio file: {}", e);
            return;
        }
    };

    let source = match Decoder::new(BufReader::new(file)) {
        Ok(decoder) => decoder,
        Err(e) => {
            eprintln!("Error creating audio decoder: {}", e);
            return;
        }
    };

    let sink = match Sink::try_new(&stream_handle) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error creating audio sink: {}", e);
            return;
        }
    };

    sink.append(source);
    sink.play();

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        if window.is_key_down(Key::D) {
            mode = if mode == "2D" { "3D" } else { "2D" };
        }

        // Procesar eventos del jugador (movimiento y rotación)
        process_event(&window, &mut player);

        framebuffer.clear(); // Limpiar el framebuffer al principio

        if mode == "2D" {
            framebuffer.set_current_color(Color::new(50, 50, 100));
            render2d(&mut framebuffer, &mut player);
        } else {
            framebuffer.set_current_color(Color::new(50, 50, 100));
            render3d(&mut framebuffer, &player);
        }

        // Crear un buffer temporal para el framebuffer
        let temp_buffer = framebuffer.cast_buffer();

        // Calcular FPS e imprimir el texto
        let fps = calculate_fps(&mut last_update, &mut frame_count);
        if fps != -1.0 {
            println!("FPS: {:.1}", fps);
        }

        // Actualizar el buffer de la ventana
        if let Err(e) = window.update_with_buffer(&temp_buffer, framebuffer_width, framebuffer_height) {
            eprintln!("Error updating window buffer: {}", e);
            break;
        }

        std::thread::sleep(Duration::from_millis(16));
    }
}