#!/bin/bash

# 獲取腳本所在目錄的絕對路徑
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# 更新 Anchor
echo "Updating Anchor..."
cargo install --git https://github.com/coral-xyz/anchor avm --locked
avm install latest
avm use latest

# 切換到項目根目錄
cd "$SCRIPT_DIR"

# 清理項目
echo "Cleaning project..."
anchor clean

# 重新構建項目
echo "Rebuilding project..."
anchor build

echo "Done. If you still encounter issues, please refer to the troubleshooting section in the README.md file."