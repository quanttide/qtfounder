# `qtfounder meta` 用户指南

`code`、`language`、`docs` 查的是写得对不对，`meta` 判的是放得对不对。一份内容只有一个正确去处，放错格的代价不是难看，是下一轮没人找得到它。

`classify` 一次判一条线索：给一段内容，出一张落位卡。卡上四行——域、格、档、写法——加一个结论：收，还是收不了。

## 十一条命令

名词读本体，动词分三种：`classify` 把内容放进格，`validate` 看有没有漂，`compare` 比两个范式。

| 部分 | 命令 | 看什么 |
|---|---|---|
| 模型 | `meta model` | 两族九宫格的轴与格 |
| 关系 | `meta relation` | 一格的迁出、迁入与分辨句 |
| 公理 | `meta axiom` | 归位时不重新论证的前提 |
| 属性 | `meta property` | 标准属性的字段名、组与定义 |
| 范畴 | `meta category` | 十八格与两端各收什么 |
| 去处 | `meta home` | 内容搬出本仓能去哪里 |
| 域（本体论） | `meta domain` | 这个域里有哪些范畴、属性与公理 |
| 范式 | `meta paradigm` | 每个范式的对象、态射与合成律 |
| 范畴（动词） | `meta classify` | 给一条内容一张落位卡 |
| 验证（动词） | `meta validate` | 文档镜像与契约是否还对得上 |
| 对比（动词） | `meta compare` | 两个范式对上了什么、各自独有什么 |

## 一、四步，顺序不能换

先定去处：留在这里，还是搬去别的仓。定去处只看正文讲哪个领域，不看它现在住在哪。单个词命中不足以指路，免得一个顺带出现的领域词把整篇搬走。正文里有已废弃、旧版这类词，搬的时候按归档处理，只移动不改写；否则是移交，要按目标侧的体例重写。

再定格：十八格里的哪一格，对应哪个目录。陈述型先看时间（过去、现在、未来），再看类型（事件、语义、自我）；程序型先看约束力（宪法、法律、法理），再看写给谁用（人读、AI 解析、直接运行）。

再过准入：名义上进对了格、但过不了的，往下一格走。阈值没有出处退论文；换一个实例就散架的手册退案例；推不翻的话进论文不进洞察。

最后给写法：那一格通常要交代什么。它是提示，不是模板；陈述型尤其不规定章节顺序。

## 二、一张卡长什么样

```text
落位卡

去处：留在这里
范畴：Insight 洞察  data/insight/
档：假说
凭什么：一句话能被推翻，推翻之后做法会变；抽掉日期和当事人还成立。
公理：template-scope（确认）、one-place（怀疑）、home-before-category（怀疑）

写法：
# 主题名

不规定章节。开篇说清这条要回答什么、推翻什么；正文按论证需要组织；可信度与推翻条件写进论证；结尾落一句能据以行动的判断。
```

凭什么那一行是刻意留的。判错了要能看出是规则错了还是引用错了——改规则，不改结果。

「公理」那一行只列这次判断真的用到的那几条，不是摊全表：卡上不重复展开原话，要看去跑 `meta axiom`。

## 三、判不了只问一句

两格打平，不硬猜，给一句能把两者分开的问话，然后停下。

```text
落位卡

去处：留在这里
待定
候选：Insight 洞察 2、Roadmap 路线图 2
问一句：这句话在解释为什么必须这样排（因为…否则…），还是现在就要做哪一步？前者进洞察，后者进路线图。
公理：home-before-category（怀疑）
先别落。
```

分辨句写在契约里，成对给两格。没有现成的一对时，问默认那一句：它说的是世界怎么运转，还是你下一步要做什么。

## 四、什么时候收不了

一格都套不上就不收，留在日志里等它重复。这类内容通常是单次一闪念、无行动、无后续，硬归进某一格只会让那一格多一条没人再看的记录。

给的是一篇材料而不是一条线索时，也先不收。段数超过契约里的上限，工具会说「拆成一条再判」——一次判一篇，各条的线索会互相干扰，结论一定偏。

有一类要现场判断：正文明显属于某个领域仓，但本仓十八格都套不上。这时卡上会指出去哪个领域，同时说明本仓不收；搬不搬、怎么搬，按 `meta-classify` 走。

## 五、它不做的事

不改写、不落盘、不搬文件。跨仓只给去处和形态，真正的迁移还是按 `meta-classify` 一步步走：先写目标、再删源、逐级更新指针。

## 六、改规则改文件

十八格的目录、每个格子认哪些线索、降格规则、分辨句，全在契约里，不在代码里。

```bash
qtfounder meta classify notes.md
qtfounder meta classify --stdin < 一句.md
qtfounder meta classify notes.md --json
qtfounder meta classify notes.md --rules ./my-contract.toml
```

契约缺省从源码树里读 `src/meta/default-contract.toml`，装到用户级目录后读 `~/.config/qtfounder/meta/default-contract.toml`。这份契约没有内置兜底：十八格的目录本身就在文件里，缺了无从判断归属，工具会停手报错，不猜一个去处。

## 七、退出码

