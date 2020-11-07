/// 显示宽度 64 像素
pub const DISPLAY_WIDTH: usize = 64;
/// 显示高度 32 像素
pub const DISPLAY_HEIGHT: usize = 32;

#[derive(Clone)]
pub struct Display {
    pub gfx: [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    pub dirty: bool,
}

impl Display {
    pub fn new() -> Display {
        Display {
            gfx: [[0u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            dirty: true,
        }
    }

    pub fn clear(&mut self) {
        self.gfx = [[0u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
        self.dirty = true;
    }

    pub fn draw(&mut self, xpos: usize, ypos: usize, sprite: &[u8]) -> bool {
        let mut collision = false;
        let h = sprite.len();

        for j in 0..h {
            for i in 0..8 {
                let y = (ypos + j) % DISPLAY_HEIGHT;
                let x = (xpos + i) % DISPLAY_WIDTH;

                if (sprite[j] & (0x80 >> i)) != 0x00 {
                    if self.gfx[y][x] == 0x01 {
                        collision = true;
                    }
                    self.gfx[y][x] ^= 0x01;
                }
            }
        }
        self.dirty = true;

        collision
    }
}

pub static FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
