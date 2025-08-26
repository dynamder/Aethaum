# Aethaum

[简体中文](README.md) | [English](README_en.md)

**重要**：此项目仍处于初期开发阶段，目前没有可用版本



一个用于编写AI智能体可交互世界的声明式框架

Aethaum 让你能够通过简单的 TOML 配置来定义复杂的游戏世界和AI交互环境，无需编写复杂的Rust代码。

## 🎯 核心理念

- **声明式配置**：使用 TOML 定义 ECS 结构
- **逻辑分离**：配置与逻辑分离，Lua 处理运行时逻辑
- **自动转译**：TOML 配置自动转译为高性能 Rust 代码
- **灵活扩展**：生成的 Rust 项目可直接扩展和定制

## 🚀 特性

- 📝 **基于 TOML 的 ECS 配置** - 简洁易读的配置语法
- 🐍 **Lua 脚本逻辑** - 灵活的运行时逻辑编写
- 🦀 **Bevy ECS 集成** - 强大的 ECS 引擎支持
- 🔥 **热重载支持** - 开发时实时更新
- 🎮 **AI 友好** - 专为 AI 智能体交互设计

## 🏗️ 项目架构

```text
world/
├── world.toml                 # 世界配置文件
├── modules/                   # 模块目录
│   ├── combat/                # 战斗模块
│   │   ├── components/        # 组件定义
│   │   ├── systems/           # 系统定义
│   │   ├── events/            # 事件定义
│   │   └── entity_protos/     # 实体原型
│   ├── explore/               # 探索模块
│       ├── components/
│       ├── systems/
│       ├── events/
│       └── entity_protos/
├── scripts/				   # Lua脚本	
└── generated/                 # 生成的Rust代码目录
```

## 🧩 模块化架构说明

Aethaum 使用模块化架构，允许将世界划分为多个功能模块，每个模块包含自己的组件、系统、事件和实体原型。模块通过命名空间进行隔离，避免命名冲突，并支持跨模块引用。

### 📦 模块定义

每个模块是一个独立的目录，位于 `modules/` 下。模块内部结构与顶层结构一致，包含：

- `components/`：模块内定义的组件
- `systems/`：模块内定义的系统
- `events/`：模块内定义的事件
- `entity_protos/`：模块内定义的实体原型

### 🌐 命名空间与引用

所有定义默认属于其所在模块的命名空间。引用其他模块的定义时，需使用 `模块名::定义名` 的格式。

```toml
# 示例：引用 combat 模块的 Health 组件
components = ["combat::Health", "Position"]
```

同一模块内的引用可省略模块前缀：

```toml
# 示例：引用本模块的组件
components = ["Health", "Position"]
```

### 🧾 模块声明

在 `world.toml` 中通过 `[modules]` 字段声明项目使用的模块及其路径：

```toml
[modules]
combat = "modules/combat"
explore = "modules/explore"
```

## 📋 配置详解

### 🌍 world.toml - 世界配置

```toml
[world]
name = "MyAIWorld"
version = "0.1.0"
author = "Your Name"

[modules]
combat = "modules/combat"
explore = "modules/explore"

[build]
output_dir = "generated"

[cargo]
# 标准 Cargo 配置
```

### ⚙️ systems/*.toml - 系统定义

```toml
#一个toml文件中，只能定义一个系统
[normal]
name = "HealthSystem"
description = "处理实体健康值更新"
category = "combat"
priority = 100 #优先级越高，越先执行

# 组件查询定义
[[queries]]
name = "living_entities"
components_include = ["combat::Health", "Position"]
components_exclude = ["Test"]
description = "查询所有存活实体"

[[queries]]
name = "damaged_entities"
components_include = ["combat::Health", "Damage"]
description = "查询受伤实体"

[update]
interval = 0.1  # 更新间隔(秒)

# 更新条件 (Lua)
condition = '''
return entity.health.value > 0
'''
#或者，condition_file = "script/xxx.lua"

# 更新逻辑 (Lua)
logic = '''
entity.health.value = entity.health.value - entity.damage.amount
entity.damage.amount = 0
'''
#或者，logic_file = "script/xxx.lua"

# 事件处理器
[[event_handlers]]
watch_for = "EntityDamaged"
priority = 10
logic = '''
entity.health.value = entity.health.value - event.damage
'''

[[event_handlers]]
watch_for = "EntityHealed"
priority = 20
logic_file = "scripts/entity_heal.lua"
```

