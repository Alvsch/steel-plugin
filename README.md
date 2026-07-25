# Steel Plugin

- [ ] Scheduler - Roblox task library + RunService integration
- [x] Signal - fire-and-forget messages
- [x] Function - request/response calls
- [ ] Permission table
- [x] DataStore - persistent storage
- [x] Require - custom module requiring

## How It Works

Steel Plugin scans a folder for plugin directories with a valid `config.toml`, checks their API and dependency versions, builds the Lua file table, compiles `init.lua`, and then calls each plugin's `on_enable` and `on_disable` hooks during startup and shutdown.

## Making A Plugin

Create a new folder with a `config.toml` and an `init.lua`. The config defines the plugin name, version, API version, authors, optional exports, and optional dependencies. Your `init.lua` should return a `Plugin` table with `on_enable` and `on_disable` functions. You can add extra `.lua` or `.luau` files alongside it and require them from the plugin.

