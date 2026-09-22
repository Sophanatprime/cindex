# cindex

```
cindex [-c] [-i] [-o ind] [-q] [-r] [-s sty] [-t log] [--strict] [-z sorts] [--kv]
    [index0 index1 ...] [-- <extras>]
cindex -h, --help
cindex --license
cindex query [-q] <kind> <text>
cindex query -h, --help
```

cindex 是通用的索引制作工具，它的作用类似于 makeindex 和 zhmakeindex，功能和用法也和它们相似。但支持使用 lua（luajit 2.1）进行特殊定制。

输入输出的格式可以由 `ist` 格式确定，但也支持使用 `lua` 文件文件进行定制。

与 makeindex 一样，cindex 默认的输入/输出格式为 `idx`/`ind`。此外，cindex 还支持一种特殊的输入格式 `ikv`，使用这个格式无需对索引条目进行复杂的转义。同时 `ikv` 完全兼容 `idx`，也就是说，**一个合法的 `idx` 文件就是一个合法的 `ikv` 文件**。

如果没有显式指定，第一个输入文件的主文件名将用于，确定输出文件和日志文件的主文件名，它们并**不**继承该输入文件的目录。若未找到某输入文件，且该文件没有扩展名，则会加上 `.idx` 再次寻找。

不像 zhmakeindex，在没有提供格式文件的情况下，cindex 并**不**寻找默认的格式文件，而是直接使用程序的默认设置。

cindex 只接受 UTF-8 编码的文件，不论是格式文件，亦或是输入输出文件，必须使用 UTF-8 编码。


## 选项说明

- `-c` ，缩索引项排序项前后的空格。默认情况下，排序项中的空格会被保留。此选项对 `ikv` 格式的索引无效，默认情况下 `ikv` 格式总是忽略行首空格，保留行末空格。
- `-i`，从标准输入流（stdin）读入索引项。如果使用了该选项，并且没有使用 `-o` 选项，则排序后的索引将输出到标准输出流（stdout）。
- `-o <ind>`，设置输出索引文件为 `<ind>`。如果没有指定该选项，默认的输出文件名是第一个输入文件 `<index0>`的主文件名加上 `.ind` 扩展名。
- `-q`，静默模式，不向标准错误流（stderr）显示信息。默认情况下处理过程与错误信息会同时在 stderr 与日志文件中输出。
- `-r`，禁止隐式页码区间构造，要求页码区间必须使用显式区间符号生成。默认情况下，三个或三个以上连续的页码会自动合并为一个页码区间（如1–5）。
- `-s <style>`，设置 `<style>` 为格式文件。没有默认值。如果没有扩展名，会使用 `.ist` 作为扩展名。cindex 会首先在当前目录查找格式文件，如果找不到则调用调用 `kpsewhich` 命令行工具查找，可使用 `KPSEWHICH_EXE_FILE` 环境变量设置 `kpsewhich` 二进制文件的位置。
    除了使用常见的 `ist` 格式文件，也可使用 `lua` 文件。
- `-t <log>`，设置 `<log>` 为日志文件。默认情况下，会使用第一个输入文件 `<index0>` 的主文件名加上 `.ilg` 扩展名作为日志文件。
- `--strict`，严格区分不同嵌入命令的页码。默认情况下，在页码区间处理时，会将如果页码左区间的嵌入命令与右区间不匹配，会以左区间为准；而如果使用该选项，则要求左右区间的命令类型必须严格匹配。
- `-z <sorts>` 设置分组与排序方式为 `<sorts>`。由 cindex 直接支持的排序方式包括 pinyin/reading，bihua/stroke，bushou/radical。默认值为 pinyin，即中文按拼音分组排序。
    `<sorts>` 为以西文逗号分隔的列表。默认情况下，cindex 只检查 `<sorts>` 的第一项，可使用 `lua` 定制。
