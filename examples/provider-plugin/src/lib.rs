use steel_plugin_sdk::{
    Plugin, info, rpc_export,
    serde::{Deserialize, Serialize},
};

pub struct ProviderPlugin;

#[derive(Deserialize)]
pub struct GreetRequest {
    name: String,
}

#[derive(Serialize)]
pub struct GreetResponse {
    message: String,
}

#[derive(Deserialize)]
pub struct NotifyRequest {
    message: String,
}

impl Plugin for ProviderPlugin {
    fn on_enable() {
        info!("hello from the provider!");
    }

    fn on_disable() {
        info!("provider disabled");
    }
}

#[rpc_export]
fn greet(request: GreetRequest) -> GreetResponse {
    GreetResponse {
        message: format!("hello {}", request.name),
    }
}

#[rpc_export]
fn notify(request: NotifyRequest) {
    info!("{}", request.message);
}

#[rpc_export]
fn echo(data: &[u8]) -> Option<Vec<u8>> {
    Some(data.to_vec())
}

steel_plugin_sdk::plugin_export!(ProviderPlugin);
