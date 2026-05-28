// use anyhow::Context;
// use steel_host::{PluginHost, discover_plugins};
// use steel_plugin_core::PluginMeta;

// mod fixtures;

// #[tokio::test]
// async fn consumer_provider_rpc_roundtrip() -> anyhow::Result<()> {
//     let fixture = fixtures::setup_layout().await?;

//     let discovered = discover_plugins(&fixture.plugin)
//         .await?
//         .into_iter()
//         .filter(|plugin| matches!(plugin.name.as_str(), "consumer-plugin" | "provider-plugin"))
//         .collect::<Vec<PluginMeta>>();

//     let host = PluginHost::new(fixtures::host_config(), fixture.data.clone())
//         .map_err(|err| anyhow::anyhow!("failed to construct PluginHost: {err}"))?;

//     let plugin_names: Vec<String> = discovered.iter().map(|meta| meta.name.clone()).collect();
//     let mut enabled_plugins = Vec::new();

//     for plugin_meta in discovered {
//         let plugin = host
//             .prepare_plugin(plugin_meta)
//             .await
//             .context("failed to prepare plugin")?;
//         host.load_plugin(&plugin)
//             .await
//             .context("failed to load plugin")?;
//         host.enable_plugin(&plugin)
//             .await
//             .context("failed to enable plugin")?;
//         enabled_plugins.push(plugin);
//     }

//     Ok(())
// }