- `--kv`，标记输入文件均为 `ikv` 格式。cindex 会检查输入文件的扩展名，如果扩展名为 `ikv`，则将其视为 `ikv` 格式。
- `-- <extras>`，输入额外的选项。可由 `lua` 进一步处理。如
    ```
    cindex -s process.lua index.idx -- some other options
    ```
    则 `<extras>` 为表 `{ "some", "other", "options" }`。


## 索引项输入语法

`idx` 格式的输入文件，其语法和 makeindex/zhmakeindex 别无二致。该格式下，每行一个索引项，允许空行，但不得跨行。

如下是几个索引项的例子：
```
\indexentry{姓名}{15}
\indexentry{ExplSyntaxOn={\verbatim@font !\verb*&!\ExplSyntaxOn&}|hdpindex{usage}}{MMMMI-6}
```

`ikv` 格式的输入文件，每个索引项分为“头”和“具体层级”两个部分：
```
<页码><分隔符><Range><分隔符><命令><分隔符><层级数>
    <排序键1>
    <显示文本1>
    <排序键2>
    <显示文本2>
    ...
```
比如假定使用默认的分隔符 `,`，上述 `idx` 格式的输入可改写为
```
15,,,1
    姓名
    姓名
MMMMI-6,,hdpindex{usage},1
    ExplSyntaxOn
    \verbatim@font \verb*&\ExplSyntaxOn&
```
值得注意的是，分隔符前后不能有多余的空格（分隔符本身可以包含空格，可以是多个字符），即便排序键和显示文本相同，也不能省略其中任何一个。排序键行和显示文本行行首的空格可以省略，cindex 默认会忽略行首空格。

`ikv` 格式完全兼容 `idx` 格式，只把扩展名改为 `ikv`，而内容本身不做改变，同样也能被 cindex 正确识别。比起 `idx` 格式，`ikv` 无需（也不能）做任何转义。在 `ikv` 中使用 `idx` 格式的索引项则除外。

在 LaTeX 中使用时，索引项输入文件一般是在 LaTeX 编译过程中自动生成的。


## 使用 lua 定制

cindex 完全兼容 zhmakeindex 的 `ist` 格式文件。而对于 makeindex 支持的格式文件，cindex 不支持 `setpage_prefix`、`setpage_suffix`、`line_max`、`indent_space`、`indent_length` 这 5 个关键字（实际上这也不被 zhmakeindex 所支持）。其余均一致。

cindex 使用的 lua 文件进行定制，此 lua 文件的返回值必须为 `nil` 或 lua 表。cindex 会在合适的时候查询这张表。对于返回的表，cindex 目前会查询如下字段：
- `parse_indices_string: function(file_path: nil|string, lines: StrRef, is_ikv: bool) -> iterator`
此函数应当返回一个迭代器函数，每次调用这个迭代器，都会把它返回的内容作为一个结构化的索引条目，直到它返回 `nil`。

- `parse_index_line: function(line: StrRef) -> <Entry like>`
这函数解析某个输入行，它应返回一个结构化的索引条目。一般来讲，这输入行是 `idx` 格式的索引文本。当它与 `parse_indices_string` 同时给出时，cindex 只使用 `parse_indices_string`，因为后者覆盖更为广泛。
    `<Entry like>` 是 `IndexEntryMutRef` 或可转为此类型的表。这张表的字段如下：
    * `levels: { { key: string-like?, display: string-like }, .. }`，该字段是一张表，它的每一项也都是一张表，内部的子表长度为 1 或 2，类型为字符串类型。
    * `range: nil | "None" | "Open" | "Close"`，是否为页面区间的起始或结束。
    * `page_commands?: string-like`，页码的输出格式，为 `nil` 或字符串类型。
    * `pages?: { { kind: "Arabic" | "roman" | "Roman" | "alpha" | "Alpha" | "ZhDigits" | "ZhNumber" , value: number }, .. }`，页码。
    * `pages_raw?: string-like`，页码的原始字符串，在输出文件中使用此项作为页码。

- `empty_indices_callout: function(array: IndexEntryArray)`
当初次解析完后，如果没有任何有效的索引项，则调用此函数。

