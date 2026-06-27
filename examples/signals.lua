--!strict
local plugin: Plugin = {
	name = "signals",
	description = "",
	author = "Alvsch",
	version = "0.1.0",
	api_version = "0.1.0",
	on_enable = function()
		signal:Connect(function(v: string)
			info(v)
		end)
	end,
	on_disable = function() end,
}

return plugin
