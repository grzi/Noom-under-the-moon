pub fn is_plasma_door_part(sprite_nb: usize) -> bool {
    PLASMA_SPRITES.contains(&sprite_nb)
}

pub fn plasma_door_next_sprite(sprite_nb: usize) -> usize {
    match sprite_nb {
        HORIZONTAL_PLASMA_0_A => HORIZONTAL_PLASMA_0_B,
        HORIZONTAL_PLASMA_0_B => HORIZONTAL_PLASMA_0_A,
        HORIZONTAL_PLASMA_1_A => HORIZONTAL_PLASMA_1_B,
        HORIZONTAL_PLASMA_1_B => HORIZONTAL_PLASMA_1_A,
        HORIZONTAL_PLASMA_2_A => HORIZONTAL_PLASMA_2_B,
        HORIZONTAL_PLASMA_2_B => HORIZONTAL_PLASMA_2_A,
        HORIZONTAL_PLASMA_3_A => HORIZONTAL_PLASMA_3_B,
        HORIZONTAL_PLASMA_3_B => HORIZONTAL_PLASMA_3_A,
        _ => 0
    }
}

pub fn plasma_door_close_sprite(sprite_nb: usize) -> usize {
    match sprite_nb {
        HORIZONTAL_PLASMA_0_A => CLOSED_PLASMA_LEFT,
        HORIZONTAL_PLASMA_0_B => CLOSED_PLASMA_LEFT,
        HORIZONTAL_PLASMA_1_A => EMPTY,
        HORIZONTAL_PLASMA_1_B => EMPTY,
        HORIZONTAL_PLASMA_2_A => EMPTY,
        HORIZONTAL_PLASMA_2_B => EMPTY,
        HORIZONTAL_PLASMA_3_A => CLOSED_PLASMA_RIGHT,
        HORIZONTAL_PLASMA_3_B => CLOSED_PLASMA_RIGHT,
        _ => 0
    }
}


pub const CLOSED_PLASMA_LEFT: usize = 144;
pub const CLOSED_PLASMA_RIGHT: usize = 154;
pub const CLOSED_PLASMA_TOP: usize = 134;
pub const CLOSED_PLASMA_BOTTOM: usize = 164;

pub const HORIZONTAL_PLASMA_0_A: usize = 114;
pub const HORIZONTAL_PLASMA_0_B: usize = 124;
pub const HORIZONTAL_PLASMA_1_A: usize = 115;
pub const HORIZONTAL_PLASMA_1_B: usize = 125;
pub const HORIZONTAL_PLASMA_2_A: usize = 116;
pub const HORIZONTAL_PLASMA_2_B: usize = 126;
pub const HORIZONTAL_PLASMA_3_A: usize = 117;
pub const HORIZONTAL_PLASMA_3_B: usize = 127;

const PLASMA_SPRITES: [usize; 12] = [CLOSED_PLASMA_LEFT, CLOSED_PLASMA_RIGHT, CLOSED_PLASMA_TOP, CLOSED_PLASMA_BOTTOM, HORIZONTAL_PLASMA_0_A, HORIZONTAL_PLASMA_0_B, HORIZONTAL_PLASMA_1_A, HORIZONTAL_PLASMA_1_B, HORIZONTAL_PLASMA_2_A, HORIZONTAL_PLASMA_2_B, HORIZONTAL_PLASMA_3_A, HORIZONTAL_PLASMA_3_B];
pub const EMPTY: usize = 39;



