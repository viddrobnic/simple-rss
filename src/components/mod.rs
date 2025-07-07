mod content;
mod help;
mod item_list;
mod toast;

pub use content::Content;
pub use help::Help;
pub use item_list::ItemList;
pub use toast::Toast;

const SPINNER_FRAMES: [u32; 10] = [
    0x280B, // ⠋
    0x2819, // ⠙
    0x2839, // ⠹
    0x2838, // ⠸
    0x283C, // ⠼
    0x2834, // ⠴
    0x2826, // ⠦
    0x2827, // ⠧
    0x2807, // ⠇
    0x280F, // ⠏
];

fn spinner_frame(tick: usize) -> char {
    let ch = SPINNER_FRAMES[(tick / 3) % SPINNER_FRAMES.len()];
    // Safe because chars are hardcoded
    unsafe { char::from_u32_unchecked(ch) }
}
