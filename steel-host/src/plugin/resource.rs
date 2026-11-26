use wasmtime_wasi::ResourceTable;

pub(crate) struct PluginResources {
    pub(crate) table: ResourceTable,
}

impl Default for PluginResources {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginResources {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            table: ResourceTable::new(),
        }
    }
}
