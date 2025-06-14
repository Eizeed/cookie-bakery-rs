#[derive(Debug, Clone, Default)]
pub enum SameSite {
    Strict,
    Lax,
    #[default]
    None,
}
