# Install (5 lines)

```bash
# 1. Install rust + tree-sitter
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
sudo apt install -y libtree-sitter-dev   # Debian/Ubuntu; or `brew install tree-sitter` on macOS

# 2. Build
cargo build --release

# 3. Verify
cargo test                              # 24/24 PASS
node test/smoke.js                      # 76/77 PASS
node test/corpus-regression.js          # 91/100 PASS

# 4. (Optional) Install globally
cargo install --path .
```