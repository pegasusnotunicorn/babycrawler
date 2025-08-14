pub const DEBUG_MODE: bool = false;

pub const GAME_PADDING: u32 = 16;

pub const MAP_SIZE: usize = 5;
pub const HAND_SIZE: usize = 4;

// UI
pub const FONT_HEIGHT: u32 = 12;

// Entrance count weights for tile random generation
pub const ENTRANCE_COUNT_WEIGHT_1: f32 = 1.0;
pub const ENTRANCE_COUNT_WEIGHT_2: f32 = 2.0;
pub const ENTRANCE_COUNT_WEIGHT_3: f32 = 2.0;
pub const ENTRANCE_COUNT_WEIGHT_4: f32 = 0.25;
// pub const ENTRANCE_COUNT_WEIGHT_1: f32 = 1.0;
// pub const ENTRANCE_COUNT_WEIGHT_2: f32 = 0.0;
// pub const ENTRANCE_COUNT_WEIGHT_3: f32 = 0.0;
// pub const ENTRANCE_COUNT_WEIGHT_4: f32 = 0.0;

// colors
pub const GAME_BG_COLOR: u32 = 0x222222ff; // Dark gray
pub const PLAYER_1_COLOR: u32 = 0xaa0000ff; // Red
pub const PLAYER_2_COLOR: u32 = 0x0000aaff; // Blue

// card colors
pub const CARD_DUMMY_COLOR: u32 = 0xffffff25;
pub const CARD_FIRE_COLOR: u32 = 0x8b0000ff;
pub const CARD_SWAP_COLOR: u32 = 0xb804b8ff;
pub const CARD_MOVE_COLOR: u32 = 0x0000aaff;
pub const CARD_ROTATE_COLOR: u32 = 0x00aa00ff;
pub const CARD_HOVER_OUTLINE_COLOR: u32 = 0xffffffaa;
pub const CARD_BUTTON_A_COLOR: u32 = 0x118811ff;
pub const CARD_BUTTON_B_COLOR: u32 = 0xff2222ff;

// frame
pub const FLASH_SPEED: f64 = 0.1; // smaller = slower
pub const GAME_CHANNEL: &str = "GLOBAL";

// damage
pub const PLAYER_HEALTH: u32 = 3;
pub const FIREBALL_DAMAGE: u32 = 1;