- `page_precedence: function | { string, .. } | string`
不同种页码的次序。页码的种类为 `"Arabic"(n) | "roman"(r) | "Roman"(R) | "alpha"(a) | "Alpha"(A) | "ZhDigits" | "ZhNumber"`，如果是以字符串而非以表给出，则字符串的每一项应为括号内的字符。如 `{ "roman", "Arabic", "alpha", "Roman", "Alpha", "ZhDigits", "ZhNumber" }` 和 `"rnaRA"` 都表示相同的优先级，这也是默认的优先级。如果是函数，则其返回值必须为表或字符串或 `nil`。

- `group_detect: function(entry: MergedEntryRef) -> string`
判断并返回条目的组别。cindex 预定义的组别及其排序为 `nil`，`Symbols`，`Numbers`，`A` \~ `Z`，`BiHua1`\~`BiHua64`（笔画数 1 至 64，超过 64 画的按 64 画计），`BuShou1` \~ `BuShou214`（康熙部首）。

- `group_compare: function(lhs: string, rhs: string, lhs_counts: number, rhs_counts: number) -> bool`
组别的排序。如果 `lhs` 要排在 `rhs` 前面，则返回 `true`。目前 cindex 使用的是 lua 内置的 `table.sort` 函数，它是不稳定排序。`lhs_counts` 和 `rhs_counts` 分别是对应组别在合并后所拥有的条目数。

- `entry_compare: function(lhs: MergedEntryRef, rhs: MergedEntryRef, group: string) -> bool`
条目的排序。如果 `lhs` 要排在 `rhs` 前面，则返回 `true`。目前 cindex 使用的是 lua 内置的 `table.sort` 函数，它是不稳定排序。`group` 为条目所属的组别。
    如果不同组别的排序方式不同，也可使用特化的 `entry_compare_<group>` 函数，比如 `entry_compare_Numbers` 如果存在，则对于 `Numbers` 这个组别，在对其条目排序时，会直接使用这个函数作为比较函数。对于按笔画数和部首的组别，除了会检查 `entry_compare_BiHua2` 这种外，cindex 还检查不带数字的排序函数：`entry_compare_BiHua`。 cindex 会优先使用更加具体的比较函数。

- `output_style: { .. }`
当使用 lua 作为格式文件时，便无法同时使用 `ist` 格式了。为此，可使用 `output_style` 表作为替代。这张表的字段就是 `ist` 中的相应关键字。

## License

Copyright (C) 2026 Wenjian Chern

This library is free software; you can redistribute it and/or modify it under the terms of the GNU Lesser General Public License as published by the Free Software Foundation; either version 2.1 of the License, or (at your option) any later version.

This library is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public License for more details.

You should have received a copy of the [GNU Lesser General Public License](LICENSE) along with this library; if not, write to the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA

### Third-Party Components and Data

This repository contains and/or incorporates third-party software and data. These components remain subject to their respective licenses and are **not relicensed as LGPL-2.1-or-later** merely because they are included in this project or statically embedded into the resulting executable.

- `CJKRadicals.txt`：https://www.unicode.org/reports/tr41/tr41-36.html#CJKRadicals [Unicode License V3].
- `Unihan_IRGSources.txt`、`Unihan_Readings.txt`：https://www.unicode.org/reports/tr41/tr41-36.html#Unihan [Unicode License V3].
- `StrokeOrder.txt`：https://github.com/CNMan/UnicodeCJK-WuBi/
    * License: **UNKNOWN**
- `ts.txt`：https://github.com/yi-bai/ids
    * [LICENSE](https://github.com/yi-bai/ids/blob/main/LICENSE)
- lpeg-1.1.0：https://www.inf.puc-rio.br/~roberto/lpeg/lpeg-1.1.0.tar.gz
    * [LICENSE](https://www.inf.puc-rio.br/~roberto/lpeg/#license)
- argparse-0.7.2：https://github.com/luarocks/argparse/blob/master
    * [LICENSE](https://github.com/luarocks/argparse/blob/master/LICENSE)