在系统中，queries得到的数据为实体的id，这些实体在每个事件处理器中都可以使用。在lua脚本中，使用[[queries]]的name字段的同名变量，可以访问实体。

### 🧩 components/*.toml - 组件定义

```toml
#在一个toml文件中可以定义多个组件，normal字段不会参与转译，作为注释和元信息提供
[normal]
tags = ["combat", "stats"]
description = "战斗相关组件"

# 健康组件
[[components]]
name = "Health"
description = "实体健康值"

[[components.fields]]
name = "value"
type = "float"
default = 100.0
description = "当前健康值"

[[components.fields]]
name = "max_value"
type = "float"
default = 100.0
description = "最大健康值"

# 位置组件
[[components]]
name = "Position"
description = "实体位置"

[[components.fields]]
name = "x"
type = "float"
default = 0.0
description = "X坐标"

[[components.fields]]
name = "y"
type = "float"
default = 0.0
description = "Y坐标"
```

### ⚡ events/*.toml - 事件定义

```toml
#在一个toml文件中可以定义多个事件，normal字段不会参与转译，作为注释和元信息提供
[normal]
tags = ["combat", "interaction"]
description = "战斗和交互事件"

# 伤害事件
[[events]]
name = "EntityDamaged"
description = "实体受到伤害"

[[events.fields]]
name = "damage"
type = "float"
description = "伤害值"

[[events.fields]]
name = "attacker"
type = "Entity" #原生类型
description = "攻击者ID"

# 治疗事件
[[events]]
name = "EntityHealed"
description = "实体被治疗"

[[events.fields]]
name = "amount"
type = "float"
description = "治疗量"

[[events.fields]]
name = "healer"
type = "Entity"
description = "治疗者ID"
```

有一些标识**不可以**作为事件名称，它们被Aethaum保留，例如：

- AethaumSpawnEntity
- AethaumSpawnEntityResponse

### 🏗️ entity_protos/*.toml - 实体原型

```toml
#在一个toml文件中可以定义多个实体原型，normal字段不会参与转译，作为注释和元信息提供
[normal]
tags = ["characters", "npcs"]
description = "角色实体原型"

# 玩家原型
[[entity_protos]]
name = "Player"
components = ["combat::Health", "Position", "PlayerControlled"]
description = "玩家角色"

# 敌人原型
[[entity_protos]]
name = "Enemy"
components = ["combat::Health", "Position", "AIControlled"]
description = "敌人角色"

# 物品原型
[[entity_protos]]
name = "HealthPotion"
components = ["Item", "Consumable"]
description = "治疗药水"
```

## 🚀 快速开始

- 正在开发中，目前不处于可用阶段

## 🛠️ 开发流程

1. **创建模块** - 在 `modules/` 下创建模块目录
2. **定义组件** - 在 `modules/{module_name}/components/` 下创建 TOML 文件
3. **创建系统** - 在 `modules/{module_name}/systems/` 下定义系统逻辑
4. **设计事件** - 在 `modules/{module_name}/events/` 下定义事件结构
5. **构建原型** - 在 `modules/{module_name}/entity_protos/` 下创建实体模板
6. **配置世界** - 编辑 `world.toml` 声明模块并包含所有定义
7. **构建运行** - 使用 CLI 工具构建和运行

## 🎯 适用场景

- 🤖 **AI 训练环境** - 为 AI 智能体提供简单交互世界
- 🤖 **AI 角色扮演** - 通过与世界交互，为 AI 角色提供可以转化为记忆的经历
- 🎮 **原型开发** - 快速构建小游戏原型
- 🔬 **模拟实验** - 复杂系统行为模拟

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT