#[derive(Debug, Clone, Copy, Default)]
pub enum SameSite {
    Strict,
    Lax,
    #[default]
    None,
}
