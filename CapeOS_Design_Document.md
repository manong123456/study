# CapeOS 系统基础模块设计文档

> **参考实现**: CasaOS (https://github.com/IceWhaleTech/CasaOS.git, commit `63f0148`)
> **参考公共库**: CasaOS-Common (https://github.com/IceWhaleTech/CasaOS-Common.git, commit `909dcbd`)
> **架构参考**: https://wiki.casaos.io/en/contribute/development
> **编写日期**: 2026-03-11

---

## 目录

1. [项目概述](#1-项目概述)
2. [系统架构](#2-系统架构)
3. [系统功能模块总览](#3-系统功能模块总览)
4. [各功能模块详细介绍](#4-各功能模块详细介绍)
5. [API 接口梳理](#5-api-接口梳理)
6. [代码框架结构](#6-代码框架结构)
7. [后端数据库实现](#7-后端数据库实现)

---

## 1. 项目概述

CapeOS 定义为运行边缘 AI 平台的操作系统，边缘 AI 时代的底层架构，实现了端侧物联网的深度集成与本地化连接和控制。通过开放的系统级扩展与核心执行引擎，致力于降低 AI 开发门槛，为用户打造性能卓越的端侧应用中间层。

CapeOS 采用 **微服务网关架构**，由多个独立的 Rust 服务组成，通过 CapeOS-Gateway 统一对外提供服务，通过 CapeOS-MessageBus 实现跨服务事件通信。前后端均使用 Rust 实现，前端编译为 WebAssembly 运行在浏览器中，后端以原生二进制运行。

### 核心技术栈

| 类别 | 技术选型 | Crate / 工具 | 许可证 |
|------|----------|-------------|--------|
| 编程语言 | Rust (Edition 2024) | rustc 1.85+ | -- |
| 前端框架 | Leptos (全栈 Rust, SSR + WASM) | `leptos` 0.7+ | MIT |
| 前端 UI 组件 | Thaw UI | `thaw` | MIT |
| 前端样式 | TailwindCSS | `tailwindcss` | MIT |
| 前端构建 | cargo-leptos | `cargo-leptos` | MIT |
| 后端 Web 框架 | Axum | `axum` 0.8+ | MIT |
| 异步运行时 | Tokio | `tokio` | MIT |
| 数据库 | SQLite (rusqlite) | `rusqlite` 0.38+ | MIT |
| 数据库异步封装 | tokio-rusqlite | `tokio-rusqlite` 0.7+ | MIT |
| 数据库迁移 | rusqlite_migration | `rusqlite_migration` | Apache-2.0 |
| API 文档生成 | utoipa + utoipa-axum | `utoipa` 5.0+ | MIT/Apache-2.0 |
| 序列化 | serde + serde_json | `serde` | MIT/Apache-2.0 |
| JWT 认证 | jsonwebtoken + p256 (ECDSA) | `jsonwebtoken`, `p256` | MIT |
| 系统信息采集 | sysinfo | `sysinfo` | MIT |
| systemd 交互 | zbus_systemd | `zbus_systemd` | MIT |
| 定时任务 | tokio-cron-scheduler | `tokio-cron-scheduler` | MIT/Apache-2.0 |
| HTTP 客户端 | reqwest | `reqwest` | MIT/Apache-2.0 |
| 内存缓存 | moka | `moka` | MIT/Apache-2.0 |
| 日志/追踪 | tracing + tracing-subscriber | `tracing` | MIT |
| 文件压缩 | zip / tar / flate2 | `zip`, `tar`, `flate2` | MIT |
| 图片处理 | image | `image` | MIT/Apache-2.0 |
| SMB/CIFS | pavao | `pavao` | LGPL-3.0 |
| 云存储 OAuth | oauth2 | `oauth2` | MIT/Apache-2.0 |
| Docker 管理 | bollard | `bollard` | Apache-2.0 |
| 反向代理 | axum-reverse-proxy | `axum-reverse-proxy` | MIT |
| 配置文件 | TOML | `toml`, `serde` | MIT/Apache-2.0 |
| 密码哈希 | argon2 | `argon2` | MIT/Apache-2.0 |
| UUID | uuid | `uuid` | MIT/Apache-2.0 |
| 消息总线 | Tokio broadcast + Axum WebSocket | `tokio::sync::broadcast` | MIT |
| 磁盘合并 | MergerFS (外部进程调用) | `tokio::process` | MIT |
| 服务管理 | systemd | `zbus_systemd` | MIT |
| 构建/发布 | cargo + cross | `cross` | MIT/Apache-2.0 |
| 目标架构 | amd64 / arm64 / armv7 | -- | -- |

---

## 2. 系统架构

### 2.1 整体架构图

CapeOS 采用模块化微服务架构，所有服务通过 Gateway 统一对外暴露，通过 MessageBus 实现事件驱动通信。前端使用 Leptos 编译为 WASM，通过 SSR + Hydration 模式在服务端渲染后交由浏览器接管。

```text
+==================================================================================+
|                            用户浏览器                                             |
|                  Leptos WASM (SSR Hydration 后由浏览器接管)                        |
|                  UI 组件: Thaw UI + TailwindCSS                                  |
+====================================+=============================================+
                                     | HTTP / WebSocket / Server Functions
                                     v
+==================================================================================+
|                                                                                  |
|                        CapeOS-Gateway  (端口 80)                                 |
|                     唯一对外暴露的网络入口 / 反向代理                              |
|                     实现: Axum + axum-reverse-proxy                               |
|                                                                                  |
|    +-------------------------------------------------------------------------+   |
|    |                       动态路由表 (RwLock<HashMap>)                       |   |
|    |                                                                         |   |
|    |  /v1/user_service/*   --> http://127.0.0.1:<port_A>                     |   |
|    |  /v1/local_storage/*  --> http://127.0.0.1:<port_B>                     |   |
|    |  /v1/app_management/* --> http://127.0.0.1:<port_C>                     |   |
|    |  /v1/message_bus/*    --> http://127.0.0.1:<port_D>                     |   |
|    |  /v1/sys, /v1/file... --> http://127.0.0.1:<port_E>  (CapeOS主服务)    |   |
|    |  /v1/capeos/*         --> http://127.0.0.1:<port_E>                     |   |
|    |  /v1/file             --> http://127.0.0.1:<port_E>                     |   |
|    +-------------------------------------------------------------------------+   |
|                                                                                  |
|    Management API (内部端口, 仅 localhost):                                      |
|      POST /v1/gateway/routes   <-- 各服务启动时注册路由                           |
|      GET  /v1/gateway/port     <-- 查询网关端口                                  |
|      PUT  /v1/gateway/port     <-- 修改网关端口                                  |
|                                                                                  |
+==================================================================================+
          |              |             |              |              |
          v              v             v              v              v
+--------------+ +--------------+ +------------+ +------------+ +------------------+
| CapeOS       | | UserService  | | Local      | | App        | | MessageBus       |
| 主服务       | |              | | Storage    | | Management | |                  |
| (Leptos+Axum)| | (Axum)       | | (Axum)     | | (Axum)     | | (Axum)           |
|              | |              | |            | |            | |                  |
| 前端SSR+WASM | | 用户认证     | | 磁盘管理   | | Docker应用 | | 事件/动作        |
| 文件管理     | | JWT签发(p256)| | USB管理    | | 应用商店   | | 发布/订阅        |
| 系统监控     | | JWKS端点     | | MergerFS   | | 容器生命   | |                  |
| Samba共享    | | 账号管理     | | 分区管理   | | 周期管理   | | REST + WS        |
| 云存储挂载   | | (argon2)     | |            | | (bollard)  | | + Unix Socket    |
|              | |              | |            | |            | |                  |
| 127.0.0.1:? | | 127.0.0.1:? | |127.0.0.1:? | |127.0.0.1:?| | 127.0.0.1:?      |
| DB:rusqlite  | | DB:rusqlite  | |DB:rusqlite | |DB:rusqlite | | (内存存储)       |
+--------------+ +--------------+ +------------+ +------------+ +------------------+
       |                |               |              |                 |
       +----------------+---------------+--------------+-----------------+
                                        |
                              +---------v----------+
                              |  capeos-common     |
                              |  (Cargo crate)     |
                              |                    |
                              | - Gateway 客户端   | reqwest -> POST /v1/gateway/routes
                              | - JWT 验证工具     | jsonwebtoken + p256
                              | - MessageBus 客户端| tokio UnixStream
                              | - 服务发现         | 读取 /var/run/capeos/*.url
                              | - 共享模型/工具    | serde models
                              | - Axum 中间件      | tower-http (CORS/Compression)
                              +--------------------+
```

### 2.2 微服务组件列表

| 服务 | Cargo crate 名 | 二进制文件 | 核心依赖 | 职责 |
|------|----------------|-----------|----------|------|
| **CapeOS-Gateway** | `capeos-gateway` | `capeos-gateway` | axum, axum-reverse-proxy | 反向代理网关, 唯一对外暴露端口(80), 动态路由 |
| **CapeOS** (主服务) | `capeos-main` | `capeos` | leptos, leptos_axum, axum, rusqlite, sysinfo | 前端 SSR+WASM, 文件管理, 系统监控, Samba, 云存储 |
| **CapeOS-UserService** | `capeos-user-service` | `capeos-user-service` | axum, rusqlite, jsonwebtoken, p256, argon2 | 用户注册/登录, JWT 签发, JWKS 公钥 |
| **CapeOS-LocalStorage** | `capeos-local-storage` | `capeos-local-storage` | axum, sysinfo, tokio::process | 磁盘管理, USB 挂载, MergerFS |
| **CapeOS-AppManagement** | `capeos-app-management` | `capeos-app-management` | axum, bollard, rusqlite | Docker 应用生命周期, 应用商店 |
| **CapeOS-MessageBus** | `capeos-message-bus` | `capeos-message-bus` | axum, tokio::sync::broadcast | 事件/动作 发布订阅, WebSocket 推送 |
| **CapeOS-Common** | `capeos-common` | (库 crate, 无二进制) | reqwest, jsonwebtoken, serde, tower-http | 共享库: Gateway 客户端, JWT, 中间件, 服务发现 |
| **CapeOS-CLI** | `capeos-cli` | `capeos-cli` | clap, reqwest | 命令行诊断和测试工具 |

### 2.3 服务启动顺序与依赖

```text
capeos-gateway                          <-- 最先启动, 绑定端口 80
    |
    +-- capeos-message-bus              <-- 依赖 Gateway
    |       |
    |       +-- capeos-user-service     <-- 依赖 MessageBus
    |       |
    |       +-- capeos-local-storage    <-- 依赖 MessageBus
    |       |
    |       +-- capeos-app-management   <-- 依赖 MessageBus + Docker
    |       |
    |       +-- capeos (主服务)         <-- 依赖 MessageBus + rclone
    |
    +-- (所有服务启动时向 Gateway 注册路由)
```

所有服务以 `systemd` 方式管理，使用 `Type=notify` 通知就绪 (通过 `sd-notify` crate)，`Restart=always` 保证高可用。

### 2.4 服务发现机制

CapeOS 采用 **基于文件的服务发现** 模式，运行时目录为 `/var/run/capeos/`:

| 文件名 | 写入者 | 内容 | 读取者 |
|--------|--------|------|--------|
| `management.url` | Gateway | 管理 API 地址 (如 `http://127.0.0.1:34703`) | 所有服务 (注册路由) |
| `gateway.url` | Gateway | 网关监听地址 (如 `http://[::]:80`) | -- |
| `message-bus.url` | MessageBus | MessageBus API 地址 | 所有服务 (发布事件) |
| `user-service.url` | UserService | UserService 地址 | CapeOS (获取 JWKS 公钥) |
| `app-management.url` | AppManagement | AppManagement 地址 | capeos-common |
| `capeos.url` | CapeOS 主服务 | CapeOS 主服务地址 | 其他服务 (发送通知) |

服务发现实现位于 `capeos-common` crate:

```rust
/// 从 URL 文件读取服务地址
pub async fn get_service_address(runtime_path: &Path, filename: &str) -> Result<String> {
    let path = runtime_path.join(filename);
    tokio::fs::read_to_string(&path).await.map(|s| s.trim().to_string())
}

/// 等待服务就绪 (重试机制)
pub async fn wait_for_service(runtime_path: &Path, filename: &str, retries: u32) -> Result<String> {
    for i in 0..retries {
        if let Ok(addr) = get_service_address(runtime_path, filename).await {
            if ping(&addr).await.is_ok() {
                return Ok(addr);
            }
        }
        tracing::info!("waiting for {} ({}/{})", filename, i + 1, retries);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Err(anyhow!("{} not available after {} retries", filename, retries))
}
```

### 2.5 认证架构

```text
用户登录
    |
    |  POST /v1/user_service/users/login
    v
+------------------+
|  UserService     |
|  (Axum)          |
|  1. argon2 验证  |
|  2. p256 签 JWT  |--->  ECDSA P-256 签名
|  3. 返回 Token   |
+------------------+
         |
         |  JWT Token
         v
+----------------------------------------------+
|  后续请求: Authorization: Bearer <JWT>       |
|                                              |
|  各服务的 Axum JWT 中间件 (capeos-common):   |
|  1. 从 Header 或 Query 中提取 token          |
|  2. reqwest 从 UserService JWKS 端点获取公钥 |
|     GET http://<user-service>/.well-known/jwks.json |
|  3. jsonwebtoken 用 ECDSA 公钥验证签名       |
|  4. localhost 请求跳过认证                   |
+----------------------------------------------+
```

### 2.6 事件驱动通信

```text
                         CapeOS-MessageBus
                    (Axum + tokio::broadcast)
                    +--------------------------+
                    |                          |
  发布事件 -------->|--> Event Store           |
  (REST / Unix Socket)                        |
                    |    +--------------+      |
                    |    | Event Types  |      |
                    |    | Action Types |      |
                    |    +--------------+      |
                    |           |              |
  订阅事件 -------->|--> WebSocket 推送 ------>|--> 前端 / 其他服务
  (axum::extract::ws)                         |
                    |                          |
                    +--------------------------+

事件发布方式:
  1. REST API:  POST /v1/message_bus/event/{source_id}/{name}
  2. Unix Socket: tokio::net::UnixStream -> /tmp/message-bus.sock
```

CapeOS 主服务注册的事件类型:

| 事件名称 | Source ID | 说明 |
|----------|-----------|------|
| `capeos:system:utilization` | `capeos` | 系统资源利用率 (每 5 秒推送) |
| `capeos:file:recover` | `capeos` | 云存储 OAuth 回调完成 |
| `capeos:file:operate` | `capeos` | 文件操作进度 (复制/移动) |

### 2.7 完整请求流程示例

以 `GET /v1/capeos/health/services` 为例:

```text
1. 浏览器发送请求
   GET http://192.168.1.100:80/v1/capeos/health/services
   Header: Authorization: Bearer eyJhbGciOiJFUzI1NiI...

2. CapeOS-Gateway (端口 80) 接收请求
   +-- axum-reverse-proxy 匹配路由表: /v1/capeos -> http://127.0.0.1:43821
   +-- 反向代理转发到 CapeOS 主服务

3. CapeOS 主服务 (127.0.0.1:43821) 处理请求
   +-- Axum Router::nest() 匹配到 /v1/capeos 路由组
   +-- tower-http CorsLayer 中间件
   +-- JWT 中间件 (capeos-common)
   |   +-- 提取 Authorization Header 中的 Token
   |   +-- reqwest GET http://<user-service>/.well-known/jwks.json 获取公钥
   |   +-- jsonwebtoken ECDSA 验签成功, 提取 user_id
   +-- utoipa OpenAPI 路由
   +-- get_health_services Handler
       +-- zbus_systemd 列出所有 capeos-* 服务状态
       +-- 返回 JSON: { running: [...], not_running: [...] }

4. 响应原路返回: CapeOS -> Gateway -> 浏览器
```

---

## 3. 系统功能模块总览

CapeOS 的功能分布在多个微服务中，以下是按服务划分的模块概览:

```text
CapeOS 生态系统
|
+-- CapeOS-Gateway ---- 网关与路由管理
|   实现: Axum + axum-reverse-proxy + RwLock<HashMap> 路由表
|
+-- CapeOS (主服务)
|   前端: Leptos SSR + WASM Hydration + Thaw UI
|   后端: Axum + rusqlite
|   |
|   +-- 系统管理模块 (System)
|   |   +-- 硬件信息采集 (CPU/内存/磁盘/网络) -- sysinfo crate
|   |   +-- 系统资源实时监控 -- tokio-cron-scheduler + MessageBus
|   |   +-- 版本管理与在线更新
|   |   +-- 电源控制 (重启/关机) -- zbus_systemd
|   |   +-- SSH WebSocket 终端 -- axum::extract::ws
|   |   +-- 日志管理 -- tracing
|   |
|   +-- 文件管理模块 (File)
|   |   +-- 文件/目录浏览与操作 -- tokio::fs
|   |   +-- 分块文件上传 -- axum::extract::Multipart
|   |   +-- 单文件/批量打包下载 -- zip / tar / flate2
|   |   +-- 文件复制/移动 (异步任务) -- tokio::spawn
|   |   +-- 图片缩略图生成 -- image crate
|   |   +-- WebSocket 实时通信 -- axum::extract::ws
|   |
|   +-- 网络存储模块 (Samba/CIFS)
|   |   +-- Samba 本地共享管理 -- rusqlite + smb.conf 生成
|   |   +-- 远程 CIFS 连接管理 -- pavao (libsmbclient)
|   |   +-- 网络挂载自动恢复 -- 启动时 tokio::spawn 恢复
|   |
|   +-- 云存储模块 (Cloud Storage)
|   |   +-- Google Drive / OneDrive / Dropbox -- oauth2 crate
|   |   +-- OAuth 2.0 认证回调
|   |   +-- rclone 挂载管理 -- tokio::process::Command
|   |
|   +-- 通知模块 (Notify) -- rusqlite + MessageBus
|   +-- 健康检查模块 (Health) -- zbus_systemd + /proc/net
|   +-- ZeroTier 网络模块 -- reqwest 代理转发
|   +-- 设备发现模块 (Peer) -- rusqlite
|
+-- CapeOS-UserService ---- 用户管理与认证
|   实现: Axum + rusqlite + jsonwebtoken + p256 + argon2
|
+-- CapeOS-LocalStorage ---- 本地存储管理
|   实现: Axum + sysinfo + tokio::process (MergerFS)
|
+-- CapeOS-AppManagement ---- 应用管理
|   实现: Axum + bollard (Docker API) + rusqlite
|
+-- CapeOS-MessageBus ---- 消息总线
    实现: Axum + tokio::sync::broadcast + tokio::net::UnixListener
```

---

## 4. 各功能模块详细介绍

### 4.1 CapeOS-Gateway -- 网关服务

Gateway 是整个 CapeOS 系统的唯一网络入口，实现动态 API 路由。

**实现方案**: Axum + `axum-reverse-proxy` crate

**工作原理**:

1. Gateway 启动后绑定外部端口 (默认 80) 和内部管理端口 (随机)
2. 将管理端口地址写入 `/var/run/capeos/management.url`
3. 其他微服务启动时读取该文件，通过 `reqwest` 调用 `POST /v1/gateway/routes` 注册自身路由
4. Gateway 维护 `RwLock<HashMap<String, String>>` 动态路由表，根据请求路径前缀转发

**路由注册模型** (定义在 `capeos-common`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub path: String,    // 路由前缀, 如 "/v1/capeos"
    pub target: String,  // 后端地址, 如 "http://127.0.0.1:43821"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePortRequest {
    pub port: String,
}
```

**Gateway 客户端接口** (定义在 `capeos-common`):

```rust
#[async_trait]
pub trait GatewayClient: Send + Sync {
    async fn register_route(&self, route: &Route) -> Result<()>;
    async fn change_port(&self, port: &str) -> Result<()>;
    async fn get_port(&self) -> Result<String>;
}
```

### 4.2 CapeOS 主服务 -- 系统管理模块

**源文件**: `crates/capeos-main/src/server/services/system.rs`

提供系统级信息采集和管理控制功能。

| 功能 | Rust 方法 | 实现方式 |
|------|-----------|----------|
| CPU 使用率 | `get_cpu_percent()` | `sysinfo::System::global_cpu_usage()` |
| CPU 信息 | `get_cpu_info()` | `sysinfo::System::cpus()` |
| CPU 核心数 | `get_cpu_core_num()` | `sysinfo::System::physical_core_count()` |
| CPU 温度 | `get_cpu_temperature()` | `sysinfo::Components` 或读取 sysfs thermal_zone |
| CPU 功耗 | `get_cpu_power()` | `tokio::fs::read_to_string` 读取 intel-rapl |
| 内存信息 | `get_mem_info()` | `sysinfo::System::total_memory()` / `used_memory()` |
| 磁盘信息 | `get_disk_info()` | `sysinfo::Disks::new_with_refreshed_list()` |
| 网络统计 | `get_net_info()` | `sysinfo::Networks::new_with_refreshed_list()` |
| 物理网卡 | `get_net_cards()` | 读取 `/sys/class/net/` 或 `tokio::process::Command` |
| 主机信息 | `get_sys_info()` | `sysinfo::System::host_name()` / `os_version()` |
| 设备信息 | `get_device_info()` | 聚合 IP/端口/主机名/设备型号/Hash |
| 系统更新 | `update_system_version()` | `tokio::process::Command::new("curl")` |
| 重启 | `system_reboot()` | `zbus_systemd` 调用 logind Reboot |
| 关机 | `system_shutdown()` | `zbus_systemd` 调用 logind PowerOff |
| 日志 | `get_capeos_logs()` | `tokio::fs::read_to_string()` |
| 系统入口 | `get_system_entry()` | 读取各模块 `entry.json` 聚合 |
| 目录操作 | `get_dir_path()` / `mkdir_all()` / `rename_file()` | `tokio::fs` 异步文件操作 |

**定时任务**: 通过 `tokio-cron-scheduler` 每 5 秒执行系统信息采集，经由 MessageBus 的 Unix Socket (`tokio::net::UnixStream`) 发布 `capeos:system:utilization` 事件，前端 Leptos 组件通过 WebSocket 订阅实现实时仪表盘。

### 4.3 CapeOS 主服务 -- 文件管理模块

**源文件**: `crates/capeos-main/src/server/services/file_ops.rs`, `upload.rs`

| 功能 | 说明 | Rust 实现 |
|------|------|-----------|
| 目录浏览 | 列出文件/目录, 返回名称/大小/类型/修改时间 | `tokio::fs::read_dir()` + `metadata()` |
| 分块上传 | 追踪上传状态, 写入 `.tmp` 文件, 完成后重命名 | `axum::extract::Multipart` + `DashMap` |
| 文件下载 | 单文件流式响应; 批量打包下载 | `axum::body::Body::from_stream()` + `zip`/`tar` |
| 文件创建 | 创建空文件, 检测路径冲突 | `tokio::fs::File::create()` |
| 文件编辑 | 读取/写入文件内容 | `tokio::fs::read_to_string()` / `write()` |
| 重命名 | 文件和目录的重命名 | `tokio::fs::rename()` |
| 删除 | 支持批量删除 | `tokio::fs::remove_file()` / `remove_dir_all()` |
| 复制/移动 | 异步执行, 进度追踪, 支持取消 | `tokio::spawn` + `CancellationToken` |
| 目录大小 | 递归计算目录大小 | `tokio::fs` 递归遍历 |
| 图片缩略图 | 缩略图生成 + EXIF 方向校正 | `image` crate + `kamadak-exif` |
| WebSocket | 对等设备实时通信 | `axum::extract::ws::WebSocket` |

**文件操作进度模型**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperate {
    pub op_type: FileOpType,              // Copy / Move
    pub items: Vec<FileItem>,             // 源文件列表
    pub total_size: u64,
    pub processed_size: u64,
    pub destination: String,              // 目标路径
    pub conflict_policy: ConflictPolicy,  // Skip / Overwrite / Rename
    pub finished: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOpType { Copy, Move }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictPolicy { Skip, Overwrite, Rename }
```

### 4.4 CapeOS 主服务 -- 网络存储模块 (Samba/CIFS)

**源文件**: `crates/capeos-main/src/server/services/shares.rs`, `connections.rs`

**Samba 共享管理 (SharesService)**:

- CRUD 操作管理本机 Samba 共享目录, 通过 `tokio-rusqlite` 存储在 SQLite `o_shares` 表
- 生成 `/etc/samba/smb.cape.conf` 配置文件, 通过 `include` 引入主 `smb.conf`
- 配置变更后通过 `tokio::process::Command` 执行 `systemctl restart smbd`
- 支持匿名访问设置

**网络连接管理 (ConnectionsService)**:

- CRUD 操作管理远程 CIFS/SMB 连接, 通过 `tokio-rusqlite` 存储在 SQLite `o_connections` 表
- 使用 `pavao` crate (libsmbclient 绑定) 验证连接凭据
- 使用 `nix::mount::mount()` 挂载远程共享到本地目录
- 启动时通过 `tokio::spawn` 异步恢复之前的网络挂载

### 4.5 CapeOS 主服务 -- 云存储模块

**源文件**: `crates/capeos-main/src/server/drivers/`

通过 `rclone` (外部进程) 实现云存储到本地文件系统的挂载。

| 云服务 | 认证 | Rust 实现 |
|--------|------|-----------|
| Google Drive | OAuth 2.0 | `oauth2` crate + `reqwest` |
| OneDrive | OAuth 2.0 | `oauth2` crate + `reqwest` |
| Dropbox | OAuth 2.0 | `oauth2` crate + `reqwest` |

**驱动 trait**:

```rust
#[async_trait]
pub trait CloudDriver: Send + Sync {
    fn name(&self) -> &str;
    fn icon(&self) -> &str;
    async fn auth_url(&self) -> Result<String>;
    async fn handle_callback(&self, code: &str) -> Result<CloudCredential>;
    async fn get_user_info(&self, cred: &CloudCredential) -> Result<CloudUserInfo>;
}
```

**StorageService trait**:

```rust
#[async_trait]
pub trait StorageService: Send + Sync {
    async fn mount_storage(&self, mount_point: &str, fs: &str) -> Result<()>;
    async fn unmount_storage(&self, mount_point: &str) -> Result<()>;
    async fn list_storages(&self) -> Result<Vec<StorageInfo>>;
    async fn create_config(&self, data: HashMap<String, String>, name: &str, t: &str) -> Result<()>;
    async fn check_and_mount_all(&self) -> Result<()>;
    async fn delete_config_by_name(&self, name: &str) -> Result<()>;
}
```

**OAuth 回调流程**:

1. 前端引导用户到云服务 OAuth 页面
2. 回调到 `GET /v1/recover/:type`
3. Handler 创建 rclone 配置 (`tokio::process::Command`), 挂载云存储
4. 通过 `tokio::net::UnixStream` 发布 `capeos:file:recover` 事件通知前端

### 4.6 CapeOS 主服务 -- 其他模块

**通知模块 (NotifyService)**: CRUD 管理通知记录 (`o_notify` 表, rusqlite), 通过 MessageBus Unix Socket 发布 `capeos:file:operate` 事件通知文件操作进度。

**健康检查模块 (HealthService)**: 通过 `zbus_systemd` 列出所有 `capeos-*` 服务运行状态; 通过读取 `/proc/net/tcp` 和 `/proc/net/udp` 获取端口占用; 打包日志下载 (`zip` crate)。

**ZeroTier 模块**: 通过 `reqwest` 将请求代理转发到本地 ZeroTier 服务 API; 查询节点信息和网络状态。

**设备发现模块 (PeerService)**: 从 Axum 请求头中提取 User-Agent 和 IP, 管理对等设备记录 (`peer_drives` 表, rusqlite)。

**CapeService**: 通过 `reqwest` 从远程 API 获取最新版本信息, 使用 `moka` 缓存 20 分钟。

**OtherService**: 搜索引擎代理, 通过 `reqwest` + `tokio::join!` 并行查询多个搜索引擎。

### 4.7 CapeOS-UserService -- 用户管理与认证

**实现**: Axum + rusqlite + `jsonwebtoken` + `p256` + `argon2`

**API 基路径**: `/v1/user_service`

| 功能 | 说明 | Rust 实现 |
|------|------|-----------|
| 用户注册 | 创建新用户账号 | `argon2` 哈希密码, `rusqlite` 存储 |
| 用户登录 | 验证凭据, 签发 JWT | `argon2::verify`, `p256` ECDSA 签名 |
| 当前用户 | 获取当前登录用户信息 | JWT 中间件提取 claims |
| 密码修改 | 修改当前用户密码 | `argon2` 新哈希 |
| 初始化状态 | 检查系统是否已初始化 | `rusqlite` 查询用户表 |
| JWKS 端点 | 发布公钥供其他服务验证 | `p256::PublicKey` -> JWK 格式 JSON |

### 4.8 CapeOS-LocalStorage -- 本地存储管理

**实现**: Axum + `sysinfo` + `tokio::process`

**API 基路径**: `/v1/local_storage`

| 功能 | 说明 | Rust 实现 |
|------|------|-----------|
| 磁盘列表 | 列出所有磁盘及分区信息 | `sysinfo::Disks` |
| USB 管理 | 列出 USB 存储设备, 自动挂载 | 读取 `/sys/block/` + `tokio::process` |
| 存储管理 | 添加/删除存储设备 | `nix::mount` / `tokio::process` |
| MergerFS | 合并多磁盘为 `/DATA` | `tokio::process::Command::new("mergerfs")` |
| 合并初始化 | 配置 MergerFS 合并策略 | 读写 MergerFS 配置文件 |

### 4.9 CapeOS-AppManagement -- 应用管理

**实现**: Axum + `bollard` (Rust Docker 客户端) + rusqlite

**API 基路径**: `/v1/app_management`

| 功能 | 说明 | Rust 实现 |
|------|------|-----------|
| 应用商店 | 浏览和搜索可安装应用 | `reqwest` 拉取远程商店目录 |
| Compose 安装 | 安装 Docker Compose 应用 | `bollard` + compose YAML 解析 |
| Compose 管理 | 启停/更新/卸载 Compose 应用 | `bollard` API |
| 容器管理 | 列出/创建/删除/启停容器 | `bollard::Docker::connect_with_local_defaults()` |
| 镜像管理 | 列出本地 Docker 镜像 | `bollard` images API |
| 全局设置 | 管理应用管理的全局配置 | rusqlite 存储 |

### 4.10 CapeOS-MessageBus -- 消息总线

**实现**: Axum + `tokio::sync::broadcast` + `tokio::net::UnixListener`

**API 基路径**: `/v1/message_bus`

| 功能 | 说明 | Rust 实现 |
|------|------|-----------|
| 事件类型注册 | 服务启动时注册事件类型 | `RwLock<HashMap<String, EventType>>` |
| 事件发布 | REST 或 Unix Socket 发布 | `broadcast::Sender::send()` |
| 动作类型注册 | 注册可触发的动作类型 | `RwLock<HashMap<String, ActionType>>` |
| 动作触发 | REST 触发动作 | `broadcast::Sender::send()` |
| WebSocket 订阅 | 实时订阅事件流 | `axum::extract::ws` + `broadcast::Receiver` |

---

## 5. API 接口梳理

### 5.1 CapeOS-Gateway 管理 API

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/v1/gateway/routes` | 注册新路由 (各服务启动时调用) |
| GET | `/v1/gateway/port` | 查询当前网关端口 |
| PUT | `/v1/gateway/port` | 修改网关端口 |
| GET | `/ping` | 健康检查 |

### 5.2 CapeOS-UserService API (`/v1/user_service`)

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/v1/user_service/users/login` | 用户登录, 返回 JWT |
| POST | `/v1/user_service/users/register` | 用户注册 |
| GET | `/v1/user_service/users/current` | 获取当前用户信息 |
| PUT | `/v1/user_service/users/current/password` | 修改密码 |
| GET | `/v1/user_service/users/name` | 检查用户名是否存在 |
| GET | `/v1/user_service/users/status` | 获取系统初始化状态 |
| DELETE | `/v1/user_service/users/current` | 删除用户账号 |
| GET | `/.well-known/jwks.json` | JWKS 公钥端点 |

### 5.3 CapeOS-LocalStorage API (`/v1/local_storage`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/v1/local_storage/disks` | 列出所有磁盘 |
| GET | `/v1/local_storage/disks/usb` | 列出 USB 存储设备 |
| GET | `/v1/local_storage/storage` | 获取存储概览 |
| POST | `/v1/local_storage/storage` | 添加存储 |
| DELETE | `/v1/local_storage/storage` | 删除存储 |
| GET | `/v1/local_storage/merge/init` | 获取 MergerFS 合并状态 |
| POST | `/v1/local_storage/merge/init` | 初始化 MergerFS 合并 |
| PUT | `/v1/local_storage/merge` | 更新合并配置 |

### 5.4 CapeOS-AppManagement API (`/v1/app_management`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/v1/app_management/info` | 获取通用信息 |
| POST | `/v1/app_management/convert` | 转换 appfile 为 compose 格式 |
| GET | `/v1/app_management/global` | 获取全局设置 |
| PUT | `/v1/app_management/global` | 更新全局设置 |
| GET | `/v1/app_management/appstore` | 浏览应用商店 |
| GET | `/v1/app_management/compose` | 列出已安装的 Compose 应用 |
| POST | `/v1/app_management/compose` | 安装 Compose 应用 |
| GET | `/v1/app_management/compose/:id` | 获取应用详情 |
| PUT | `/v1/app_management/compose/:id` | 更新应用 |
| DELETE | `/v1/app_management/compose/:id` | 卸载应用 |
| PUT | `/v1/app_management/compose/:id/status` | 启动/停止应用 |
| GET | `/v1/app_management/container` | 列出容器 |
| POST | `/v1/app_management/container` | 创建容器 |
| DELETE | `/v1/app_management/container/:id` | 删除容器 |
| PUT | `/v1/app_management/container/:id/status` | 启停容器 |
| GET | `/v1/app_management/image` | 列出镜像 |

### 5.5 CapeOS-MessageBus API (`/v1/message_bus`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/v1/message_bus/event_types` | 列出已注册的事件类型 |
| POST | `/v1/message_bus/event_types` | 注册事件类型 |
| GET | `/v1/message_bus/action_types` | 列出已注册的动作类型 |
| POST | `/v1/message_bus/action_types` | 注册动作类型 |
| POST | `/v1/message_bus/event/:source_id/:name` | 发布事件 |
| POST | `/v1/message_bus/action/:source_id/:name` | 触发动作 |
| GET | `/v1/message_bus/subscribe` | WebSocket 订阅事件/动作流 |

### 5.6 CapeOS 主服务 API

#### 公开接口 (无需认证)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/ping` | -- | 健康检查 |
| GET | `/v1/sys/version/current` | -- | 返回当前版本号 |
| GET | `/v1/sys/debug` | `get_system_config_debug` | 系统调试信息 |
| GET | `/v1/sys/version/check` | `get_system_check_version` | 版本更新检查 |
| GET | `/v1/recover/:type` | `get_recover_storage` | 云存储 OAuth 回调 |

#### 系统管理 (`/v1/sys`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/sys/version` | `get_system_check_version` | 版本检查 |
| POST | `/v1/sys/update` | `system_update` | 触发系统更新 |
| GET | `/v1/sys/hardware` | `get_system_hardware_info` | 硬件信息 |
| GET | `/v1/sys/wsssh` | `ws_ssh` | WebSocket SSH 终端 |
| POST | `/v1/sys/ssh-login` | `post_ssh_login` | SSH 登录验证 |
| GET | `/v1/sys/logs` | `get_capeos_error_logs` | 错误日志 |
| POST | `/v1/sys/stop` | `post_kill_capeos` | 停止 CapeOS |
| GET | `/v1/sys/utilization` | `get_system_utilization` | 系统资源利用率 |
| GET | `/v1/sys/proxy` | `get_system_proxy` | 代理 URL |
| PUT | `/v1/sys/state/:state` | `put_system_state` | 关机/重启 |
| GET | `/v1/sys/entry` | `get_system_entry` | 模块入口配置 |

#### 端口管理 (`/v1/port`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/port/` | `get_port` | 获取可用端口 |
| GET | `/v1/port/state/:port` | `port_check` | 检查端口可用性 |

#### 文件管理 (`/v1/file`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/file` | `get_download_single_file` | 下载单个文件 |
| POST | `/v1/file` | `post_create_file` | 创建文件 |
| PUT | `/v1/file` | `put_file_content` | 更新文件内容 |
| PUT | `/v1/file/name` | `rename_path` | 重命名 |
| GET | `/v1/file/content` | `get_file_content` | 读取文件内容 |
| POST | `/v1/file/upload` | `post_file_upload` | 上传文件 |
| GET | `/v1/file/upload` | `get_file_upload` | 检查分块上传状态 |
| GET | `/v1/file/ws` | `connect_websocket` | WebSocket 连接 |
| GET | `/v1/file/peers` | `get_peers` | 已连接对等设备 |

#### 目录管理 (`/v1/folder`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/folder` | `dir_path` | 列出目录内容 |
| POST | `/v1/folder` | `mkdir_all` | 创建目录 |
| PUT | `/v1/folder/name` | `rename_path` | 重命名目录 |
| GET | `/v1/folder/size` | `get_size` | 目录大小 |
| GET | `/v1/folder/count` | `get_file_count` | 文件数量 |

#### 批量操作 (`/v1/batch`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/batch` | `get_download_file` | 批量下载 (zip/tar) |
| DELETE | `/v1/batch` | `delete_file` | 批量删除 |
| POST | `/v1/batch/task` | `post_operate_file_or_dir` | 创建复制/移动任务 |
| DELETE | `/v1/batch/:id/task` | `delete_operate_file_or_dir` | 取消操作任务 |

#### 图片 (`/v1/image`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/image` | `get_file_image` | 缩略图/原图 |

#### Samba (`/v1/samba`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/samba/connections` | `get_samba_connections_list` | 列出网络连接 |
| POST | `/v1/samba/connections` | `post_samba_connections_create` | 创建网络连接 |
| DELETE | `/v1/samba/connections/:id` | `delete_samba_connections` | 删除网络连接 |
| GET | `/v1/samba/shares` | `get_samba_shares_list` | 列出共享 |
| POST | `/v1/samba/shares` | `post_samba_shares_create` | 创建共享 |
| DELETE | `/v1/samba/shares/:id` | `delete_samba_shares` | 删除共享 |
| GET | `/v1/samba/shares/status` | `get_samba_status` | Samba 服务状态 |

#### 通知 (`/v1/notify`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| POST | `/v1/notify/:path` | `post_notify_message` | 发送通知 |
| POST | `/v1/notify/system_status` | `post_system_status_notify` | 系统状态通知 |

#### 云存储 (`/v1/cloud`, `/v1/driver`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/cloud` | `list_storages` | 列出已挂载云存储 |
| DELETE | `/v1/cloud` | `umount_storage` | 卸载云存储 |
| GET | `/v1/driver` | `list_driver_info` | 可用驱动列表 |

#### 其他

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/other/search` | `get_search_result` | 搜索引擎代理 |
| ANY | `/v1/zt/*url` | `zerotier_proxy` | ZeroTier API 代理 |

#### 健康检查与文件服务

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/capeos/health/services` | `get_health_services` | 各 capeos-* 服务运行状态 |
| GET | `/v1/capeos/health/ports` | `get_health_ports` | TCP/UDP 端口占用 |
| GET | `/v1/capeos/health/logs` | `get_health_logs` | 日志压缩包下载 |
| GET | `/v1/capeos/file/upload` | `check_upload_chunk` | 检查分块上传状态 |
| POST | `/v1/capeos/file/upload` | `post_upload_file` | 分块上传文件 |
| GET | `/v1/capeos/zt/info` | `get_zerotier_info` | ZeroTier 节点信息 |
| PUT | `/v1/capeos/zt/:network_id/status` | `set_zerotier_network_status` | 设置 ZeroTier 网络状态 |

### 5.7 文件服务与文档接口

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/v1/file?token=xxx&path=xxx` | Token 鉴权的文件直接访问 |
| GET | `/doc/v1/capeos` | utoipa Swagger UI 文档页面 |
| GET | `/doc/v1/capeos/openapi.json` | utoipa 自动生成的 OpenAPI 规范 |

### 5.8 通用响应格式

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: i32,     // HTTP 状态码
    pub message: String,  // 消息
    pub data: Option<T>,  // 响应数据
}
```

JSON 示例:

```json
{
    "success": 200,
    "message": "ok",
    "data": {}
}
```

---

## 6. 代码框架结构

### 6.1 Cargo Workspace 项目结构

```text
capeos/
+-- Cargo.toml                          # Workspace 根配置
+-- Cargo.lock
+-- rust-toolchain.toml                 # Rust 工具链版本锁定
|
+-- crates/
|   |
|   +-- capeos-common/                  # 共享库 (lib crate)
|   |   +-- Cargo.toml
|   |   +-- src/
|   |       +-- lib.rs
|   |       +-- gateway_client.rs       # reqwest -> POST /v1/gateway/routes
|   |       +-- jwt.rs                  # jsonwebtoken + p256 ECDSA 验证
|   |       +-- message_bus_client.rs   # tokio::net::UnixStream -> MessageBus
|   |       +-- service_discovery.rs    # 读取 /var/run/capeos/*.url + 重试
|   |       +-- middleware/
|   |       |   +-- mod.rs
|   |       |   +-- jwt_auth.rs         # Axum JWT 中间件层
|   |       |   +-- cors.rs             # tower-http CorsLayer 封装
|   |       |   +-- compression.rs      # tower-http CompressionLayer 封装
|   |       +-- models/
|   |       |   +-- mod.rs
|   |       |   +-- route.rs            # Route { path, target }
|   |       |   +-- device.rs           # DeviceInfo
|   |       |   +-- event.rs            # EventType, PropertyType
|   |       |   +-- response.rs         # ApiResponse<T> 统一响应
|   |       +-- error.rs                # 统一错误类型 + 错误码
|   |       +-- utils/
|   |           +-- mod.rs
|   |           +-- sysctl.rs           # zbus_systemd 封装
|   |           +-- file_ops.rs         # 文件操作工具
|   |           +-- net.rs              # IP/MAC 地址工具
|   |           +-- port.rs             # 端口可用性检测
|   |
|   +-- capeos-gateway/                 # API 网关 (bin crate)
|   |   +-- Cargo.toml
|   |   +-- src/
|   |       +-- main.rs                 # 启动: 绑定端口, 写入 URL 文件
|   |       +-- route_table.rs          # Arc<RwLock<HashMap<String, String>>>
|   |       +-- proxy.rs                # axum-reverse-proxy 请求转发
|   |       +-- management.rs           # /v1/gateway/{routes,port} handlers
|   |
|   +-- capeos-message-bus/             # 消息总线 (bin crate)
|   |   +-- Cargo.toml
|   |   +-- src/
|   |       +-- main.rs
|   |       +-- event_store.rs          # RwLock<HashMap> 事件/动作类型注册
|   |       +-- publisher.rs            # REST + Unix Socket 接收发布
|   |       +-- subscriber.rs           # axum::extract::ws -> broadcast::Receiver
|   |
|   +-- capeos-user-service/            # 用户认证 (bin crate)
|   |   +-- Cargo.toml
|   |   +-- src/
|   |       +-- main.rs
|   |       +-- routes.rs               # /v1/user_service/users/*
|   |       +-- jwt_issuer.rs           # p256 密钥对生成 + JWT 签发
|   |       +-- jwks.rs                 # /.well-known/jwks.json 端点
|   |       +-- repo.rs                 # rusqlite 用户表 CRUD
|   |
|   +-- capeos-local-storage/           # 本地存储管理 (bin crate)
|   |   +-- Cargo.toml
|   |   +-- src/
|   |       +-- main.rs
|   |       +-- routes.rs               # /v1/local_storage/*
|   |       +-- disk.rs                 # sysinfo::Disks
|   |       +-- usb.rs                  # USB 设备检测
|   |       +-- mergerfs.rs             # tokio::process MergerFS 管理
|   |
|   +-- capeos-app-management/          # 应用管理 (bin crate)
|   |   +-- Cargo.toml
|   |   +-- src/
|   |       +-- main.rs
|   |       +-- routes.rs               # /v1/app_management/*
|   |       +-- compose.rs              # Docker Compose 操作
|   |       +-- container.rs            # bollard 容器 CRUD
|   |       +-- image.rs                # bollard 镜像管理
|   |       +-- app_store.rs            # 远程应用商店
|   |
|   +-- capeos-main/                    # 主服务 - Leptos 全栈 (bin crate)
|   |   +-- Cargo.toml
|   |   +-- src/
|   |   |   +-- main.rs                 # Leptos + Axum 启动入口
|   |   |   +-- app.rs                  # Leptos 根组件 + 前端路由
|   |   |   |
|   |   |   +-- server/                 # === 后端 (#[cfg(feature="ssr")]) ===
|   |   |   |   +-- mod.rs
|   |   |   |   +-- state.rs            # AppState { db, cache, ... }
|   |   |   |   +-- db.rs               # rusqlite 初始化 + rusqlite_migration
|   |   |   |   |
|   |   |   |   +-- services/           # Service 层
|   |   |   |   |   +-- mod.rs
|   |   |   |   |   +-- system.rs       # sysinfo 系统管理
|   |   |   |   |   +-- file_ops.rs     # tokio::fs 文件操作
|   |   |   |   |   +-- upload.rs       # 分块上传 (DashMap)
|   |   |   |   |   +-- shares.rs       # Samba 共享
|   |   |   |   |   +-- connections.rs  # 网络连接 (pavao)
|   |   |   |   |   +-- storage.rs      # 云存储 (rclone)
|   |   |   |   |   +-- notify.rs       # 通知 (MessageBus)
|   |   |   |   |   +-- health.rs       # 健康检查
|   |   |   |   |   +-- peer.rs         # 设备发现
|   |   |   |   |
|   |   |   |   +-- repo/               # Repository 层 (rusqlite)
|   |   |   |   |   +-- mod.rs
|   |   |   |   |   +-- shares_repo.rs
|   |   |   |   |   +-- connections_repo.rs
|   |   |   |   |   +-- notify_repo.rs
|   |   |   |   |   +-- peer_repo.rs
|   |   |   |   |
|   |   |   |   +-- api/                # REST API
|   |   |   |   |   +-- mod.rs
|   |   |   |   |   +-- v1.rs           # Axum handlers + utoipa
|   |   |   |   |
|   |   |   |   +-- drivers/            # 云存储驱动
|   |   |   |       +-- mod.rs          # CloudDriver trait
|   |   |   |       +-- google_drive.rs
|   |   |   |       +-- onedrive.rs
|   |   |   |       +-- dropbox.rs
|   |   |   |
|   |   |   +-- pages/                  # === 前端页面 (Leptos WASM) ===
|   |   |   |   +-- mod.rs
|   |   |   |   +-- dashboard.rs        # 仪表盘
|   |   |   |   +-- files.rs            # 文件管理器
|   |   |   |   +-- apps.rs             # 应用商店
|   |   |   |   +-- storage.rs          # 存储管理
|   |   |   |   +-- shares.rs           # 网络共享
|   |   |   |   +-- settings.rs         # 系统设置
|   |   |   |   +-- terminal.rs         # SSH 终端
|   |   |   |   +-- login.rs            # 登录页
|   |   |   |
|   |   |   +-- components/             # === 可复用 UI 组件 ===
|   |   |       +-- mod.rs
|   |   |       +-- layout.rs           # 页面布局
|   |   |       +-- file_list.rs        # 文件列表
|   |   |       +-- file_uploader.rs    # 拖拽上传
|   |   |       +-- cpu_gauge.rs        # CPU 仪表盘
|   |   |       +-- memory_bar.rs       # 内存条
|   |   |       +-- network_chart.rs    # 网络流量图
|   |   |       +-- notification.rs     # 通知弹窗
|   |   |
|   |   +-- style/
|   |   |   +-- main.css                # TailwindCSS
|   |   +-- public/
|   |       +-- favicon.ico
|   |
|   +-- capeos-cli/                     # CLI 工具 (bin crate)
|       +-- Cargo.toml
|       +-- src/
|           +-- main.rs
|
+-- config/
|   +-- capeos.toml.sample              # TOML 配置文件模板
|
+-- migrations/
|   +-- 001_initial.sql                 # 初始数据库迁移
|
+-- deploy/
    +-- systemd/                        # systemd 服务单元文件
    |   +-- capeos-gateway.service
    |   +-- capeos-message-bus.service
    |   +-- capeos-user-service.service
    |   +-- capeos-local-storage.service
    |   +-- capeos-app-management.service
    |   +-- capeos.service
    +-- scripts/
        +-- install.sh                  # 安装脚本
```

### 6.2 三层架构

```text
+----------------------------------------------------------------+
|                   Route 层 (路由 / HTTP Handler)                |
|                                                                |
|  Leptos Server Functions (#[server])                           |
|  + Axum handlers (api/v1.rs) + utoipa OpenAPI 标注             |
|                                                                |
|  职责: 请求解析, 参数校验, 调用 Service, 构造响应               |
+-------------------------------+--------------------------------+
                                | 调用
                                v
+----------------------------------------------------------------+
|                   Service 层 (业务逻辑)                        |
|                                                                |
|  server/services/*.rs                                          |
|                                                                |
|  职责: 业务逻辑, 调用 Repo, 调用外部服务 (Gateway/MessageBus)  |
+-------------------------------+--------------------------------+
                                | 调用
                                v
+----------------------------------------------------------------+
|                   Repository 层 (数据访问)                     |
|                                                                |
|  server/repo/*.rs         --> rusqlite SQL 操作                |
|  capeos-common/           --> 外部服务客户端 (reqwest)          |
|  tokio::fs / tokio::process --> 文件系统 / 子进程              |
|                                                                |
|  职责: 数据持久化, 外部 API 调用, 文件系统操作                 |
+----------------------------------------------------------------+
```

### 6.3 前后端连接: Leptos Server Functions

Leptos Server Functions 是全栈 Rust 的核心桥梁 -- 同一个 Rust 函数在服务端直接执行, 在 WASM 端自动生成 HTTP 调用:

```rust
#[server(ListShares, "/api")]
pub async fn list_shares() -> Result<Vec<Share>, ServerFnError> {
    let state = expect_context::<AppState>();
    state.shares_service.list().await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn SharesPage() -> impl IntoView {
    let shares = create_resource(|| (), |_| list_shares());
    view! {
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || shares.get().map(|result| match result {
                Ok(list) => view! { <ShareTable shares=list /> }.into_view(),
                Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_view(),
            })}
        </Suspense>
    }
}
```

同时保留 `api/v1.rs` 中的传统 REST 端点供第三方和 CLI 调用。

---

## 7. 后端数据库实现

### 7.1 技术选型

| 项目 | 选型 | 说明 |
|------|------|------|
| 数据库引擎 | SQLite | 轻量嵌入式, 无独立进程, 适合边缘设备 |
| Rust 绑定 | `rusqlite` 0.38+ (MIT) | 基于 C sqlite3 的安全绑定 |
| 异步封装 | `tokio-rusqlite` 0.7+ (MIT) | 后台线程 + mpsc channel, 适配 Tokio |
| 迁移 | `rusqlite_migration` (Apache-2.0) | 利用 SQLite `user_version` pragma |

设计原则: 结构化数据设计, 少冗余, 少磁盘 IO, 所有结构化数据用表存储, 不使用文件存储。

### 7.2 数据库初始化

```rust
use tokio_rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

pub async fn init_db(db_path: &str) -> Result<Connection> {
    tokio::fs::create_dir_all(db_path).await?;
    let db_file = format!("{}/capeos.db", db_path);
    let conn = Connection::open(&db_file).await?;

    conn.call(|conn| {
        // 性能优化
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;

        // 执行迁移
        let migrations = Migrations::new(vec![
            M::up(include_str!("../../migrations/001_initial.sql")),
        ]);
        migrations.to_latest(conn)?;
        Ok(())
    }).await?;

    Ok(conn)
}
```

**数据库文件位置**: `/var/lib/capeos/db/capeos.db`

### 7.3 数据库表结构

初始迁移 SQL (`migrations/001_initial.sql`):

```sql
CREATE TABLE IF NOT EXISTS o_notify (
    custom_id  TEXT PRIMARY KEY,
    id         TEXT,
    name       TEXT,
    icon       TEXT,
    state      INTEGER DEFAULT 0,
    type       INTEGER DEFAULT 0,
    class      INTEGER DEFAULT 0,
    message    TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS o_shares (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    name      TEXT NOT NULL,
    path      TEXT NOT NULL,
    anonymous INTEGER DEFAULT 0,
    created   INTEGER DEFAULT (unixepoch()),
    updated   INTEGER DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS o_connections (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    username    TEXT,
    password    TEXT,
    host        TEXT NOT NULL,
    port        TEXT DEFAULT '445',
    status      TEXT DEFAULT 'disconnected',
    directories TEXT,
    mount_point TEXT,
    created     INTEGER DEFAULT (unixepoch()),
    updated     INTEGER DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS peer_drives (
    id           TEXT PRIMARY KEY,
    user_agent   TEXT,
    display_name TEXT,
    device_name  TEXT,
    model        TEXT,
    ip           TEXT,
    os           TEXT,
    browser      TEXT,
    created      INTEGER DEFAULT (unixepoch()),
    updated      INTEGER DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS o_rely (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    custom_id            TEXT,
    container_custom_id  TEXT,
    container_id         TEXT,
    type                 INTEGER DEFAULT 0,
    created_at           TEXT DEFAULT (datetime('now')),
    updated_at           TEXT DEFAULT (datetime('now'))
);
```

#### 表结构说明

**通知表 `o_notify`**

| 字段 | 类型 | 属性 | 说明 |
|------|------|------|------|
| custom_id | TEXT | **PK** | 自定义唯一标识 |
| id | TEXT | | 通知 ID |
| name | TEXT | | 通知名称 |
| icon | TEXT | | 图标 URL |
| state | INTEGER | 默认 0 | 0=变动中未读, 1=未读, 2=已读 |
| type | INTEGER | 默认 0 | 通知类型 |
| class | INTEGER | 默认 0 | 通知分类 |
| message | TEXT | | 通知内容 |
| created_at | TEXT | 自动填充 | 创建时间 |
| updated_at | TEXT | 自动填充 | 更新时间 |

**Samba 共享表 `o_shares`**

| 字段 | 类型 | 属性 | 说明 |
|------|------|------|------|
| id | INTEGER | **PK**, 自增 | 共享 ID |
| name | TEXT | NOT NULL | 共享名称 |
| path | TEXT | NOT NULL | 共享目录路径 |
| anonymous | INTEGER | 默认 0 | 0=不允许, 1=允许匿名 |
| created | INTEGER | unixepoch() | 创建时间戳 |
| updated | INTEGER | unixepoch() | 更新时间戳 |

**网络连接表 `o_connections`**

| 字段 | 类型 | 属性 | 说明 |
|------|------|------|------|
| id | INTEGER | **PK**, 自增 | 连接 ID |
| username | TEXT | | 远程用户名 |
| password | TEXT | | 远程密码 |
| host | TEXT | NOT NULL | 远程主机地址 |
| port | TEXT | 默认 445 | 远程端口 |
| status | TEXT | 默认 disconnected | 连接状态 |
| directories | TEXT | | 目录列表 (JSON 字符串数组) |
| mount_point | TEXT | | 本地挂载点父目录 |
| created | INTEGER | unixepoch() | 创建时间戳 |
| updated | INTEGER | unixepoch() | 更新时间戳 |

**对等设备表 `peer_drives`**

| 字段 | 类型 | 属性 | 说明 |
|------|------|------|------|
| id | TEXT | **PK** | 设备唯一标识 |
| user_agent | TEXT | | User-Agent |
| display_name | TEXT | | 显示名称 |
| device_name | TEXT | | 设备名称 |
| model | TEXT | | 设备型号 |
| ip | TEXT | | IP 地址 |
| os | TEXT | | 操作系统 |
| browser | TEXT | | 浏览器 |
| created | INTEGER | unixepoch() | 创建时间戳 |
| updated | INTEGER | unixepoch() | 更新时间戳 |

**依赖关系表 `o_rely`**

| 字段 | 类型 | 属性 | 说明 |
|------|------|------|------|
| id | INTEGER | **PK**, 自增 | 记录 ID |
| custom_id | TEXT | | 自定义 ID |
| container_custom_id | TEXT | | 容器自定义 ID |
| container_id | TEXT | | 容器 ID |
| type | INTEGER | 默认 0 | 类型 |
| created_at | TEXT | 自动填充 | 创建时间 |
| updated_at | TEXT | 自动填充 | 更新时间 |

### 7.4 数据访问模式 (Repository 层)

所有 SQL 操作集中在 `server/repo/` 目录, 通过 `tokio-rusqlite` 的 `conn.call()` 方法异步执行:

```rust
use tokio_rusqlite::Connection;
use rusqlite::params;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Share {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub anonymous: bool,
    pub created: i64,
    pub updated: i64,
}

#[derive(Clone)]
pub struct SharesRepo {
    db: Connection,
}

impl SharesRepo {
    pub fn new(db: Connection) -> Self {
        Self { db }
    }

    pub async fn find_all(&self) -> Result<Vec<Share>> {
        self.db.call(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, path, anonymous, created, updated
                 FROM o_shares ORDER BY id"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(Share {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    anonymous: row.get::<_, i32>(3)? != 0,
                    created: row.get(4)?,
                    updated: row.get(5)?,
                })
            })?.collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        }).await
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Share>> {
        self.db.call(move |conn| {
            conn.query_row(
                "SELECT id, name, path, anonymous, created, updated
                 FROM o_shares WHERE id = ?1",
                [id],
                |row| Ok(Share {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    anonymous: row.get::<_, i32>(3)? != 0,
                    created: row.get(4)?,
                    updated: row.get(5)?,
                }),
            ).optional().map_err(Into::into)
        }).await
    }

    pub async fn insert(&self, name: String, path: String, anonymous: bool) -> Result<i64> {
        self.db.call(move |conn| {
            conn.execute(
                "INSERT INTO o_shares (name, path, anonymous)
                 VALUES (?1, ?2, ?3)",
                params![name, path, anonymous as i32],
            )?;
            Ok(conn.last_insert_rowid())
        }).await
    }

    pub async fn delete(&self, id: i64) -> Result<bool> {
        self.db.call(move |conn| {
            let affected = conn.execute(
                "DELETE FROM o_shares WHERE id = ?1", [id]
            )?;
            Ok(affected > 0)
        }).await
    }
}
```

### 7.5 系统路径常量

定义在 `capeos-common/src/lib.rs`:

```rust
pub mod paths {
    pub const DEFAULT_CONFIG_PATH: &str = "/etc/capeos";
    pub const DEFAULT_CONSTANT_PATH: &str = "/usr/share/capeos";
    pub const DEFAULT_DATA_PATH: &str = "/var/lib/capeos";
    pub const DEFAULT_FILE_PATH: &str = "/var/lib/capeos/files";
    pub const DEFAULT_LOG_PATH: &str = "/var/log/capeos";
    pub const DEFAULT_RUNTIME_PATH: &str = "/var/run/capeos";
    pub const DEFAULT_DB_PATH: &str = "/var/lib/capeos/db";
}
```

### 7.6 数据存储策略

| 存储方式 | 用途 | 位置 |
|----------|------|------|
| **SQLite (rusqlite)** | 所有结构化数据 | `/var/lib/capeos/db/capeos.db` |
| **TOML 配置文件** | 应用启动配置 | `/etc/capeos/capeos.toml` |
| **内存缓存 (moka)** | 高频读取数据 | 进程内存 |
| **文件系统** | 用户数据, 日志 | `/var/lib/capeos/files/`, `/var/log/capeos/` |
| **运行时 URL 文件** | 服务发现 | `/var/run/capeos/*.url` |
