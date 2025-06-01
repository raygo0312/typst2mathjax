cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/typst2mathjax.wasm
cp pkg/typst2mathjax_bg.wasm /mnt/c/Users/raygo/OneDrive/development/study_note/lib/typst2mathjax_bg.wasm
cp pkg/typst2mathjax.js /mnt/c/Users/raygo/OneDrive/development/study_note/lib/typst2mathjax.js