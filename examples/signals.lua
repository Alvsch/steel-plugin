--!strict
local plugin: Plugin = {
	name = "signals",
	description = "",
	version = "0.1.0",
	author = "Alvsch",
	on_enable = function()
		signal:Connect(function(v: string)
			info(v)
		end)
	end,
	on_disable = function() end,
}

return plugin
