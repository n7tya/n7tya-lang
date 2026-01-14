# n7tya-lang 言語仕様書

> バージョン: 0.1.0（ドラフト）
> 最終更新: 2026-01-14

---

## 1. 概要

### 1.1 言語の目的
- フルスタックWebアプリを1言語で開発
- Pythonの書きやすさ + Rustの安全性・速度

### 1.2 ファイル拡張子
- `.n7t` - n7tya-langソースファイル（JSX含む全てのコード）

---

## 2. 字句構造

### 2.1 エンコーディング
- UTF-8

### 2.2 インデント
- タブ（Go式）
- インデントでブロックを表現
- スペースとの混在は禁止
- フォーマッタ（`n7tya fmt`）で自動統一

### 2.3 コメント
```python
# 行コメント

###
複数行コメント
###
```

### 2.4 予約語
```
def fn let const if else elif for while 
return import from as class struct enum
match case break continue pass
async await yield
true false none
and or not in is
component server route
test assert
```

---

## 3. 型システム

### 3.1 プリミティブ型
| 型名 | 説明 | 例 |
|------|------|-----|
| `Int` | 64bit整数 | `42` |
| `Float` | 64bit浮動小数点 | `3.14` |
| `Bool` | 真偽値 | `true`, `false` |
| `Str` | 文字列（UTF-8） | `"hello"` |
| `None` | 値なし | `none` |

### 3.2 コレクション型
| 型名 | 説明 | 例 |
|------|------|-----|
| `List[T]` | 可変長配列 | `[1, 2, 3]` |
| `Dict[K, V]` | 辞書 | `{"key": "value"}` |
| `Set[T]` | 集合 | `{1, 2, 3}` |
| `Tuple[T, U]` | タプル | `(1, "a")` |

### 3.3 型推論
```python
# 型を書かなくても推論される
x = 42          # Int
name = "Alice"  # Str
items = [1, 2]  # List[Int]

# 明示的に書くことも可能
y: Float = 3.14
```

### 3.4 構造的部分型
```python
# 同じ構造なら同じ型として扱う
struct Point
    x: Int
    y: Int

struct Coordinate
    x: Int
    y: Int

# Point と Coordinate は互換性あり
def print_x obj
    print obj.x

print_x Point(1, 2)      # OK
print_x Coordinate(3, 4) # OK
```

---

## 4. 変数と定数

### 4.1 変数宣言
```python
# 変更可能
let x = 10
x = 20  # OK

# 変更不可
const PI = 3.14159
PI = 3  # コンパイルエラー
```

### 4.2 所有権（自動推論）
```python
# ユーザーは所有権を意識しない
def process data
    result = transform data  # 所有権は自動で移動
    return result

# コンパイラが裏で所有権を追跡
# 推論不能な場合はコンパイルエラーと修正提案
```

---

## 5. 関数

### 5.1 関数定義
```python
# 基本形（型推論）
def add a, b
    return a + b

# 型を明示
def greet name: Str -> Str
    return "Hello, " + name
```

### 5.2 ラムダ式
```python
square = x -> x * x
add = (a, b) -> a + b
```

### 5.3 非同期関数
```python
async def fetch_data url
    response = await http.get url
    return response.json
```

---

## 6. 制御構文

### 6.1 条件分岐
```python
if x > 0
    print "positive"
elif x < 0
    print "negative"
else
    print "zero"
```

### 6.2 ループ
```python
# for ループ
for item in items
    print item

# for with index
for i, item in enumerate items
    print i, item

# while ループ
while condition
    do_something
```

### 6.3 パターンマッチ
```python
match value
    case 0
        print "zero"
    case 1..10
        print "small"
    case _
        print "other"
```

---

## 7. コンポーネント（JSX統合）

### 7.1 基本構文
```python
component Button
    props
        label: Str
        onclick: Fn
    
    render
        <button class="btn" onclick={self.onclick}>
            {self.label}
        </button>
```

