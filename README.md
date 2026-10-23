This crate is a fork of slotmap with additional features. Keys are provided by
pi_key with full 32-bit versions. Enable the serde feature to serialize maps
and keys stored in their values.

# slotmap

A Rust library providing three containers with persistent unique keys to access
stored values, `SlotMap`, `HopSlotMap` and `DenseSlotMap`. Upon insertion a key
is returned that can be used to later access or remove the values. Insertion,
deletion and access all take O(1) time with low overhead. Great for storing
collections of objects that need stable, safe references but have no clear
ownership otherwise, such as game entities or graph nodes. Two secondary maps,
`SecondaryMap` and `SparseSecondaryMap` are also provided that allow you to map
further objects to the keys created by one of the slot maps. Please refer to the
[**the documentation**](https://docs.rs/slotmap) for more information.

To start using `pi_slotmap`, add the following to your `Cargo.toml`:

```toml
[dependencies]
pi_slotmap = "0.5"
```

# Example

A short example:

```rust
use pi_slotmap::{SlotMap, SecondaryMap};

let mut sm = SlotMap::new();
let foo = sm.insert("foo");  // Key generated on insert.
let bar = sm.insert("bar");
assert_eq!(sm[foo], "foo");
assert_eq!(sm[bar], "bar");

sm.remove(bar);
let reuse = sm.insert("reuse");  // Space from bar reused.
assert_eq!(sm.contains_key(bar), false);  // After deletion a key stays invalid.

let mut sec = SecondaryMap::new();
sec.insert(foo, "noun");  // We provide the key for secondary maps.
sec.insert(reuse, "verb");

for (key, val) in sm {
    println!("{} is a {}", val, sec[key]);
}
```

# SlotMap 整理与缩容

以下接口仅针对普通 `SlotMap`：

| 方法 | 行为 | 版本与保留位 |
| --- | --- | --- |
| `compact()` | 按索引升序重建空闲链；不移动元素、不改变 Vec 长度和容量 | 全部保留 |
| `shrink_to_fit()` | 仅释放 Vec 长度以外的预留容量 | 全部保留 |
| `shrink_to_last_occupied()` | 截掉最后占用槽位之后的空闲槽，重建空闲链并缩容；空表保留哨兵 | 被截槽位的版本和 bits 丢失 |

整理后的空闲链包含尾部空闲槽，链尾指向 `slots.len()`。插入时索引小于该长度则复用槽位和版本，否则追加新槽位。整理和截尾按已初始化槽位数量线性扫描。中间空洞不会被截掉。

截尾缩容保留存活元素的 key，但后续追加可能再次产生已删除的旧 key。调用方须处理这些旧 key 及 secondary map 中的关联数据。

# SlotMap 版本号保留位

`SlotMap<K, V, const N: u32 = 0>` 使用 32 位版本号：

- 低 N 位由调用方设置，新槽位初始为 0。
- 第 N 位为占用位：1 表示使用中，0 表示空闲；N = 0 时仍是奇数占用、偶数空闲。
- 每次释放和复用均以回绕加法增加 `1 << N`，低 N 位不变。
- 所有 key 访问严格比较完整版本，包含低 N 位。
- N 必须小于 32；相同 bits 下，版本经过 `2^(31-N)` 次释放/复用周期后重复。N = 31 时一次周期就重复。

`set_key_bits(key, bits) -> Option<K>` 要求 key 严格有效，返回更新后的 key。bits 改变后旧 key 不匹配；无效 key 返回 None；bits 超出 N 位则 panic 且不修改槽位。N = 0 时只允许 bits = 0。将 bits 改回旧值可能让同代际的旧 key 再次匹配。

`get_key_bits(index) -> Option<u32>` 按索引读取占用槽位的 bits，不检查代际；不存在或空闲时返回 None。构造 key 时必须保留原来的高位代际，不能仅凭索引和 bits 绕过代际检查。

```rust
use pi_slotmap::{DefaultKey, Key, KeyData, SlotMap};

let mut sm = SlotMap::<DefaultKey, i32, 2>::with_key();
let old = sm.insert(42); // version = 0b0100
let updated = sm.set_key_bits(old, 0b11).unwrap(); // version = 0b0111
assert!(sm.get(old).is_none());
assert_eq!(sm.get(updated), Some(&42));

let data = old.data();
let bits = sm.get_key_bits(data.index()).unwrap();
let rebuilt: DefaultKey = KeyData::new(
    data.index(), (data.version() & !0b11) | bits,
).into();
assert_eq!(rebuilt, updated);

sm.compact();
sm.shrink_to_fit();
```

非零 N 使用 `with_key()`、`with_capacity_and_key(capacity)` 或 `default()` 构造；默认 N 的 `SlotMap::new()` 和 `SlotMap::with_capacity(capacity)` 保持原用法。自定义 key 的构造若无法推导 N，可显式写 `SlotMap::<MyKey, _>::with_capacity_and_key(capacity)`。

修改 bits 后需同步更新调用方保存的 key、值中内嵌的 key 和 secondary map 关联数据。序列化保存完整版本及 bits；反序列化必须使用相同的 N，序列化数据本身不记录 N。

# License

`slotmap` is released under the Zlib license, a permissive license. It is
OSI and FSF approved and GPL compatible.
