--!strict
local plugin: Plugin = {
	name = "data-store",
	description = "",
	author = "Alvsch",
	version = "0.1.0",
	api_version = "0.1.0",
	on_enable = function()
        game.Store:SetAsync("id", 100)
        local id = game.Store:GetAsync("id")
        assert(id == 100)
        game.Store:UpdateAsync("id", function(old)
            return old * 2
        end)
        assert(game.Store:RemoveAsync("id") == 200)
        assert(game.Store:GetAsync("id") == nil)
    end,
	on_disable = function() end,
}

return plugin