`0` 有落位（留在这里或搬走），`1` 不收，`3` 待定，`2` 是读不了文件或契约。待定单独一档，是为了让脚本能停下来等人回答那一个问题，而不是当成失败重跑。

## 八、relation：看归位边

`classify` 给一张卡，`relation` 补它和邻居的关系。

```bash
qtfounder meta relation            # 全部迁移与分辨句
qtfounder meta relation insight    # 只看这一格
```

```text
insight  Insight 洞察
  迁出：
    → essay  推不翻也改不了做法，适合进论文，不进洞察。
  迁入：
    （无）
  分辨：
    ↔ roadmap
      这句话在解释为什么必须这样排（因为…否则…），还是现在就要做哪一步？前者进洞察，后者进路线图。
    ↔ profile
      ...
```

用途：

- 判完一条，想知道它以后会往哪走，看迁出；迁入回答「谁会降到这一格」。
- 两格打平，`classify` 只给一句问话；`relation` 把这一格的所有对端一次列全，方便对照。
- 一格没有任何边（如 `brochure`），说明契约里它是终点，不必在它身上找分流。

只读契约，不重新判内容，也不替你选格。

## 九、model：本体模型

```bash
qtfounder meta model
```

```text
归位本体
一份内容只有一个正确去处。十八格分两族，各成九宫格。

陈述型  组织知道什么
  过去是已履行的契约，现在是正在履行的，未来是将要履行的。
  时间：过去 / 现在 / 未来
  类型：事件 / 语义 / 自我
  九格：report、library、history、journal、profile、brochure、roadmap、insight、intention

程序型  组织怎么做事
  宪法定权责，法律写程序，法理讲方法论。
  约束力：宪法 / 法律 / 法理
  消费对象：人读 / AI 解析 / 直接运行
  九格：bylaw、specification、toolkit、handbook、gallery、platform、tutorial、essay、laboratory

领域（6）
  quanttide-write  write 领域
  ...
```

用途：

- 第一次读契约，先看轴，知道十八格为什么这样分。
- 新增或调整一格时，先确认它落在哪一族、两条轴上的位置；模型与 `family` 对不上时，契约加载就会报错。
- 和 `asset relation` 的分工：`model` 看本体的轴与族，`asset relation` 看资产在仓库里的家族与目录分组。

## 十、axiom：公理

```bash
qtfounder meta axiom
qtfounder meta axiom no-fallback
```

```text
one-place  怀疑  一份内容只有一个正确去处。
  放错格的代价不是难看，是下一轮没人找得到它。

home-before-category  怀疑  先定去处，再定范畴，再过准入，最后给写法。
  顺序换了，域错时格越准，搬得越远。
...
```

名字后面跟一个标记：确认是自己验过的，怀疑是还没验过。没写 `status` 的一律当怀疑——没验过就不当前提用。确认的排在前面，同组内保持契约里的顺序；卡片上的公理也按这个顺序点。

用途：

- 判一条内容前，先看哪些前提不再论证，省得每次重新讨论。
- 迁移或归档前，`target-then-source` 与 `archive-vs-transfer` 是操作边界。
- 写模板前，`template-scope` 说明有固定结构才有模板：手册、规格与按日期落的日志可以有，结构随内容走的不设固定写法。

公理在契约里是 `[[axiom]]`，每条写清 name、description 与 why；`status` 写 `confirmed` 或 `suspected`，缺省怀疑。name 为空、description 为空、why 为空、name 重复、status 写了别的值，都会在读契约时被拦下。只有结论的公理是口号：记不住当初的失败，后来的人就无从判断它还算不算前提。

### 卡上点哪几条

公理不参与算分，只留一行出处。判完对不上时，先看这一行：前提改了，还是规则用错了。

- 每张卡：`home-before-category`
- 指了去处：`text-only`、`archive-vs-transfer`
- 收了（落位或搬走）：`one-place`、`template-scope`
- 不收：`no-fit`

点名只点契约里还在的：把一条 `[[axiom]]` 删了，卡上就不会再出现它——沉默比报错早一步暴露改动。反过来，契约里加了新公理，卡也不会自动引用，要在 `classify` 的 `cited` 里写上它管哪一步。

## 十一、home：去处

```bash
qtfounder meta home
qtfounder meta home quanttide-write
```

```text
quanttide-write  write 领域  确认
  线索：小说、章节、文风、叙事、人物、改稿
...
```

去处是内容搬出本仓能去的地方，与落进哪一格无关。去处写在契约的 `[[home]]` 里，每条有 name、label、cues 与 status；`classify` 定去处时读同一份名单。

用途：

- 加一个去处时，先在这里登记仓名与线索；线索是认路的词，不是关键词清单。
- 去处 name 重复、为空，或 status 写了别的值，读契约时就报。

## 十二、domain：域（本体论）

```bash
qtfounder meta domain
qtfounder meta domain --json
```

```text
域：meta

范畴（18）
  report、library、history、...

属性（19）
  标识 4
  分类 2
  ...

公理（9）
  确认 1，怀疑 8
```

域是本体论的根：一个域由范畴、属性与公理组成。本仓的域叫 meta，写在契约顶层的 `domain` 里；每个命令的契约都用同一行标明自己属于哪个域。

