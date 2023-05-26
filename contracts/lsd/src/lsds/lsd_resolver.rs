use crate::error::LsdError;
use crate::traits::{command::LsdCommand, identity::Identify};

// Supported exchanges on Juno
#[cfg(feature = "juno")]
pub use crate::lsds::{
    junoswap::{JunoSwap, JUNOSWAP},
    wyndex::{WynDex, WYNDEX},
};

#[cfg(feature = "terra")]
pub use crate::exchanges::terraswap::{Terraswap, TERRASWAP};

#[cfg(feature = "terra")]
pub use crate::exchanges::astroport::{Astroport, ASTROPORT};

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub use crate::lsds::osmosis::{Osmosis, OSMOSIS};

pub(crate) fn identify_lsd(value: &str) -> Result<&'static dyn Identify, LsdError> {
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
        _ => Err(LsdError::UnknownDex(value.to_owned())),
    }
}

pub(crate) fn resolve_lsd(value: &str) -> Result<&'static dyn LsdCommand, LsdError> {
    match value {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(&JunoSwap {}),
        #[cfg(feature = "juno")]
        WYNDEX => Ok(&WynDex {}),
        // #[cfg(feature = "osmosis")]
        // OSMOSIS => Ok(&Osmosis {
        //     local_proxy_addr: None,
        // }),
        #[cfg(feature = "terra")]
        TERRASWAP => Ok(&Terraswap {}),
        #[cfg(feature = "terra")]
        ASTROPORT => Ok(&Astroport {}),
        _ => Err(LsdError::ForeignDex(value.to_owned())),
    }
}
