--!strict
local plugin: Plugin = {
	name = "Template Plugin",
	description = "A template plugin",
	version = "0.1.0",
	author = "Alvsch",
	on_enable = function()
		log.Info("Enabled")
	end,
	on_disable = function()
		log.Info("Disabled")
	end,
}

return plugin