use bon::Builder;
use serde::{Deserialize, Serialize};

pub mod dll;
pub mod inject;

/**
<div class="warning">

The injector and the DLL MUST use the same crate features.
Otherwise, deserialization may result in corrupted config or fail.
</div>
*/
#[derive(Default, Serialize, Deserialize, Clone, Builder, Debug)]
pub struct HookConfig {
    #[cfg(feature = "item")]
    item: crate::item::hook::HookConfig,
}

/// Set the hook with optional config.
/// If config is None, the hook is disabled.
pub fn set_hook(config: Option<HookConfig>) {
    #[cfg(feature = "item")]
    crate::item::hook::set_hook(config.map(|c| c.item));
}
