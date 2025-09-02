// ANSI color codes (foreground)
pub const RESET: &str = "\x1b[0m";
pub const CYAN: u8 = 36;
pub const WHITE: u8 = 37;
pub const GREEN: u8 = 32;
pub const BLUE: u8 = 34;
pub const MAGENTA: u8 = 35;
pub const YELLOW: u8 = 33;
pub const BRIGHT_BLACK: u8 = 90;

pub fn bold(s: &str) -> String {
    format!("\x1b[1m{}{}", s, RESET)
}
pub fn color(code: u8, s: &str) -> String {
    format!("\x1b[{}m{}{}", code, s, RESET)
}
pub fn bold_color(code: u8, s: &str) -> String {
    format!("\x1b[1;{}m{}{}", code, s, RESET)
}
