#!/bin/bash
# Build WASM binaries for the NPM package

set -e

echo "🔨 Building LLM Shield WASM..."

# Navigate to the WASM crate
cd "$(dirname "$0")/../../.."
WORKSPACE_ROOT=$(pwd)

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack is not installed"
    echo "Install with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf packages/core/wasm/

# Build for different targets
echo "📦 Building WASM for web target..."
wasm-pack build \
  --target web \
  --out-dir ../../packages/core/wasm/web \
  --scope llm-shield \
  crates/llm-shield-wasm

echo "📦 Building WASM for Node.js target..."
wasm-pack build \
  --target nodejs \
  --out-dir ../../packages/core/wasm/node \
  --scope llm-shield \
  crates/llm-shield-wasm

echo "📦 Building WASM for bundler target..."
wasm-pack build \
  --target bundler \
  --out-dir ../../packages/core/wasm/bundler \
  --scope llm-shield \
  crates/llm-shield-wasm

# Optimize WASM files if wasm-opt is available
if command -v wasm-opt &> /dev/null; then
    echo "⚡ Optimizing WASM with wasm-opt..."

    for wasm_file in packages/core/wasm/*/llm_shield_wasm_bg.wasm; do
        if [ -f "$wasm_file" ]; then
            wasm-opt -Oz --enable-mutable-globals "$wasm_file" -o "$wasm_file"
        fi
    done
else
    echo "⚠️  wasm-opt not found, skipping optimization"
    echo "Install with: npm install -g wasm-opt"
fi

# Create package.json for each target
echo "📝 Creating package.json files..."

cat > packages/core/wasm/web/package.json <<EOF
{
  "name": "@llm-shield/wasm-web",
  "version": "0.1.0",
  "type": "module",
  "main": "llm_shield_wasm.js",
  "types": "llm_shield_wasm.d.ts"
}
EOF

cat > packages/core/wasm/node/package.json <<EOF
{
  "name": "@llm-shield/wasm-node",
  "version": "0.1.0",
  "type": "module",
  "main": "llm_shield_wasm.js",
  "types": "llm_shield_wasm.d.ts"
}
EOF

cat > packages/core/wasm/bundler/package.json <<EOF
{
  "name": "@llm-shield/wasm-bundler",
  "version": "0.1.0",
  "type": "module",
  "main": "llm_shield_wasm.js",
  "types": "llm_shield_wasm.d.ts"
}
EOF

echo "✨ WASM build complete!"
echo ""
echo "Output directories:"
echo "  - packages/core/wasm/web/"
echo "  - packages/core/wasm/node/"
echo "  - packages/core/wasm/bundler/"
