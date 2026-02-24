use std::fmt::{self, Debug};

use self_cell::self_cell;

use crate::PluginMeta;

self_cell!(
    pub struct PluginContainer {
        owner: Vec<u8>,
        #[covariant]
        dependent: PluginMeta,
    }
);

impl Debug for PluginContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dependant = self.borrow_dependent();
        f.debug_struct("PluginContainer")
            .field("dependant", dependant)
            .finish()
    }
}
