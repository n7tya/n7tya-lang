# n7tya-lang 言語リファレンス

> シンプルで強力なフルスタック言語

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

### データ型

| 型 | 例 | 説明 |
|---|---|---|
| Int | `42`, `-10` | 整数 |
| Float | `3.14`, `-0.5` | 浮動小数点 |
| Str | `"hello"` | 文字列 |
| Bool | `true`, `false` | 真偽値 |
| List | `[1, 2, 3]` | リスト |
| Dict | `{"a": 1}` | 辞書 |
| None | `none` | 空値 |

---

## 関数

```python
# 基本的な関数
def greet name: Str
    print "Hello, " + name

# 戻り値の型指定
def add a: Int, b: Int -> Int
    return a + b

# ラムダ式
let double = x -> x * 2
let sum = (a, b) -> a + b
```

---

## 制御構文

### 条件分岐

```python
if x > 10
    print "big"
elif x > 0
    print "small"
else
    print "zero or negative"
```

### ループ

```python
# forループ
for i in range(10)
    print i

for item in [1, 2, 3]
    print item

# whileループ
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

---

## 組み込み関数

### 入出力
| 関数 | 説明 | 例 |
|---|---|---|
| `print(...)` | 出力（改行なし） | `print "Hello"` |
| `println(...)` | 出力（改行あり） | `println "Hello"` |
| `input(prompt)` | 入力受付 | `let s = input("Name: ")` |

### 型変換
| 関数 | 説明 |
|---|---|
| `str(x)` | 文字列に変換 |
| `int(x)` | 整数に変換 |
| `float(x)` | 浮動小数点に変換 |
| `type(x)` | 型名を取得 |

### コレクション操作
| 関数 | 説明 | 例 |
|---|---|---|
| `len(x)` | 長さ取得 | `len([1,2,3])` → `3` |
| `range(n)` | 0〜n-1のリスト | `range(5)` → `[0,1,2,3,4]` |
| `range(a,b)` | a〜b-1のリスト | `range(2,5)` → `[2,3,4]` |
| `sum(list)` | 合計 | `sum([1,2,3])` → `6` |
| `sorted(list)` | ソート済みリスト | `sorted([3,1,2])` → `[1,2,3]` |
| `reversed(list)` | 逆順リスト | `reversed([1,2,3])` → `[3,2,1]` |
| `enumerate(list)` | インデックス付き | `enumerate(["a","b"])` → `[[0,"a"],[1,"b"]]` |
| `zip(a, b)` | ペアリスト | `zip([1,2],["a","b"])` → `[[1,"a"],[2,"b"]]` |

### 数値
| 関数 | 説明 |
|---|---|
| `abs(x)` | 絶対値 |
| `min(...)` | 最小値 |
| `max(...)` | 最大値 |

---

## メソッド

### List メソッド

```python
let items = [1, 2, 3]

items.append(4)      # 末尾に追加 → [1, 2, 3, 4]
items.pop()          # 末尾を削除・取得 → 4
items.insert(0, 0)   # 位置指定で挿入 → [0, 1, 2, 3]
items.clear()        # 全削除 → []
items.index(2)       # 値の位置 → 1
items.count(1)       # 出現回数 → 1
items.copy()         # コピー作成
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
s.find("Wo")           # → 6
s.contains("llo")      # → true
```

### Dict メソッド

```python
let d = {"a": 1, "b": 2}

d.keys()              # → ["a", "b"]
d.values()            # → [1, 2]
d.items()             # → [["a", 1], ["b", 2]]
d.get("a")            # → 1
d.get("c", 0)         # → 0 (デフォルト値)
d.pop("a")            # キー削除・値取得
d.clear()             # 全削除
d.contains("b")       # → true
```

---

## クラス

```python
class Person
    name: Str
    age: Int

    def greet
        print "I am " + self.name
```

---

## Python連携

```python
# Pythonモジュールのインポート
from math import sqrt, floor

let result = sqrt(16)
println result  # → 4.0
```

---

## サーバー定義

```python
server MyApp
    route "/" GET
        return "Hello, World!"

    route "/api/data" GET
        return {"status": "ok"}
```

---

## コンポーネント (実験的)

```python
component Counter
    state count = 0

    def increment
        count = count + 1

    render
        <div>
            <span>{count}</span>
            <button onclick={increment}>+</button>
        </div>
```
