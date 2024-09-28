#!/bin/bash

set -e
set -x

echo "Script is running..."

# 設置正確的項目根目錄
PROJECT_ROOT="/home/dc/flexx_bnpl/flexxcash_bnpl"

# 切換到項目根目錄
cd "$PROJECT_ROOT"

echo "Current working directory: $(pwd)"
echo "Contents of current directory:"
ls -la

# 檢查 Anchor.toml 是否存在
if [ -f "Anchor.toml" ]; then
    echo "Anchor.toml found"
else
    echo "Anchor.toml not found"
    exit 1
fi

# 顯示 Solana 版本
solana --version

# 顯示 Anchor 版本
anchor --version

# 清理項目
echo "Cleaning project..."
anchor clean

# 重新構建項目
echo "Rebuilding project..."
RUST_BACKTRACE=1 anchor build

echo "Build complete. Checking for the compiled program..."
if [ -f "$PROJECT_ROOT/target/deploy/oracles.so" ]; then
    echo "Program file oracles.so has been successfully created."
else
    echo "Error: Program file oracles.so was not created. Check the build output for errors."
    echo "Listing contents of target/deploy directory:"
    ls -la "$PROJECT_ROOT/target/deploy"
    echo "Checking Anchor.toml content:"
    cat "$PROJECT_ROOT/Anchor.toml"
    echo "Checking Cargo.toml content:"
    cat "$PROJECT_ROOT/programs/oracles/Cargo.toml"
    exit 1
fi

echo "Done. If you still encounter issues, please refer to the troubleshooting section in the README.md file."