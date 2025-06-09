# コンパイラとして使いたい場合

このプロジェクトを`cargo run`で実行してください．distディレクトリにコンパイル後格納されます．

- staticディレクトリ内にあるファイル構造はそのままdistディレクトリにコピーされます．
- pagesディレクトリ内にあるファイルは$で囲まれた部分を置換します．

# jsファイルとして使いたい場合

pkgディレクトリ内にある，mathmatics.jsとtypst2mathjax_bg.wasmとtypst2mathjax.jsを同じディレクトリ内に置き，mathmatics.jsを読み込んでください．
