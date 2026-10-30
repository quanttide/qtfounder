# 在本地造一条存得住的事件总线

`infra` 的立场是 CLI 只声明「什么触发、跑什么」，触发的执行机制属于环境。
云端总线是一种环境，本机也可以有一条：事件推给它，它排着队，
订阅端什么时候起来都收得到，机器重启也不丢。

本机这条用 Redis Streams，装在 docker 容器里，apt 一个包不动。

## 一、装

```bash
docker run -d --name qtbus -v qtbus-data:/data -p 6379:6379 \
  redis:7-alpine redis-server --appendonly yes
```

`--appendonly yes` 是持久化开关，写在卷 `qtbus-data` 里，删容器不丢。
推一条事件就是一行：

```bash
docker exec qtbus redis-cli XADD events '*' name release-published version v0.1.0
```

事件名后面可以跟任意多的字段名值对，Redis 不管那些字段是什么意思。

## 二、存得住是它的全部意义

跟命名管道比一下就清楚了。管道写进去的东西必须当场被另一头接走，
读端不在，写端就一直卡住；Redis 把事件落盘排队，读端晚来多久都行。
下面这轮是实测：

1. 没有任何订阅端在跑，推三个事件，`XLEN events` 报 3
2. `docker restart qtbus` 整个容器重启，重启后 `XLEN events` 还是 3
3. 这时候才起订阅端，积压的三条一条不少地取走

消费用消费组，取完要确认。确认之前事件一直在待处理名单里，
确认之后不再重投：

```bash
docker exec qtbus redis-cli XGROUP CREATE events fire 0 MKSTREAM    # 建组，0 表示从头收
docker exec qtbus redis-cli XREADGROUP GROUP fire s1 COUNT 10 STREAMS events '>'
docker exec qtbus redis-cli XPENDING events fire                     # 取了但没确认的
docker exec qtbus redis-cli XACK events fire <事件id>                 # 确认，不再重投
```

## 三、接到 fire 上

订阅端十行 shell：消费组取一条，把事件名交给 `fire`，然后确认。
它对 `infra` 的全部认知就是「事件名是命令行的第三个参数」。

```bash
while true; do
  out=$(docker exec qtbus redis-cli --raw \
    XREADGROUP GROUP fire s1 COUNT 1 BLOCK 1500 STREAMS events '>' 2>/dev/null)
  [ -z "$out" ] && continue
  ev=$(echo "$out" | awk '/^name$/{getline; print}')
  qtfounder infra fire "$ev"
  docker exec qtbus redis-cli XACK events fire \
    "$(echo "$out" | awk '/^[0-9]+-[0-9]+$/{print}')" >/dev/null
done
```

先照[登记](index.md)那节登两条订阅，再推事件，实跑输出是：

```text
总线交给 fire：release-published
跑订阅「rel」：…
总线交给 fire：deploy-finished
跑订阅「dep」：…
```

## 四、载荷跟着事件走

`fire` 只有一个事件名，订阅的命令是固定串，这是有意的。
云端事件天然带细节，实测下来载荷不用 `fire` 带：

```text
name   release-published
version  v0.1.0
```

总线自己就把字段带过来了，订阅端想用就读，不想用就不读。
载荷属于驱动方，跟「触发的执行机制属于环境」是同一条。
哪条订阅真需要按内容决定跑什么，在订阅端那十行 shell 里加判断，
不用动 `fire` 的签名。

## 五、为什么是 Redis

客户端是一条命令，订阅端是十行 shell，没有 SDK，没有要学的 schema。
更要紧的是它存的是不透明的字段名值对，事件名随便叫什么它都不管。

别的总线没这个性质：NATS 用 subject 命名约定，Kafka 要先想清楚
partition 和 key，两者都把解释的意思塞进了基础设施里。
「事件名不枚举」在 Redis 上天然成立，在那两个上得跟它较劲。

要更像云端总线的形状，即分区、offset 与消费组都是一等公民，
那是 Redpanda 那一类，容器大得多，概念也多一倍，接的是同一条线。

## 六、留着还是拆掉

留着是后台常驻，占约 10MB。看积压：`docker exec qtbus redis-cli XLEN events`。
拆掉连数据一起删：

```bash
docker rm -f qtbus && docker volume rm qtbus-data
```

## 七、它没测的东西

CLI 这边 `fire` 与 `run` 已有测试，这条总线验的是接线：
一个对外部一无所知的发布者，能不能推着事件驱动 `fire`。
答案是能，而且发布端不用知道有 `infra` 这个东西。

云端总线那条路仍未走通——qtcloud 的 provider 目前只有健康检查，
接线方不存在。本机这条是它的等价物，不是它的替代品。
