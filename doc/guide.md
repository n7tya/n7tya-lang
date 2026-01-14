# n7tya-lang ドキュメント

## 目次
- [インストール](#インストール)
- [基本構文](#基本構文)
- [型システム](#型システム)
- [CLIコマンド](#cliコマンド)

---

## インストール

```bash
git clone https://github.com/n7tya/n7tya-lang.git
cd n7tya-lang/n7tya
cargo build --release
```

ビルド後、`target/release/n7tya` が実行ファイルになります。

---

## 基本構文

### Hello World

```python
print "Hello, World!"
```

### 変数

```python
let x = 10           # 変数
const PI = 3.14      # 定数
```

### 関数

```python
def add a: Int, b: Int -> Int
	return a + b

let result = add 3, 5
print result  # 8
```

### 条件分岐

```python
if x > 10
	print "big"
elif x > 5
	print "medium"
else
	print "small"
```

### ループ

```python
# for ループ
for i in range 5
	print i

# while ループ
let count = 0
while count < 5
	print count
	count = count + 1
```

### パターンマッチ

```python
match status
	case 200
		print "OK"
	case 404
		print "Not Found"
	case _
		print "Unknown"
```

### クラス

```python
class Person
	name: Str
	age: Int
	
	def greet
		print "Hello, " + self.name
```

### リスト

```python
let numbers = [1, 2, 3, 4, 5]
print len numbers  # 5
print numbers[0]   # 1
```

---

## 型システム

| 型 | 説明 |
|----|------|
| `Int` | 整数 |
| `Float` | 浮動小数点 |
| `Str` | 文字列 |
| `Bool` | 真偽値 |
| `List<T>` | リスト |

---

## CLIコマンド

```bash
n7tya <file.n7t>     # ファイル実行
n7tya run            # プロジェクト実行
n7tya build          # ビルド（型チェック）
n7tya test           # テスト実行
n7tya fmt            # フォーマット
n7tya new <name>     # 新規プロジェクト作成
n7tya check <file>   # 型チェックのみ
n7tya --version      # バージョン表示
n7tya --update       # アップデート手順表示
```

---

## 組み込み関数

| 関数 | 説明 |
|------|------|
| `print` | 出力 |
| `len` | 長さ取得 |
| `range` | 範囲生成 |
| `input` | 入力取得 |
| `str` | 文字列変換 |
| `int` | 整数変換 |
| `float` | 浮動小数点変換 |
| `type` | 型取得 |
| `abs` | 絶対値 |
| `min` / `max` | 最小/最大値 |
