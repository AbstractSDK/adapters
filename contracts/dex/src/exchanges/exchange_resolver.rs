use crate::dex_trait::Identify;
use crate::error::DexError;
use crate::DEX;
use cosmwasm_std::Addr;

// Supported exchanges on Juno
#[cfg(feature = "juno")]
pub use crate::exchanges::{
    junoswap::{JunoSwap, JUNOSWAP},
    wyndex::{WynDex, WYNDEX},
};

#[cfg(feature = "terra")]
pub use crate::exchanges::terraswap::{Terraswap, TERRASWAP};

#[cfg(feature = "terra")]
pub use crate::exchanges::astroport::{Astroport, ASTROPORT};

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub use crate::exchanges::osmosis::{Osmosis, OSMOSIS};

/// Used to map a string to a DEX without requiring the DEX to be deployed locally.
pub(crate) fn identify_exchange(value: &str) -> Result<&'static dyn Identify, DexError> {
    match value {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(&JunoSwap {}),
        #[cfg(feature = "juno")]
        WYNDEX => Ok(&WynDex {}),
        #[cfg(feature = "juno")]
        OSMOSIS => Ok(&Osmosis {
            local_proxy_addr: None,
        }),
        #[cfg(feature = "terra")]
        TERRASWAP => Ok(&Terraswap {}),
        #[cfg(feature = "terra")]
        ASTROPORT => Ok(&Astroport {}),
        _ => Err(DexError::UnknownDex(value.to_owned())),
    }
}

/// Used to map a string to a DEX that is locally deployed.
pub(crate) fn resolve_exchange(
    value: &str,
    proxy_addr: Option<&Addr>,
) -> Result<Box<dyn DEX>, DexError> {
    match value {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(Box::new(JunoSwap {})),
        #[cfg(feature = "juno")]
        WYNDEX => Ok(Box::new(WynDex {})),
        #[cfg(feature = "osmosis")]
        OSMOSIS => Ok(Box::new(Osmosis {
            local_proxy_addr: proxy_addr.cloned(),
        })),
        #[cfg(feature = "terra")]
        TERRASWAP => Ok(Box::new(Terraswap {})),
        #[cfg(feature = "terra")]
        ASTROPORT => Ok(Box::new(Astroport {})),
        _ => Err(DexError::ForeignDex(value.to_owned())),
    }
}
