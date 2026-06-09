# Reovim 移植詳細タスクリスト (TASKS.md)

本ドキュメントは、[PLAN.md](file:///Users/echirui/work/reovim/PLAN.md) で定義された実装計画に基づき、各フェーズで具体的にどのファイルを作成、変更、または削除・無効化するかを定義したタスクリストです。

---

## 🛠 フェーズ 1: ビルドシステムの統合 (CMake + Cargo)

### 1. Rust側: スタティックライブラリ化
- [x] **【変更】** `Cargo.toml`
  - `[lib]` セクションを追加。
  - `crate-type = ["staticlib"]` を設定。
- [x] **【作成】** `src/lib.rs`
  - Cから呼び出し可能なダミー関数 `reovim_hello` を定義。
  ```rust
  #[unsafe(no_mangle)]
  pub extern "C" fn reovim_hello() {
      println!("Hello from Rust!");
  }
  ```

### 2. Neovim側: ビルドプロセスへのRustリンク追加
- [x] **【変更】** `vim_src/neovim-0.12.2/src/nvim/CMakeLists.txt`
  - Cargoを呼び出して `libreovim.a` をビルドする `add_custom_target` または `add_custom_command` を追加。
  - Neovimバイナリのターゲットである `nvim_bin` に対し、`libreovim.a` をリンクする設定を追加。
- [x] **【変更】** `vim_src/neovim-0.12.2/src/nvim/main.c`
  - `reovim_hello` の C 向けプロトタイプ宣言（`extern void reovim_hello(void);`）を追加。
  - `main` 関数の初期化ステップで `reovim_hello()` を呼び出すように変更。

---

## 📦 フェーズ 2: 依存の少ないユーティリティモジュールの移植

### 1. `sha256.c` の移植
- [x] **【変更】** `Cargo.toml`
  - 依存関係を追加せずにピュアRustで完全に等価な動作を自己完結実装。
- [x] **【作成】** `src/sha256.rs`
  - Cの `sha256.c` のすべてのインターフェースをRustで実装し、FFI経由で公開。
    - `sha256_start`, `sha256_update`, `sha256_finish`, `sha256_bytes`, `sha256_self_test`
- [x] **【変更】** `src/lib.rs`
  - `mod sha256;` を追加してモジュールを公開。
- [x] **【変更】** `vim_src/neovim-0.12.2/src/nvim/CMakeLists.txt` (および `sha256.h` の修正)
  - ソースコード一覧（glob）から `sha256.c` を除外。また `sha256.h` に関数プロトタイプを手動定義。
- [x] **【削除/リネーム】** `vim_src/neovim-0.12.2/src/nvim/sha256.c`
  - `sha256.c.orig` にリネームしてC側のビルド対象から完全に除外。

### 2. `path.c` の移植
- [ ] **【作成】** `src/path.rs`
  - Cの `path.c` 内の文字列操作や絶対・相対パス判定関数をRustで実装。
- [ ] **【変更】** `src/lib.rs`
  - `mod path;` を追加。
- [ ] **//【変更】** `vim_src/neovim-0.12.2/src/nvim/CMakeLists.txt`
  - ソースコード一覧から `path.c` を除外。
- [ ] **【削除/リネーム】** `vim_src/neovim-0.12.2/src/nvim/path.c`

---

## 🔗 フェーズ 3: FFI バインディングの自動生成とデータ構造の共有

### 1. `bindgen` の導入
- [ ] **【変更】** `Cargo.toml`
  - `[build-dependencies]` に `bindgen` を追加。
- [ ] **【作成】** `build.rs`
  - Neovimのヘッダーファイル（例：`vim_src/neovim-0.12.2/src/nvim/types.h` や `globals.h` など）を解析して、Rustのバインディング（`bindings.rs`）を自動生成するビルドスクリプトを作成。
- [ ] **【変更】** `src/lib.rs`
  - 自動生成されたバインディングを取り込む `pub mod c_api` を追加。

---

## 🚀 フェーズ 4: コアロジックの段階的リプレース

### 1. MessagePack-RPCの移植
- [ ] **【作成】** `src/msgpack_rpc/` (ディレクトリと関連ファイル)
  - RPCハンドラ、メッセージデシリアライザをRustで実装。
- [ ] **【変更】** `src/lib.rs`
- [ ] **【変更】** `vim_src/neovim-0.12.2/src/nvim/CMakeLists.txt`
  - `src/nvim/msgpack_rpc/` のCコードをビルド対象から除外。
- [ ] **【削除/リネーム】** `vim_src/neovim-0.12.2/src/nvim/msgpack_rpc/` 内の該当Cファイル。
