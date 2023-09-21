#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
use wasm4::*;

#[rustfmt::skip]
const SMILEY: [u8; 8] = [
    0b11000011,
    0b10000001,
    0b00100100,
    0b00100100,
    0b00000000,
    0b00100100,
    0b10011001,
    0b11000011,
];

const FONT_WIDTH: u32 = 208;
const FONT_FLAGS: u32 = BLIT_1BPP;
const CHAR_WIDTH: u32 = 8;
const CHAR_HEIGHT: u32 = 8;
const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const FONT: &'static [u8] = &[];

#[no_mangle]
fn update() {
    unsafe { *DRAW_COLORS = 2 }
    text("Hello from Rust!", 10, 10);

    let gamepad = unsafe { *GAMEPAD1 };
    if gamepad & BUTTON_1 != 0 {
        unsafe { *DRAW_COLORS = 4 }
    }

    blit(&SMILEY, 76, 76, 8, 8, BLIT_1BPP);
    text("Press X to blink", 16, 90);

    let mouse = unsafe { *MOUSE_BUTTONS };
    let mouse_x = unsafe { *MOUSE_X };
    let mouse_y = unsafe { *MOUSE_Y };

    if mouse & MOUSE_LEFT != 0 {
        unsafe { *DRAW_COLORS = 4 }
        rect(i32::from(mouse_x) - 8, i32::from(mouse_y) - 8, 16, 16);
    } else {
        unsafe { *DRAW_COLORS = 2 }
        rect(i32::from(mouse_x) - 4, i32::from(mouse_y) - 4, 8, 8);
    }

    write("HELLO WORLD WITH\nOUR CUSTOM FONT", 4, 4, 0x30);

    pixel(16, 90);

    let game_data = unsafe {
        let mut buffer = [0u8; core::mem::size_of::<i32>()];
    
        diskr(buffer.as_mut_ptr(), buffer.len() as u32);
    
        i32::from_le_bytes(buffer)
    };

    unsafe {
        let game_data_bytes = game_data.to_le_bytes();
        diskw(game_data_bytes.as_ptr(), core::mem::size_of::<i32>() as u32);
    }
}

fn pixel(x: i32, y: i32) {
    // The byte index into the framebuffer that contains (x, y)
    let idx = (y as usize * 160 + x as usize) >> 2;

    // Calculate the bits within the byte that corresponds to our position
    let shift = (x as u8 & 0b11) << 1;
    let mask = 0b11 << shift;

    unsafe {
        let palette_color: u8 = (*DRAW_COLORS & 0xf) as u8;
        if palette_color == 0 {
            // Transparent
            return;
        }
        let color = (palette_color - 1) & 0b11;

        let framebuffer = &mut *FRAMEBUFFER;

        framebuffer[idx] = (color << shift) | (framebuffer[idx] & !mask);
    }
}

fn draw_space(x: i32, y: i32, column: u32, line: u32, colors: u16) {
    unsafe { *DRAW_COLORS = *DRAW_COLORS & 0x0F }
    rect(
        x + (column * CHAR_WIDTH) as i32,
        y + (line * CHAR_HEIGHT) as i32,
        CHAR_WIDTH,
        CHAR_HEIGHT
    );
    unsafe { *DRAW_COLORS = colors }
}

pub fn write(text: &str, x: i32, y: i32, colors: u16) {
    unsafe { *DRAW_COLORS = colors }

    let mut line: u32 = 0;
    let mut column: u32 = 0;

    for c in text.chars() {
        let char_code = c as u32;

        if char_code == 10 {
            line += 1;
            column = 0;
            continue;
        }

        let char_index: u32;

        match CHARSET.find(c) {
            Some(x) => char_index = x as u32,

            None => {
                draw_space(x, y, column, line, colors);
                column += 1;
                continue;
            }
        }

        blit_sub(
            FONT,
            x + (column * CHAR_WIDTH) as i32,
            y + (line * CHAR_HEIGHT) as i32,
            CHAR_WIDTH,
            CHAR_HEIGHT,
            char_index * CHAR_WIDTH,
            0,
            FONT_WIDTH,
            FONT_FLAGS
        );
        column += 1;
    }
}