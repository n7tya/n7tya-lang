# n7tya-lang 言語リファレンス

> シンプルで強力なフルスタック言語

## 目次
1. [基本構文](#基本構文)
2. [データ型](#データ型)
3. [関数](#関数)
4. [制御構文](#制御構文)
5. [組み込み関数](#組み込み関数)
6. [メソッド](#メソッド)
7. [文字列](#文字列)
8. [クラス](#クラス)
9. [サーバー](#サーバー)
10. [Python連携](#python連携)

---

## 基本構文

### 変数宣言

```python
# 変更可能な変数
let x = 10
let name = "Taro"

# 定数 (変更不可)
const PI = 3.14159
const MAX_SIZE = 100
```

### インデント

n7tya はインデントベースの言語です。ブロックはタブ（または4スペース）でインデントします。

```python
def greet name
    println "Hello, " + name    # 1レベルのインデント
    if name == "World"
        println "Welcome!"      # 2レベルのインデント
```

### コメント

```python
# これは1行コメント
let x = 10  # 行末コメント
```

---

## データ型

| 型 | 例 | 説明 |
|---|---|---|
| `Int` | `42`, `-10` | 64ビット整数 |
| `Float` | `3.14`, `-0.5` | 64ビット浮動小数点 |
| `Str` | `"hello"` | 文字列 |
| `Bool` | `true`, `false` | 真偽値 |
| `List` | `[1, 2, 3]` | リスト |
| `Dict` | `{"a": 1}` | 辞書 |
| `None` | `none` | 空値 |

---

## 関数

### 関数定義

```python
# 基本的な関数
def greet name
    println "Hello, " + name

# 型注釈付き
def add a: Int, b: Int -> Int
    return a + b

# 複数の文
def factorial n
    if n <= 1
        return 1
    return n * factorial(n - 1)
```

### ラムダ式

```python
# 単一引数
let double = x -> x * 2

# 複数引数
let sum = (a, b) -> a + b

# 使用例
let result = double(5)      # 10
let total = sum(3, 4)       # 7
```

---

## 制御構文

### 条件分岐

```python
if x > 10
    println "big"
elif x > 0
    println "small"
else
    println "zero or negative"
```

### for ループ

```python
# リストをイテレート
for item in [1, 2, 3]
    println item

# range を使用
for i in range(5)           # 0, 1, 2, 3, 4
    println i

for i in range(2, 5)        # 2, 3, 4
    println i

for i in range(0, 10, 2)    # 0, 2, 4, 6, 8
    println i
```

### while ループ

```python
let count = 0
while count < 5
    println count
    count = count + 1
```

### break / continue

```python
for i in range(10)
    if i == 5
        break       # ループを終了
    if i % 2 == 0
        continue    # 次のイテレーションへ
    println i
```

### パターンマッチ

```python
match status
    case 200
        println "OK"
    case 404
        println "Not Found"
    case _          # ワイルドカード
        println "Unknown"
```

---

## 組み込み関数

### 入出力

| 関数 | 説明 | 例 |
|---|---|---|
| `print(...)` | 出力（改行なし） | `print("Hello")` |
| `println(...)` | 出力（改行あり） | `println("Hello")` |
| `input(prompt)` | 入力受付 | `let s = input("Name: ")` |

### 型変換

| 関数 | 説明 | 例 |
|---|---|---|
| `str(x)` | 文字列に変換 | `str(42)` → `"42"` |
| `int(x)` | 整数に変換 | `int("42")` → `42` |
| `float(x)` | 浮動小数点に変換 | `float("3.14")` → `3.14` |
| `type(x)` | 型名を取得 | `type([1,2])` → `"List"` |

### コレクション操作

| 関数 | 説明 | 例 |
|---|---|---|
| `len(x)` | 長さ取得 | `len([1,2,3])` → `3` |
| `range(n)` | 0〜n-1のリスト | `range(5)` → `[0,1,2,3,4]` |
| `range(a, b)` | a〜b-1のリスト | `range(2, 5)` → `[2,3,4]` |
| `range(a, b, step)` | ステップ付き | `range(0, 10, 2)` → `[0,2,4,6,8]` |
| `sum(list)` | 合計 | `sum([1,2,3])` → `6` |
| `sorted(list)` | ソート済みリスト | `sorted([3,1,2])` → `[1,2,3]` |
| `reversed(list)` | 逆順リスト | `reversed([1,2,3])` → `[3,2,1]` |
| `enumerate(list)` | インデックス付き | `enumerate(["a","b"])` → `[[0,"a"],[1,"b"]]` |
| `zip(a, b)` | ペアリスト | `zip([1,2],["a","b"])` → `[[1,"a"],[2,"b"]]` |

### 数値

| 関数 | 説明 | 例 |
|---|---|---|
| `abs(x)` | 絶対値 | `abs(-5)` → `5` |
| `min(...)` | 最小値 | `min(1, 2, 3)` → `1` |
| `max(...)` | 最大値 | `max(1, 2, 3)` → `3` |

---

## メソッド

### List メソッド

```python
let items = [1, 2, 3]

items.append(4)        # 末尾に追加 → [1, 2, 3, 4]
items.pop()            # 末尾を削除・取得 → 4
items.insert(0, 0)     # 位置指定で挿入 → [0, 1, 2, 3]
items.clear()          # 全削除 → []
items.index(2)         # 値の位置 → 1
items.count(1)         # 出現回数
items.copy()           # コピー作成
```

### Str メソッド

```python
let s = "Hello World"

s.upper()              # → "HELLO WORLD"
s.lower()              # → "hello world"
s.strip()              # 前後の空白削除
s.split(" ")           # → ["Hello", "World"]
",".join(["a","b"])    # → "a,b"
s.replace("o", "0")    # → "Hell0 W0rld"
s.startswith("Hello")  # → true
s.endswith("World")    # → true
s.find("Wo")           # → 6 (見つからない場合は -1)
s.contains("llo")      # → true
```

### Dict メソッド

```python
let d = {"a": 1, "b": 2}

d.keys()               # → ["a", "b"]
d.values()             # → [1, 2]
d.items()              # → [["a", 1], ["b", 2]]
d.get("a")             # → 1
d.get("c", 0)          # → 0 (デフォルト値)
d.pop("a")             # キー削除・値取得
d.clear()              # 全削除
d.contains("b")        # → true
```

---

## 文字列

### エスケープシーケンス

| シーケンス | 意味 |
|---|---|
| `\n` | 改行 |
| `\t` | タブ |
| `\r` | キャリッジリターン |
| `\\` | バックスラッシュ |
| `\"` | ダブルクォート |
| `\'` | シングルクォート |

```python
let s = "Hello\nWorld"   # 2行の文字列
println s
# 出力:
# Hello
# World
```

### 複数行文字列

バッククォート `` ` `` で囲むと、改行を含む文字列をそのまま記述できます。

```python
let html = `
<html>
<body>
    <h1>Hello</h1>
</body>
</html>
`
```

### 文字列連結

`+` 演算子で連結します。

```python
let greeting = "Hello, " + name + "!"
```

---

## クラス

```python
class Person
    name: Str
    age: Int

    def greet
        println "I am " + self.name
```

---

## サーバー

HTTPサーバーを定義できます。

```python
server MyApp
    GET "/"
        return "Hello, World!"

    GET "/api/data"
        return "OK"
```

**注意**: サーバー定義内でも空行を含めることができます。

---

## Python連携

n7tya は Python ライブラリを呼び出すことができます。

```python
# モジュールのインポート
from math import sqrt, floor

let result = sqrt(16)
println result  # → 4.0
```

### 使用可能な Python 機能

- 標準ライブラリ (math, json, os など)
- サードパーティライブラリ (インストール済みのもの)

**制約**: 
- 複雑なオブジェクトの受け渡しは制限があります
- クラスインスタンスは n7tya の Dict に変換されます

---

## 演算子

### 算術演算子

| 演算子 | 説明 | 例 |
|---|---|---|
| `+` | 加算 / 文字列連結 | `1 + 2` → `3` |
| `-` | 減算 | `5 - 3` → `2` |
| `*` | 乗算 | `2 * 3` → `6` |
| `/` | 除算 | `10 / 3` → `3` |
| `%` | 剰余 | `10 % 3` → `1` |

### 比較演算子

| 演算子 | 説明 |
|---|---|
| `==` | 等価 |
| `!=` | 非等価 |
| `<` | より小さい |
| `>` | より大きい |
| `<=` | 以下 |
| `>=` | 以上 |

### 論理演算子

| 演算子 | 説明 |
|---|---|
| `and` | 論理AND |
| `or` | 論理OR |
| `not` | 論理NOT |

### メンバーシップ

```python
if "a" in ["a", "b", "c"]
    println "found"
```
