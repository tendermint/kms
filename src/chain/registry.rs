//! Registry of information about known Tendermint blockchain networks

use super::{Chain, Guard, Id};
use crate::{
    error::{Error, ErrorKind::*},
    prelude::*,
};
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, sync::RwLock};

/// State of Tendermint blockchain networks
pub static REGISTRY: Lazy<GlobalRegistry> = Lazy::new(GlobalRegistry::default);

/// Registry of blockchain networks known to the KMS
#[derive(Default)]
pub struct Registry(BTreeMap<Id, Chain>);

impl Registry {
    /// Register a `Chain` with the registry
    pub fn register_chain(&mut self, chain: Chain) -> Result<(), Error> {
        let chain_id = chain.id;

        if self.0.insert(chain_id, chain).is_none() {
            Ok(())
        } else {
            // TODO(tarcieri): handle updating the set of registered chains
            fail!(ConfigError, "chain ID already registered: {}", chain_id);
        }
    }

    /// Get information about a particular chain ID (if registered)
    pub fn get_chain(&self, chain_id: &Id) -> Option<&Chain> {
        self.0.get(chain_id)
    }

    /// Get a mutable reference to the given chain
    pub(crate) fn get_chain_mut(&mut self, chain_id: &Id) -> Result<&mut Chain, Error> {
        self.0.get_mut(chain_id).ok_or_else(|| {
            format_err!(
                InvalidKey,
                "can't add signer to unregistered chain: {}",
                chain_id
            )
            .into()
        })
    }
}

/// Global registry of blockchain networks known to the KMS
// NOTE: The `RwLock` is a bit of futureproofing as this data structure is for the
// most part "immutable". New chains should be registered at boot time.
// The only case in which this structure may change is in the event of
// runtime configuration reloading, so the `RwLock` is included as
// futureproofing for such a feature.
//
// See: <https://github.com/tendermint/kms/issues/183>
#[derive(Default)]
pub struct GlobalRegistry(pub(super) RwLock<Registry>);

impl GlobalRegistry {
    /// Acquire a read-only (concurrent) lock to the internal chain registry
    pub fn get(&self) -> Guard<'_> {
        // TODO(tarcieri): better handle `PoisonError` here?
        self.0.read().unwrap().into()
    }

    /// Register a chain with the registry
    pub fn register(&self, chain: Chain) -> Result<(), Error> {
        // TODO(tarcieri): better handle `PoisonError` here?
        let mut registry = self.0.write().unwrap();
        registry.register_chain(chain)
    }
}
