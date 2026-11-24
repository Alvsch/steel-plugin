wit_bindgen::generate!({
    path: "../wit",
    world: "plugin-world",
    pub_export_macro: true,
    default_bindings_module: "steel_plugin_sdk::__export",
});
