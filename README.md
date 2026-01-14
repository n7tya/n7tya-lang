# n7tya-lang 🍵

> フルスタックWebアプリを1言語で開発

**n7tya-lang（なちゃ言語）** は、Rustベースの新しいプログラミング言語です。

## 特徴

- 🐍 **Python連携** - pyo3によるPythonライブラリ呼び出し
- ⚡ **高速** - GCなし、Wasmファースト設計
- 📝 **シンプル構文** - インデントベース、ノイズ排除
- 🔧 **型安全** - 漸進的型付け、コンパイル時検出
- 🌐 **JSX統合** - フロントエンドもバックエンドも

## インストール

### ソースからビルド

```bash
# リポジトリをクローン
git clone https://github.com/n7tya/n7tya-lang.git
cd n7tya-lang/n7tya

# ビルド
cargo build --release

# パスを通す（オプション）
export PATH="$PATH:$(pwd)/target/release"
```

### 必要な環境
- Rust 1.80+ 
- Python 3.8+（Python連携を使う場合）

## 使い方

### ファイルを実行
```bash
n7tya hello.n7t
```

### プロジェクト作成
```bash
n7tya new myapp
cd myapp
n7tya run
```

### その他のコマンド
```bash
n7tya build    # 型チェック
n7tya test     # テスト実行
n7tya fmt      # コードフォーマット
n7tya check    # 型チェックのみ
```

## Hello World

`hello.n7t`:
```python
# n7tya-lang Hello World

def greet name: Str
	print "Hello, " + name + "!"

greet "World"
```

実行:
```bash
n7tya hello.n7t
# => Hello, World!
```

## 言語機能

```python
# 変数
let x = 10
const PI = 3.14

# 関数
def add a: Int, b: Int -> Int
	return a + b

# クラス
class Person
	name: Str
	
	def greet
		print "I am " + self.name

# 制御構文
if x > 5
	print "big"
elif x > 0
	print "small"
else
	print "zero"

# ループ
for i in range 10
	print i

# パターンマッチ
match status
	case 200
		print "OK"
	case 404
		print "Not Found"
	case _
		print "Unknown"
```

## ライセンス

MIT License

## 貢献

Issue・PRを歓迎します！
