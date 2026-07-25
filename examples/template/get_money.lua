--!strict
local money = require("./money")

return function(id)
	local money = money[id]
	if money == nil then
		return 0
	end
	return money
end
