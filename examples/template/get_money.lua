--!strict
local money = {
	doug = 432,
	fred = 999,
	steve = 727,
} :: { [string]: number }

return function(id)
	local money = money[id]
	if money == nil then
		return 0
	end
	return money
end
