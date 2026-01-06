pub const ANIMATION_FRAMES: usize = 32;

const BULL: &str = "🐂";
const BEAR: &str = "🐻";

const STARS: [char; 6] = ['✦', '⋆', '·', '∘', '°', ' '];
const CLOUDS: [&str; 4] = ["☁", "░░", "▒", "░"];
const DUST: [char; 4] = ['✦', '•', '∘', '·'];
const GROUND_CHARS: [char; 6] = ['▁', '▂', '▁', '▂', '▃', '▂'];
const GRASS: [char; 4] = ['⌇', '╻', '┃', '╽'];

struct Layer {
    chars: Vec<Vec<char>>,
}

impl Layer {
    fn new(height: usize, width: usize) -> Self {
        Self {
            chars: vec![vec![' '; width]; height],
        }
    }

    fn set(&mut self, row: usize, col: usize, ch: char) {
        if row < self.chars.len() && col < self.chars[0].len() {
            self.chars[row][col] = ch;
        }
    }

    fn get(&self, row: usize, col: usize) -> char {
        if row < self.chars.len() && col < self.chars[0].len() {
            self.chars[row][col]
        } else {
            ' '
        }
    }
}

fn hash(a: usize, b: usize) -> usize {
    let mut x = a.wrapping_mul(374761393);
    x = x.wrapping_add(b.wrapping_mul(668265263));
    x = (x ^ (x >> 13)).wrapping_mul(1274126177);
    x ^ (x >> 16)
}

fn render_starfield(frame: usize, width: usize, reverse: bool) -> Layer {
    let mut layer = Layer::new(1, width);
    let scroll = frame / 4;

    for col in 0..width {
        let world_col = if reverse {
            col.wrapping_sub(scroll)
        } else {
            col.wrapping_add(scroll)
        };
        let star_hash = hash(world_col, 42);

        if star_hash % 12 == 0 {
            let star_type = (star_hash / 12) % STARS.len();
            let twinkle = (frame.wrapping_add(col)) % 8 < 4;
            if twinkle || star_type > 2 {
                layer.set(0, col, STARS[star_type]);
            }
        }
    }
    layer
}

