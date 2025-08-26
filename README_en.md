# Aethaum

[简体中文](README.md) | [English](README_en.md)

**Important**: This project is still in early development and currently has no available version.

A declarative framework for writing interactive worlds for AI agents.

Aethaum allows you to define complex game worlds and AI interaction environments through simple TOML configuration, without writing complex Rust code.

## 🎯 Core Concepts

- **Declarative Configuration**: Define ECS structures using TOML
- **Logic Separation**: Separate configuration from logic, with Lua handling runtime logic
- **Automatic Translation**: TOML configurations are automatically translated into high-performance Rust code
- **Flexible Extension**: Generated Rust projects can be directly extended and customized

## 🚀 Features

- 📝 **TOML-based ECS Configuration** - Clean and readable configuration syntax
- 🐍 **Lua Script Logic** - Flexible runtime logic writing
- 🦀 **Bevy ECS Integration** - Powerful ECS engine support
- 🔥 **Hot Reload Support** - Real-time updates during development
- 🎮 **AI-Friendly** - Designed specifically for AI agent interaction

## 🏗️ Project Structure

```text
world/
├── world.toml                 # World configuration file
├── modules/                   # Module directory
│   ├── combat/                # Combat module
│   │   ├── components/        # Component definitions
│   │   ├── systems/           # System definitions
│   │   ├── events/            # Event definitions
│   │   └── entity_protos/     # Entity prototypes
│   ├── explore/               # Exploration module
│       ├── components/
│       ├── systems/
│       ├── events/
│       └── entity_protos/
├── scripts/				   # Lua Scripts
└── generated/                 # Generated Rust code directory
```

## 🧩 Modular Architecture Explanation

Aethaum uses a modular architecture that allows dividing the world into multiple functional modules. Each module contains its own components, systems, events, and entity prototypes. Modules are isolated via namespaces to avoid naming conflicts and support cross-module references.

### 📦 Module Definition

Each module is an independent directory located under `modules/`. The internal structure of a module mirrors the top-level structure, containing:

- `components/`: Components defined within the module
- `systems/`: Systems defined within the module
- `events/`: Events defined within the module
- `entity_protos/`: Entity prototypes defined within the module

### 🌐 Namespaces and References

All definitions default to belonging to the namespace of their respective module. To reference definitions from other modules, use the format `module_name::definition_name`.

```toml
# Example: Referencing the Health component from the combat module
components = ["combat::Health", "Position"]
```

References within the same module can omit the module prefix:

```toml
# Example: Referencing components within the same module
components = ["Health", "Position"]
```

### 🧾 Module Declaration

Modules used in the project and their paths are declared in `world.toml` using the `[modules]` field:

```toml
[modules]
combat = "modules/combat"
explore = "modules/explore"
```

## 📋 Configuration Details

### 🌍 world.toml - World Configuration

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
# Standard Cargo configuration
```

### ⚙️ systems/*.toml - System Definitions

```toml
# Only one system can be defined per TOML file
[normal]
name = "HealthSystem"
description = "Handles entity health updates"
category = "combat"
priority = 100 #The system with higher priority will be executed first

# Component query definitions
[[queries]]
name = "living_entities"
components_include = ["combat::Health", "Position"]
components_exclude = ["Test"]
description = "Query all living entities"

[[queries]]
name = "damaged_entities"
components_include = ["combat::Health", "Damage"]
description = "Query damaged entities"

[update]
interval = 0.1  # Update interval (seconds)

# Update condition (Lua)
condition = '''
return entity.health.value > 0
'''
# Or condition_file = "sciprt/xxx.lua"

# Update logic (Lua)
logic = '''
entity.health.value = entity.health.value - entity.damage.amount
entity.damage.amount = 0
'''
# Or logic_file = "script/xxx.lua"

# Event handlers
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

The result data for a query is Entity, which is available in "update" and all the event_handlers. You can directly accessed them in your lua code.

### 🧩 components/*.toml - Component Definitions

```toml
# Multiple components can be defined in one TOML file; the normal field does not participate in translation and serves as comments or metadata
[normal]
tags = ["combat", "stats"]
description = "Combat-related components"

# Health component
[[components]]
name = "Health"
description = "Entity health value"

[[components.fields]]
name = "value"
type = "float"
default = 100.0
description = "Current health value"

[[components.fields]]
name = "max_value"
type = "float"
default = 100.0
description = "Maximum health value"

# Position component
[[components]]
name = "Position"
description = "Entity position"

[[components.fields]]
name = "x"
type = "float"
default = 0.0
description = "X coordinate"

[[components.fields]]
name = "y"
type = "float"
default = 0.0
description = "Y coordinate"
```

Some Identifiers are **reserved** and thus can't be used as event name, like:

- AethaumSpawnEntity
- AethaumSpawnEntityResponse

### ⚡ events/*.toml - Event Definitions

```toml
# Multiple events can be defined in one TOML file; the normal field does not participate in translation and serves as comments or metadata
[normal]
tags = ["combat", "interaction"]
description = "Combat and interaction events"

# Damage event
[[events]]
name = "EntityDamaged"
description = "Entity takes damage"

[[events.fields]]
name = "damage"
type = "float"
description = "Damage value"

[[events.fields]]
name = "attacker"
type = "Entity" #Primitive Type
description = "Attacker ID"

# Heal event
[[events]]
name = "EntityHealed"
description = "Entity is healed"

[[events.fields]]
name = "amount"
type = "float"
description = "Heal amount"

[[events.fields]]
name = "healer"
type = "Entity"
description = "Healer ID"
```

### 🏗️ entity_protos/*.toml - Entity Prototypes

```toml
# Multiple entity prototypes can be defined in one TOML file; the normal field does not participate in translation and serves as comments or metadata
[normal]
tags = ["characters", "npcs"]
description = "Character entity prototypes"

# Player prototype
[[entity_protos]]
name = "Player"
components = ["combat::Health", "Position", "PlayerControlled"]
description = "Player character"

# Enemy prototype
[[entity_protos]]
name = "Enemy"
components = ["combat::Health", "Position", "AIControlled"]
description = "Enemy character"

# Item prototype
[[entity_protos]]
name = "HealthPotion"
components = ["Item", "Consumable"]
description = "Health potion"
```

## 🚀 Quick Start

- Under development, not yet available for use

## 🛠️ Development Workflow

1. **Create Module** - Create a module directory under `modules/`
2. **Define Components** - Create TOML files under `modules/{module_name}/components/`
3. **Create Systems** - Define system logic under `modules/{module_name}/systems/`
4. **Design Events** - Define event structures under `modules/{module_name}/events/`
5. **Build Prototypes** - Create entity templates under `modules/{module_name}/entity_protos/`
6. **Configure World** - Edit `world.toml` to declare modules and include all definitions
7. **Build and Run** - Use CLI tools to build and run

## 🎯 Use Cases

- 🤖 **AI Training Environments** - Provide simple interactive worlds for AI agents
- 🤖 **AI Role Playing** - Enable AI characters to gain experiences through world interactions that can be converted into memories
- 🎮 **Prototype Development** - Rapidly build small game prototypes
- 🔬 **Simulation Experiments** - Simulate complex system behaviors

## 🤝 Contributing

Issues and Pull Requests are welcome!

## 📄 License

This project is licensed under the MIT License.