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
use image::{GrayImage, DynamicImage};
use std::convert::TryInto;
use image::Luma;

fn load_texture(file_path: &str) -> Option<GrayImage> {
    match image::open(file_path) {
        Ok(image) => Some(image.to_luma8()), // Convierte la imagen a escala de grises
        Err(e) => {
            eprintln!("Error loading texture: {}", e);
            None
        }
    }
}
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
            let color = match maze[row][col] {
                '-' => Color::new(105, 105, 105), 
                '|' => Color::new(135, 135, 135), 
                '+' => Color::new(115, 115, 115), 
                'p' => Color::new(255, 255, 0), 
                'g' => Color::new(255, 165, 0), 
                _ => Color::new(0, 50, 0), 
            };

            let x = col * block_size;
            let y = row * block_size;

            if x + block_size <= framebuffer.width && y + block_size <= framebuffer.height {
                framebuffer.set_current_color(color);
                framebuffer.draw_rectangle(x, y, block_size, block_size);
            }
        }
    }

    // Dibuja el jugador
    framebuffer.set_current_color(Color::new(255, 0, 0));
    let player_x = (player.pos.x / block_size as f32).round() as usize * block_size;
    let player_y = (player.pos.y / block_size as f32).round() as usize * block_size;
    framebuffer.set_pixel(player_x, player_y, framebuffer.current_color);

    // Dibuja los rayos
    let num_rays = 50;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        cast_ray(framebuffer, &maze, &player, a, block_size, true);
    }
}


fn render3d(framebuffer: &mut FrameBuffer, player: &Player, texture: &GrayImage) {
    let maze = match load_maze("maze.txt") {
        Ok(maze) => maze,
        Err(e) => {
            eprintln!("Failed to load maze: {}", e);
            return;
        }
    };

    let block_size = framebuffer.width / maze[0].len();
    let num_rays = framebuffer.width;
    let hh = framebuffer.height as f32 / 2.0;
    let distance_to_projection_plane = 100.0;

    let mut heights = vec![0; num_rays];

    // Primero, limpia el framebuffer con el color del fondo
    framebuffer.set_current_color(Color::new(0, 0, 50)); // Color de fondo
    framebuffer.draw_rectangle(0, 0, framebuffer.width, hh as usize);

    framebuffer.set_current_color(Color::new(0, 50, 0)); // Color del suelo
    framebuffer.draw_rectangle(0, hh as usize, framebuffer.width, framebuffer.height - hh as usize);

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);

        let distance_to_wall = intersect.distance as f32;
        let stake_height = (hh / distance_to_wall) * distance_to_projection_plane;
        heights[i] = stake_height as usize;
    }

    framebuffer.set_current_color(Color::new(255, 0, 0));

    for i in 0..num_rays {
        let stake_height = heights[i];
        let stake_top = (hh - (stake_height as f32 / 2.0)) as usize;
        let stake_bottom = (hh + (stake_height as f32 / 2.0)) as usize;

        if i < framebuffer.width {
            for y in stake_top..stake_bottom {
                if y >= framebuffer.height {
                    continue; // Evitar acceso fuera de los límites del framebuffer
                }
                
                let texture_y = ((y - stake_top) as f32 / stake_height as f32 * texture.height() as f32) as u32;
                let texture_y = texture_y.min(texture.height() - 1) as usize; // Convertir a usize
                
                let pixel = texture.get_pixel(i.min(texture.width() as usize - 1) as u32, texture_y as u32);
                let luminance = pixel[0]; // La imagen en escala de grises tiene solo un componente
                let color = Color::new(luminance, luminance, luminance); 
                
                framebuffer.set_pixel(i, y, color);
            }
        } else {
            eprintln!("Índice i fuera del rango: {}", i);
        }
    }

    // Dibuja el minimapa en la esquina inferior derecha
    let minimap_size = 150; // Tamaño del minimapa
    let minimap_x = framebuffer.width - minimap_size;
    let minimap_y = framebuffer.height - minimap_size;

    // Dibuja el minimapa fondo
    framebuffer.set_current_color(Color::new(0, 0, 0));
    framebuffer.draw_rectangle(minimap_x, minimap_y, minimap_size, minimap_size);

    // Dibuja el laberinto en el minimapa
    let minimap_block_size = minimap_size / maze[0].len();
    for row in 0..maze.len() {
        for col in 0..maze[row].len() {
            let color = match maze[row][col] {
                '-' => Color::new(105, 105, 105), 
                '|' => Color::new(135, 135, 135), 
                '+' => Color::new(115, 115, 115), 
                'p' => Color::new(255, 255, 0), 
                'g' => Color::new(255, 165, 0), 
                _ => Color::new(0, 50, 0), 
            };

            let x = minimap_x + col * minimap_block_size;
            let y = minimap_y + row * minimap_block_size;

            if x + minimap_block_size <= framebuffer.width && y + minimap_block_size <= framebuffer.height {
                framebuffer.set_current_color(color);
                framebuffer.draw_rectangle(x, y, minimap_block_size, minimap_block_size);
            }
        }
    }

    // Dibuja la ubicación del jugador en el minimapa
    let player_minimap_x = minimap_x + (player.pos.x / block_size as f32 * minimap_block_size as f32) as usize;
    let player_minimap_y = minimap_y + (player.pos.y / block_size as f32 * minimap_block_size as f32) as usize;
    framebuffer.set_current_color(Color::new(255, 0, 0)); // Color del jugador en el minimapa
    framebuffer.set_pixel(player_minimap_x, player_minimap_y, framebuffer.current_color);
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
        "Graficos por computadora - Proyecto1",
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

    // Cargar la textura
    let texture = match load_texture("assets/textura.png") {
        Some(tex) => tex,
        None => {
            eprintln!("Failed to load texture.");
            return;
        }
    };

    // Inicializar jugador
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
            render3d(&mut framebuffer, &player, &texture);
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
