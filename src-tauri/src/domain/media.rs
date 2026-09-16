/// Catalog/media source. Change 001 registers none; later changes add JW and local adapters.
pub trait MediaProvider: Send + Sync {
    /// Stable provider id used on media handles (`jw-org`, `local`, …).
    fn id(&self) -> &'static str;
}

/// Holds `MediaProvider` implementations. Empty until a later change registers one.
pub struct ProviderRegistry {
    inner: Vec<Box<dyn MediaProvider>>,
}

impl ProviderRegistry {
    /// Empty registry. Playback commands must not special-case provider ids.
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// True when no provider has been registered yet.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Registers a provider. Duplicate ids are rejected by later changes; 001 never inserts.
    pub fn register(&mut self, provider: Box<dyn MediaProvider>) {
        self.inner.push(provider);
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_empty() {
        assert!(ProviderRegistry::new().is_empty());
    }
}
