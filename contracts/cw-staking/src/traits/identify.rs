pub type ProviderName = &'static str;
/// Identify a staking provider by its name
pub trait Identify {
    fn name(&self) -> ProviderName;
}
