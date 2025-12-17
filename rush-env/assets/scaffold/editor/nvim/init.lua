-- init.lua

-- === 1. 兼容旧 Vim 插件 ===
vim.opt.runtimepath:prepend(vim.fn.expand("~/.vim"))
vim.opt.runtimepath:append(vim.fn.expand("~/.vim/after"))
vim.opt.packpath = vim.o.runtimepath

vim.cmd("source ~/.vimrc") -- 如果需要保留旧配置，可以留着，否则建议注释掉

-- === 2. 光标样式设置 ===
vim.opt.guicursor = {
    "n:block-nCursor-blinkon1-blinkwait1-blinkoff1",
    "i:ver25-blinkon1-blinkwait1-blinkoff1",
    "r:hor100-blinkon1-blinkwait1-blinkoff1"
}

-- === 3. 自动命令组 ===
local manual = vim.api.nvim_create_augroup("manual", { clear = true })

-- 自动跳到上次光标位置
vim.api.nvim_create_autocmd("BufReadPost", {
    group = manual,
    callback = function()
        local line = vim.fn.line
        if line("'\"") > 1 and line("'\"") <= line("$") then
            vim.cmd([[normal! g'"]])
        end
    end
})

-- VimEnter 自动打开 NERDTree 并切回原窗口
vim.api.nvim_create_autocmd("VimEnter", {
    group = manual,
    command = "NERDTreeToggle | wincmd p"
})

-- 如果只剩下 NERDTree，则退出
local function close_if_only_nerdtree()
    local bufnr = vim.api.nvim_get_current_buf()
    local bufname = vim.api.nvim_buf_get_name(bufnr)
    local win_count = vim.fn.winnr('$')
    local tab_count = vim.fn.tabpagenr('$')

    if tab_count == 1 and win_count == 1 and bufname:find("NERD_tree_") then
        vim.opt.guicursor = "a:ver25-blinkon1-blinkwait1-blinkoff1"
        vim.cmd("quit")
    end
end

vim.api.nvim_create_autocmd("BufEnter", {
    group = manual,
    callback = close_if_only_nerdtree
})

vim.api.nvim_create_autocmd("TabNew", {
    group = manual,
    callback = function()
        if vim.b.NERDTree then
            vim.cmd("NERDTreeToggle | wincmd p")
        end
    end
})

-- VimLeave 恢复光标样式
vim.api.nvim_create_autocmd("VimLeave", {
    group = manual,
    command = "set guicursor=a:ver25-blinkon1-blinkwait1-blinkoff1"
})

-- === 4. 缩进设置 ===
vim.opt.expandtab = true
vim.opt.shiftwidth = 4
vim.opt.tabstop = 4
vim.opt.softtabstop = 4

-- === 5. indent-blankline 彩色缩进线 ===
vim.defer_fn(function()
    local highlight = {
        "RainbowRed",
        "RainbowYellow",
        "RainbowBlue",
        "RainbowOrange",
        "RainbowGreen",
        "RainbowViolet",
        "RainbowCyan",
    }

    local hooks = require("ibl.hooks")
    hooks.register(hooks.type.HIGHLIGHT_SETUP, function()
        vim.api.nvim_set_hl(0, "RainbowRed",    { fg = "#E06C75" })
        vim.api.nvim_set_hl(0, "RainbowYellow", { fg = "#E5C07B" })
        vim.api.nvim_set_hl(0, "RainbowBlue",   { fg = "#61AFEF" })
        vim.api.nvim_set_hl(0, "RainbowOrange", { fg = "#D19A66" })
        vim.api.nvim_set_hl(0, "RainbowGreen",  { fg = "#98C379" })
        vim.api.nvim_set_hl(0, "RainbowViolet", { fg = "#C678DD" })
        vim.api.nvim_set_hl(0, "RainbowCyan",   { fg = "#56B6C2" })
    end)

    require("ibl").setup {
        indent = { highlight = highlight },
    }
end, 0)