### 7.2 状態管理
```python
component Counter
    state count = 0
    
    def increment
        self.count += 1
    
    render
        <div>
            <p>Count: {self.count}</p>
            <button onclick={self.increment}>+1</button>
        </div>
```

---

## 8. サーバー定義

### 8.1 基本構文
```python
server app
    port = 8080
    
    route "/" method: GET
        return html "<h1>Hello</h1>"
    
    route "/api/data" method: POST
        data = request.json
        return json result: process data
```

---

## 9. Python連携

### 9.1 インポート
```python
# Pythonライブラリをそのまま使用
import numpy as np
import torch
from langchain import ChatOpenAI

# 使用
array = np.array [1, 2, 3]
model = ChatOpenAI model: "gpt-4"
```

### 9.2 型の相互運用
- Pythonオブジェクトは `PyObject` 型として扱う
- 参照カウントで自動管理
- n7tya-lang型との変換は自動

---

## 10. クラス定義

### 10.1 基本構文
```python
class ChatBot
	model: ChatOpenAI
	memory: ConversationMemory
	
	def __init__
		self.model = ChatOpenAI model: "gpt-4"
		self.memory = ConversationMemory
	
	def respond message: Str -> Str
		return self.model.chat message, memory: self.memory
```

### 10.2 継承
```python
class AdvancedBot ChatBot
	def respond message: Str -> Str
		# オーバーライド
		processed = preprocess message
		return super.respond processed
```

---

## 11. テスト

### 11.1 テスト定義
```python
test "足し算のテスト"
    assert add(1, 2) == 3

test "非同期APIのテスト"
    response = await fetch "/api/health"
    assert response.status == 200
```

### 11.2 実行
```bash
n7tya test           # 全テスト実行
n7tya test src/      # ディレクトリ指定
```

---

## 12. パッケージ管理

### 12.1 n7tya.toml
```toml
[package]
name = "my-app"
version = "0.1.0"

[dependencies]
http = "^1.0"

[python]
packages = ["numpy", "torch"]

[server]
port = 8080

[build]
target = ["native", "wasm"]
```

### 12.2 CLI

#### プロジェクト管理
```bash
n7tya new my-app          # 新規プロジェクト作成
n7tya init                # 既存ディレクトリを初期化
```

#### 依存関係
```bash
n7tya add http            # パッケージ追加
n7tya remove http         # パッケージ削除
n7tya sync                # n7tya.lockから依存関係を同期（uv sync相当）
n7tya upgrade             # 依存関係を最新版に更新
n7tya audit               # セキュリティ脆弱性チェック
```

#### 開発
```bash
n7tya run                 # 実行
n7tya dev                 # 開発サーバー（ホットリロード）
n7tya repl                # 対話型シェル
```

#### ビルド・チェック
```bash
n7tya build               # ビルド
n7tya build --target wasm # WASMビルド
n7tya check               # 型チェック（ビルドなし）
n7tya fmt                 # コードフォーマット
```

#### テスト
```bash
n7tya test                # 全テスト実行
n7tya test src/           # ディレクトリ指定
n7tya test --watch        # ファイル変更時に自動再実行
```

#### セキュリティ
```bash
n7tya run --allow-net     # ネットワークアクセス許可
n7tya run --allow-read    # ファイル読み取り許可
n7tya run --allow-all     # 全権限許可
```

#### 公開・ドキュメント
```bash
n7tya publish             # パッケージを公開
n7tya doc                 # ドキュメント生成
n7tya doc --open          # ブラウザで開く
```

---

## 13. 標準ライブラリ（予定）

| モジュール | 説明 |
|-----------|------|
| `io` | ファイル入出力 |
| `http` | HTTPクライアント/サーバー |
| `json` | JSON処理 |
| `crypto` | 暗号化 |
| `time` | 日時処理 |
| `async` | 非同期ユーティリティ |

---

*ドラフト版 - フィードバックに基づき更新予定*
