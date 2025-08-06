pub const GAME_PADDING: u32 = 16;

pub const MAP_SIZE: usize = 5;
pub const HAND_SIZE: usize = 3;

// Entrance count weights for tile random generation
pub const ENTRANCE_COUNT_WEIGHT_1: f32 = 0.5;
pub const ENTRANCE_COUNT_WEIGHT_2: f32 = 2.0;
pub const ENTRANCE_COUNT_WEIGHT_3: f32 = 3.0;
pub const ENTRANCE_COUNT_WEIGHT_4: f32 = 0.5;

// colors
pub const GAME_BG_COLOR: u32 = 0x222222ff; // Dark gray
pub const PLAYER_1_COLOR: u32 = 0xff0000ff; // Red
pub const PLAYER_2_COLOR: u32 = 0x3366ccff; // Blue
pub const WALL_COLOR: u32 = 0x8b4513ff; // Brown
pub const FLOOR_COLOR: u32 = 0x000000ff; // Black

// card colors
pub const CARD_DUMMY_COLOR: u32 = 0xffffff20; // Light gray
pub const CARD_HOVER_FILL_COLOR: u32 = 0xffffff80;
pub const CARD_BUTTON_A_COLOR: u32 = 0x118811ff;
pub const CARD_BUTTON_B_COLOR: u32 = 0xff2222ff;

// frame
pub const FLASH_SPEED: f64 = 0.1; // smaller = slower
pub const GAME_CHANNEL: &str = "GLOBAL";
