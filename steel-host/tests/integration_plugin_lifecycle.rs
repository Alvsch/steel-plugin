use anyhow::Context;
use steel_host::{PluginHost, discover_plugins};

mod fixtures;

#[tokio::test]
async fn discover_orders_provider_before_consumer() -> anyhow::Result<()> {
    let fixture = fixtures::setup_layout().await?;

    let discovered = discover_plugins(&fixture.plugin).await?;
    let names: Vec<String> = discovered.into_iter().map(|(meta, _)| meta.name).collect();

    let provider_index = names
        .iter()
        .position(|name| name == "provider-plugin")
        .context("provider-plugin was not discovered")?;
    let consumer_index = names
        .iter()
        .position(|name| name == "consumer-plugin")
        .context("consumer-plugin was not discovered")?;

    assert!(
        provider_index < consumer_index,
        "provider-plugin should be discovered before consumer-plugin; order={names:?}"
    );
    assert!(
        names.iter().any(|name| name == "listening-plugin"),
        "listening-plugin should be discovered; order={names:?}"
    );

    Ok(())
}

#[tokio::test]
async fn lifecycle_load_enable_disable_all_fixtures() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let fixture = fixtures::setup_layout().await?;
    let discovered = discover_plugins(&fixture.plugin).await?;

    assert!(
        !discovered.is_empty(),
        "expected at least one plugin fixture to be discovered",
    );

    let host = PluginHost::new(fixtures::host_config(), fixture.data.clone())
        .map_err(|err| anyhow::anyhow!("failed to construct PluginHost: {err}"))?;

    let plugin_names: Vec<String> = discovered
        .iter()
        .map(|(meta, _)| meta.name.clone())
        .collect();
    let mut enabled_plugins = Vec::new();

    for (plugin_meta, file_path) in discovered {
        let name = plugin_meta.name.clone();
        let plugin = host
            .prepare_plugin(plugin_meta, &file_path)
            .await
            .context("failed to prepare plugin")?;
        host.load_plugin(&plugin)
            .await
            .context("failed to load plugin")?;
        host.enable_plugin(&plugin)
            .await
            .with_context(|| format!("failed to enable {name}"))?;
        enabled_plugins.push(plugin);
    }

    for plugin_name in &plugin_names {
        assert!(
            host.state.resolve_plugin(plugin_name).is_some(),
            "plugin '{plugin_name}' should be registered after load/enable"
        );
    }

    // host.state
    //     .handler_registry
    //     .write()
    //     .await
    //     .dispatch_topic(FakeEvent)
    //     .await
    //     .expect("failed to dispatch event");

    while let Some(plugin) = enabled_plugins.pop() {
        host.disable_plugin(&plugin)
            .await
            .context("failed to disable plugin")?;
    }

    for plugin_name in &plugin_names {
        assert!(
            host.state.resolve_plugin(plugin_name).is_none(),
            "plugin '{plugin_name}' should be unregistered after disable"
        );
    }

    Ok(())
}
