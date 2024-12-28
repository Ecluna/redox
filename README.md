# 🚀 Redox - Rust 实现的 Redis 风格键值存储

一个轻量级、高性能的键值存储系统，支持多种数据类型和持久化存储。

## ✨ 特性

### 📝 数据类型
- **字符串 (String)** 🔤: 基础的键值存储
- **列表 (List)** 📜: 有序的字符串集合，支持双端操作
- **集合 (Set)** 🎯: 无序的唯一元素集合
- **哈希表 (Hash)** 📑: 字段-值对的集合
- **有序集合 (Sorted Set)** 📊: 按分数排序的成员集合

### 🛠️ 核心功能
- **数据持久化** 💾: 支持 JSON 文件存储和加载
- **密码认证** 🔐: 可选的访问控制
- **自动保存** ⏱️: 可配置的自动保存间隔
- **端口选择** 🔌: 自动端口选择（当默认端口被占用时）
- **命令行界面** 💻: 交互式命令行工具

## 📦 安装

确保你已经安装了 Rust 和 Cargo
```bash
# Windows
winget install Rustlang.Rust

# macOS
brew install rust

# Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
克隆仓库

```bash
git clone https://github.com/Ecluna/redox.git
cd redox
```
构建项目
```bash
cargo build --release
```

## 🚀 使用指南

### 🖥️ 启动服务器
#### 方式一：使用 cargo run

```bash
基本启动
cargo run -p redox-server
启用持久化
cargo run -p redox-server -- -f data.json
完整配置启动
cargo run -p redox-server -- -f data.json -i 60 -p mypassword -P 2001
```

#### 方式二：直接使用命令（推荐）
```bash
安装server
cargo install --path redox-server
启动server
redox-server
启动持久化
redox-server -f data.json
完整配置启动
redox-server -f data.json -i 60 -p mypassword -P 2001
```
服务器参数说明：
- `-f, --data-file <路径>` 📁: 指定数据文件路径
- `-i, --save-interval <秒数>` ⏲️: 自动保存间隔（默认：60秒）
- `-p, --password <密码>` 🔑: 设置访问密码
- `-P, --port <端口>` 🔌: 监听端口（默认：2001）

### 🖱️ 使用客户端
#### 方式一：使用 cargo run
```bash
默认连接2001端口
cargo run -p redox-cli
指定端口连接
cargo run -p redox-cli -- 2001
```

#### 方式二：直接使用命令（推荐）
```bash
安装cli
cargo install --path redox-cli
默认连接2001端口
redox-cli
指定端口连接
redox-cli 2001
```

## 📝 支持的命令

### 认证命令 🔐
- `AUTH password`
  - 参数：
    - password: 服务器设置的密码
  - 返回：成功返回 OK，失败返回错误信息

### 字符串命令 🔤
- `SET key value`
  - 参数：
    - key: 键名
    - value: 字符串值
  - 返回：OK

- `GET key`
  - 参数：
    - key: 键名
  - 返回：字符串值或 NIL

- `MSET key1 value1 [key2 value2 ...]`
  - 参数：
    - key value: 一个或多个键值对
  - 返回：成功设置的键值对数量

- `MGET key1 [key2 ...]`
  - 参数：
    - key: 一个或多个键名
  - 返回：对应值的数组，不存在的键返回 NIL

### 列表命令 📜
- `LPUSH key value`
  - 参数：
    - key: 列表键名
    - value: 要插入的值
  - 返回：操作后列表长度

- `RPUSH key value`
  - 参数：
    - key: 列表键名
    - value: 要插入的值
  - 返回：操作后列表长度

- `LPOP key`
  - 参数：
    - key: 列表键名
  - 返回：弹出的值或 NIL

- `RPOP key`
  - 参数：
    - key: 列表键名
  - 返回：弹出的值或 NIL

- `LRANGE key start stop`
  - 参数：
    - key: 列表键名
    - start: 起始索引（支持负数）
    - stop: 结束索引（支持负数）
  - 返回：指定范围内的值列表

### 集合命令 🎯
- `SADD key member`
  - 参数：
    - key: 集合键名
    - member: 要添加的成员
  - 返回：1 表示添加成功，0 表示成员已存在

- `SREM key member`
  - 参数：
    - key: 集合键名
    - member: 要删除的成员
  - 返回：1 表示删除成功，0 表示成员不存在

- `SMEMBERS key`
  - 参数：
    - key: 集合键名
  - 返回：集合中所有成员

- `SISMEMBER key member`
  - 参数：
    - key: 集合键名
    - member: 要检查的成员
  - 返回：1 表示存在，0 表示不存在

### 哈希表命令 📑
- `HSET key field value`
  - 参数：
    - key: 哈希表键名
    - field: 字段名
    - value: 字段值
  - 返回：1 表示新建字段，0 表示更新字段

- `HGET key field`
  - 参数：
    - key: 哈希表键名
    - field: 字段名
  - 返回：字段值或 NIL

- `HDEL key field`
  - 参数：
    - key: 哈希表键名
    - field: 字段名
  - 返回：1 表示删除成功，0 表示字段不存在

- `HGETALL key`
  - 参数：
    - key: 哈希表键名
  - 返回：所有字段和值的列表

### 有序集合命令 📊
- `ZADD key score member`
  - 参数：
    - key: 有序集合键名
    - score: 分数（浮点数）
    - member: 成员名
  - 返回：1 表示添加成功，0 表示更新分数

- `ZREM key member`
  - 参数：
    - key: 有序集合键名
    - member: 要删除的成员
  - 返回：1 表示删除成功，0 表示成员不存在

- `ZRANGE key start stop`
  - 参数：
    - key: 有序集合键名
    - start: 起始索引（支持负数）
    - stop: 结束索引（支持负数）
  - 返回：指定范围内的成员和分数

- `ZRANGEBYSCORE key min max`
  - 参数：
    - key: 有序集合键名
    - min: 最小分数
    - max: 最大分数
  - 返回：分数在指定范围内的成员和分数

### 键过期命令 ⏱️
- `EXPIRE key seconds`: 设置键的过期时间
  - 参数：
    - key: 键名
    - seconds: 过期秒数
  - 返回：1 表示成功，0 表示键不存在

- `TTL key`: 获取键的剩余生存时间
  - 参数：
    - key: 键名
  - 返回：
    - 正数：剩余秒数
    - -1：键已过期
    - -2：键不存在或没有设置过期时间

- `PERSIST key`: 移除键的过期时间
  - 参数：
    - key: 键名
  - 返回：1 表示成功，0 表示键不存在或没有过期时间

### 通用命令 🛠️
- `DEL key [key ...]`
  - 参数：
    - key: 一个或多个键名
  - 返回：成功删除的键数量

- `INFO`
  - 参数：无
  - 返回：服务器统计信息，包括：
    - keys: 键总数
    - strings: 字符串键数量
    - lists: 列表键数量
    - sets: 集合键数量
    - hashes: 哈希表键数量
    - zsets: 有序集合键数量

- `QUIT`
  - 参数：无
  - 返回：无，关闭连接

## 📁 项目结构
```
redox/
├── redox-cli/ # 命令行界面
├── redox-server/ # 服务器实现
└── redox-protocol/ # 通信协议定义
```

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件