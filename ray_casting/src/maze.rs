use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// Función para cargar el laberinto desde un archivo
pub fn load_maze(file_path: &str) -> Result<Vec<Vec<char>>, Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);
    let mut maze = Vec::new();
    let mut row_length = None;

    for line in reader.lines() {
        let line = line?;
        let row: Vec<char> = line.chars().collect();
        
        // Verificar que todas las filas tengan la misma longitud
        if let Some(length) = row_length {
            if row.len() != length {
                return Err("Inconsistent row length in maze file".into());
            }
        } else {
            row_length = Some(row.len());
        }
        
        maze.push(row);
    }

    Ok(maze)
}
