use steel_plugin_sdk::{
    Plugin, info, rpc,
    serde::{Deserialize, Serialize},
};

pub struct ConsumerPlugin;

#[derive(Serialize)]
struct GreetRequest {
    name: String,
}

#[derive(Deserialize)]
struct GreetResponse {
    message: String,
}

#[derive(Serialize)]
struct NotifyRequest {
    message: String,
}

impl Plugin for ConsumerPlugin {
    fn on_enable() {
        info!("hello from the consumer!");
        let response: GreetResponse = rpc::request(
            "provider-plugin",
            "greet",
            &GreetRequest {
                name: "Steve".to_owned(),
            },
        )
        .expect("provider greet call should succeed");
        assert_eq!(response.message, "hello Steve");

        rpc::notify(
            "provider-plugin",
            "notify",
            &NotifyRequest {
                message: "one-way call".to_owned(),
            },
        )
        .expect("provider notify call should succeed");

        let error = rpc::call("provider-plugin", "greet", b"\xc1")
            .expect_err("invalid MessagePack request should be rejected");
        assert!(matches!(error, rpc::CallError::InvalidRequest));

        let error = rpc::call("provider-plugin", "missing", b"")
            .expect_err("unknown provider method should return an error");
        assert!(matches!(error, rpc::CallError::UnknownMethod));

        let raw_response = rpc::call("provider-plugin", "echo", b"raw bytes")
            .expect("raw RPC should succeed")
            .expect("echo returns bytes");
        assert_eq!(raw_response, b"raw bytes");
    }

    fn on_disable() {
        info!("consumer disabled");
    }
}

steel_plugin_sdk::plugin_export!(ConsumerPlugin, {
    depends: ["provider-plugin"]
});
