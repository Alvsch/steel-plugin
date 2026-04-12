use steel_plugin_sdk::{info, on_enable, plugin_meta, rpc_export};

plugin_meta!();

#[on_enable]
pub fn on_enable() {
    info!("hello from the provider!");
}

#[rpc_export]
fn get_balance(data: &[u8]) -> Option<Vec<u8>> {
    let msg = str::from_utf8(data).unwrap();
    let result = format!("get_balance: {msg}");
    info!("{result}");

    Some(result.into_bytes())
}
