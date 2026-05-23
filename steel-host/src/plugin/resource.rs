use wasmtime_wasi::ResourceTable;

pub struct PluginResources {
    pub table: ResourceTable,
}

impl Default for PluginResources {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginResources {
    #[must_use]
    pub fn new() -> Self {
        Self {
            table: ResourceTable::new(),
        }
    }
}