用途：

- 想知道本仓有哪些类目、字段与前提，跑这一条。
- 域与去处是两回事：域回答「这份内容属于哪一类知识」，去处回答「它该落在哪个仓」。
- 结构视角看同一批类目，跑 `meta paradigm`；类目作为点的读法，跑 `meta model`。

## 十三、property：标准属性

```bash
qtfounder meta property
qtfounder meta property name
qtfounder meta property --json
```

```text
标识属性
  id  确认  全局唯一键，用 uuid；改名不动它，引用认它。
  name  确认  机器可读标识，小写、连字符、英文；同一类里不重复，格、家族、公理都用它。
...
```

属性定义写在契约的 `[[property]]` 里，`docs/gallery/meta/property.md` 是给人看的镜像。组内确认的排前面，没写 `status` 的按怀疑算。

用途：

- 加字段前先看这个名字有没有：有就照它的意思用，不另起一个。
- 契约里 name 重复、缺 group、缺 description、status 写了别的值，读契约时就报。

## 十四、category：十八格与两端

```bash
qtfounder meta category
qtfounder meta category insight
qtfounder meta category context
```

```text
陈述型
  report  Report 报告  data/report/
    这一轮已经结束，收的是履行完毕的契约。
...

两端
  context  Context 入口  data/context/  入口
    按「领域 / 资产」先放进来、还没通过整理校验的稿子；校验通过就搬去对应的长期记忆。
  archive  Archive 归档  data/archive/  出口
    已经退出使用的内容，只移动不改写，留一份可追溯的底。
```

十八格判类型，两端记位置：context 是还没归好类的入口，archive 是已经退出的出口。两者的「收什么」写在契约 `[[asset]]` 的 `description` 里，`classify` 不会把内容判进这两端。

`category` 是名词，看格有哪些、各收什么、落在哪个目录；`classify` 是动词，把一条内容放进某一格。要写法与分档看 `asset contract`。

`docs/gallery/meta/category.md` 是给人看的镜像，名字、族、目录与界线都与契约逐条对，两端也在其中，跑 `meta validate` 能查出漂移。

## 十五、validate：查漂移

```bash
qtfounder meta validate
qtfounder meta validate --docs docs/gallery/meta
```

```text
对得上：公理与属性都一致
```

对不上时退出码 1，逐条列出：文档多写了谁、少写了谁、状态不一致、分组不一致、定义文字不一致。文档目录不在就直接报错，不猜。

它只对五份镜像——`axioms.md`、`property.md`、`domain.md`、`category.md` 与 `paradigm.md`；模型、关系两页还没纳入。改完契约或这五份文档，跑一遍再提交。

## 十六、paradigm：把归位写成一个范式

```bash
qtfounder meta paradigm            # 有哪些范式，各自的点与边
qtfounder meta paradigm aristotle  # 只看一个
qtfounder meta paradigm --check    # 查合成断没断、有没有互相可达
```

```text
quanttide-founder  归位  确认  合成：部分  共 18 个对象 / 3 条态射
  insight  Insight 洞察  出 1 入 0
  ...
  态射：
    insight → essay  推不翻也改不了做法，适合进论文，不进洞察。
```

一个范式就是一个范畴的写法：对象、态射、合成律。归位本身是其中一个，契约里写 `source = "categories"`，对象直接取 `[[category]]`、态射取 `[[transition]]`，不另存一份。别的一并挂在 `[[paradigm]]` 下，只用于对比与吸收，不参与判域判格。

`compose` 写三档：`transitive` 要求合成闭合，缺一处就退出码 1；`partial` 是条件不满足就不断言，缺了只报个数；`none` 当图看，不查。

`--check` 另查一样：两个对象互相可达。这类两个代表同在一个同构类里，按 `one-place` 该只留一个。它报的是嫌疑，不是结论。

`docs/gallery/meta/paradigm.md` 是给人看的镜像，名字、状态、来源、合成与对象数都与契约逐条对。

## 十七、compare：比两个范式

```bash
qtfounder meta compare quanttide-founder aristotle
qtfounder meta compare quanttide-founder aristotle --json
```

```text
归位  ↔  亚里士多德十范畴

  形状    对象 18 ↔ 10；态射 3 ↔ 9；最长链 1 ↔ 1
  合成    部分（缺 0） ↔ 传递（缺 0）

  同名：无
  同称呼：无
  只在左边 18 个：report、library、...
  只在右边 10 个：substance、quantity、...

  可吸收（右边独有，名字与称呼都没对上）：
    substance  实体
      主词本身，不依附别的东西。
```

对上的条件只有两条：name 相同，或 label 相同。两条都不算同构，只算线索——是不是同一个东西，仍要人看。

可吸收清单是候选，不是建议。它只说明「对方有我这边没有对上的」，加不加、怎么加，看它能不能落进本边的轴：落得进就加；落不进，说明两边不是同一个范式，记下差异就够。

形状那一行是跨范式真正能对上的部分：点数、边数、最长链、合成是否闭合。名字对不上时，先看形状——一个只有星形边、一个没有边，说明两张表的用法本来就不同。