fn render_clouds(frame: usize, width: usize, reverse: bool) -> (Layer, Vec<(usize, &'static str)>) {
    let layer = Layer::new(1, width);
    let scroll = frame / 3;
    let mut cloud_positions = Vec::new();

    for i in 0..3 {
        let base_pos = (i * width / 3).wrapping_add(hash(i, 99) % 10);
        let cloud_col = if reverse {
            (base_pos.wrapping_add(scroll)) % width
        } else {
            (base_pos.wrapping_add(width).wrapping_sub(scroll % width)) % width
        };
        let cloud_type = hash(i, 77) % CLOUDS.len();

        if cloud_col < width.saturating_sub(2) {
            cloud_positions.push((cloud_col, CLOUDS[cloud_type]));
        }
    }

    (layer, cloud_positions)
}

fn render_ground(frame: usize, width: usize, going_left: bool) -> Layer {
    let mut layer = Layer::new(1, width);
    let scroll = if going_left {
        frame
    } else {
        frame.wrapping_neg()
    };

    for col in 0..width {
        let world_col = col.wrapping_add(scroll);
        let ground_hash = hash(world_col, 123);

        let base_char = GROUND_CHARS[ground_hash % GROUND_CHARS.len()];
        layer.set(0, col, base_char);

        if ground_hash % 8 == 0 {
            let grass = GRASS[(ground_hash / 8) % GRASS.len()];
            layer.set(0, col, grass);
        }
    }
    layer
}

fn render_bull(frame: usize, width: usize) -> Vec<String> {
    let height = 5;
    let progress = (frame % ANIMATION_FRAMES) as f32 / (ANIMATION_FRAMES - 1) as f32;
    let max_pos = width.saturating_sub(4);
    let bull_pos = max_pos.saturating_sub((progress * max_pos as f32) as usize);

    let stars = render_starfield(frame, width, false);
    let (_, cloud_positions) = render_clouds(frame, width, false);
    let ground = render_ground(frame, width, true);

    let mut lines: Vec<String> = Vec::with_capacity(height);

    let mut star_line: String = (0..width).map(|c| stars.get(0, c)).collect();
    for (col, cloud) in &cloud_positions {
        if *col + cloud.chars().count() <= width {
            let before: String = star_line.chars().take(*col).collect();
            let after: String = star_line
                .chars()
                .skip(*col + cloud.chars().count())
                .collect();
            star_line = format!("{}{}{}", before, cloud, after);
        }
    }
    lines.push(star_line);

    lines.push(" ".repeat(width));

    let mut animal_line = vec![' '; width];
    let trail_chars = ['✦', '⭐', '·', '•', '∘', '·', ' ', ' '];
    for (i, &ch) in trail_chars.iter().enumerate() {
        let trail_pos = bull_pos + 2 + i;
        if trail_pos < width && ch != ' ' {
            animal_line[trail_pos] = ch;
        }
    }

    let mut animal_str = String::new();
    for (i, &ch) in animal_line.iter().enumerate() {
        if i == bull_pos {
            animal_str.push_str(BULL);
        } else if i != bull_pos {
            animal_str.push(ch);
        }
    }
    while animal_str.chars().count() < width {
        animal_str.push(' ');
    }
    lines.push(animal_str);

    let mut dust_line = vec![' '; width];
    let dust_positions = [
        (bull_pos + 2, 0),
        (bull_pos + 3, 1),
        (bull_pos + 4, 2),
        (bull_pos + 5, 3),
    ];
    for (pos, dust_idx) in dust_positions {
        if pos < width && (frame + pos) % 3 != 0 {
            dust_line[pos] = DUST[dust_idx % DUST.len()];
        }
    }
    lines.push(dust_line.into_iter().collect());

    let ground_line: String = (0..width).map(|c| ground.get(0, c)).collect();
    lines.push(ground_line);

    lines
}

fn render_bear(frame: usize, width: usize) -> Vec<String> {
    let height = 5;
    let progress = (frame % ANIMATION_FRAMES) as f32 / (ANIMATION_FRAMES - 1) as f32;
    let max_pos = width.saturating_sub(4);
    let bear_pos = (progress * max_pos as f32) as usize;

    let stars = render_starfield(frame, width, true);
    let (_, cloud_positions) = render_clouds(frame, width, true);
    let ground = render_ground(frame, width, false);

    let mut lines: Vec<String> = Vec::with_capacity(height);

    let mut star_line: String = (0..width).map(|c| stars.get(0, c)).collect();
    for (col, cloud) in &cloud_positions {
        if *col + cloud.chars().count() <= width {
            let before: String = star_line.chars().take(*col).collect();
            let after: String = star_line
                .chars()
                .skip(*col + cloud.chars().count())
                .collect();
            star_line = format!("{}{}{}", before, cloud, after);
        }
    }
    lines.push(star_line);

    lines.push(" ".repeat(width));

    let mut animal_line = vec![' '; width];
    let trail_chars = [' ', ' ', '·', '∘', '•', '·', '⭐', '✦'];
    for (i, &ch) in trail_chars.iter().enumerate() {
        let trail_pos = bear_pos.saturating_sub(i + 1);
        if trail_pos < bear_pos && ch != ' ' {
            animal_line[trail_pos] = ch;
        }
    }

    let mut animal_str = String::new();
    for (i, &ch) in animal_line.iter().enumerate() {
        if i == bear_pos {
            animal_str.push_str(BEAR);
        } else if i != bear_pos {
            animal_str.push(ch);
        }
    }
    while animal_str.chars().count() < width {
        animal_str.push(' ');
    }
    lines.push(animal_str);

    let mut dust_line = vec![' '; width];
    let dust_positions = [
        (bear_pos.saturating_sub(2), 0),
        (bear_pos.saturating_sub(3), 1),
        (bear_pos.saturating_sub(4), 2),
        (bear_pos.saturating_sub(5), 3),
    ];
    for (pos, dust_idx) in dust_positions {
        if pos < bear_pos && (frame + pos) % 3 != 0 {
            dust_line[pos] = DUST[dust_idx % DUST.len()];
        }
    }
    lines.push(dust_line.into_iter().collect());

    let ground_line: String = (0..width).map(|c| ground.get(0, c)).collect();
    lines.push(ground_line);

    lines
}

pub fn get_mascot_lines(success: bool, frame: usize, canvas_width: usize) -> Vec<String> {
    if success {
        render_bull(frame, canvas_width)
    } else {
        render_bear(frame, canvas_width)
    }
}

pub fn mascot_height() -> u16 {
    5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bull_moves_left() {
        let frame0 = render_bull(0, 60);
        let frame31 = render_bull(31, 60);

        let bull_pos_0 = frame0[2].find(BULL).unwrap_or(60);
        let bull_pos_31 = frame31[2].find(BULL).unwrap_or(0);

        assert!(bull_pos_31 < bull_pos_0);
    }

    #[test]
    fn test_bear_moves_right() {
        let frame0 = render_bear(0, 60);
        let frame31 = render_bear(31, 60);

        let bear_pos_0 = frame0[2].find(BEAR).unwrap_or(0);
        let bear_pos_31 = frame31[2].find(BEAR).unwrap_or(60);

        assert!(bear_pos_31 > bear_pos_0);
    }

    #[test]
    fn test_animation_height() {
        let lines = render_bull(0, 60);
        assert_eq!(lines.len(), 5);
        assert_eq!(mascot_height(), 5);
    }

    #[test]
    fn test_has_stars() {
        let lines = render_bull(0, 80);
        let star_line = &lines[0];
        let has_star = star_line.chars().any(|c| STARS.contains(&c) || c == '☁');
        assert!(has_star || star_line.contains("░"));
    }

    #[test]
    fn test_has_ground() {
        let lines = render_bull(0, 60);
        let ground_line = &lines[4];
        let has_ground = ground_line
            .chars()
            .any(|c| GROUND_CHARS.contains(&c) || GRASS.contains(&c));
        assert!(has_ground);
    }

    #[test]
    fn test_animation_cycles() {
        let frame0 = render_bull(0, 60);
        let frame32 = render_bull(32, 60);

        let pos0 = frame0[2].find(BULL);
        let pos32 = frame32[2].find(BULL);
        assert_eq!(pos0, pos32);
    }

    #[test]
    fn test_mascot_selection() {
        let bull_lines = get_mascot_lines(true, 5, 60);
        let bear_lines = get_mascot_lines(false, 5, 60);

        assert!(bull_lines[2].contains(BULL));
        assert!(bear_lines[2].contains(BEAR));
    }
}
