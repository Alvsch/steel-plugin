local get_money = require("@template/get_money")

return {
    on_enable = function()
        local who = "doug"
        local money: number = get_money(who)
        info(who.." has $"..money)
    end,
    on_disable = function()
        
    end,
} :: Plugin
