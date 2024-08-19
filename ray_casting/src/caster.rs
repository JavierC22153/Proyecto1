use crate::framebuffer::FrameBuffer;
use crate::player::Player;
use crate::color::Color;

pub struct Intersect {
    pub distance: f32,
    pub impact: char
}

pub fn cast_ray(framebuffer: &mut FrameBuffer, maze: &Vec<Vec<char>>, player: &Player,
    a: f32, block_size: usize, draw_line: bool) -> Intersect {
    let mut d = 0.0;
    let mut x;
    let mut y;

    // Establecer el color actual
    framebuffer.set_current_color(Color::new(255, 0, 0));

    loop {
        let cos = a.cos();
        let sin = a.sin();
        x = (player.pos.x + d * cos) as usize;
        y = (player.pos.y + d * sin) as usize;

        // Verificar que x y y están dentro de los límites del framebuffer
        if x >= framebuffer.width || y >= framebuffer.height {
            break;
        }

        let i = x / block_size;
        let j = y / block_size;

        // Verificar que i y j están dentro de los límites de maze
        if j >= maze.len() || i >= maze[j].len() {
            d += 0.1;
            continue;
        }

        if draw_line {
            // Usar draw_rectangle para dibujar un píxel de 1x1
            framebuffer.draw_rectangle(x, y, 1, 1);  
        }

        if maze[j][i] != ' ' {
            return Intersect {
                distance: d,
                impact: maze[j][i]
            };
        }

        d += 0.1;
    }

    // Si no hay intersección, devolver un valor predeterminado o manejar el caso
    Intersect {
        distance: d,
        impact: ' '
    }
}
