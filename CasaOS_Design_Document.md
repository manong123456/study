# CasaOS 系统设计文档

> **CasaOS 仓库**: https://github.com/IceWhaleTech/CasaOS.git, 分支 `main`, commit `63f0148`  
> **CasaOS-Common 仓库**: https://github.com/IceWhaleTech/CasaOS-Common.git, 分支 `main`, commit `909dcbd`  
> **参考**: https://wiki.casaos.io/en/contribute/development  
> **版本**: CasaOS v0.4.15  
> **编写日期**: 2026-03-02

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

CasaOS 是由 IceWhale Technology 开发的开源个人云操作系统，面向家庭用户提供简洁易用的私有云解决方案。运行在 Linux 设备（ZimaBoard、树莓派、x86 小主机等）上，提供文件管理、Docker 应用商店、磁盘管理、网络共享、云存储挂载、系统监控等能力。

CasaOS 采用 **微服务网关架构**，由多个独立的 Go 服务组成，通过 CasaOS-Gateway 统一对外提供服务，通过 CasaOS-MessageBus 实现跨服务事件通信。

### 核心技术栈

| 类别 | 技术选型 |
|------|----------|
| 编程语言 | Go 1.21 |
| Web 框架 | Echo v4 (labstack/echo) |
| ORM | GORM (gorm.io/gorm) |
| 数据库 | SQLite (glebarez/sqlite) |
| API 规范 | OpenAPI 3.0 + oapi-codegen |
| 认证 | ECDSA JWT (P-256 曲线) + JWKS |
| 消息总线 | CasaOS-MessageBus (事件驱动, WebSocket + Unix Socket) |
| 定时任务 | robfig/cron v3 |
| 系统信息 | gopsutil v3 |
| 网络共享 | go-smb2 (SMB/CIFS) |
| 云存储 | Google Drive / OneDrive / Dropbox (OAuth 2.0) |
| 容器管理 | Docker Engine API |
| 磁盘合并 | MergerFS |
| 服务管理 | systemd (coreos/go-systemd) |
| 构建 | goreleaser + UPX (amd64/arm64/arm-7) |

---

## 2. 系统架构

### 2.1 整体架构图

参考 https://wiki.casaos.io/en/contribute/development 的 Architecture 部分，CasaOS 采用模块化微服务架构，所有服务通过 Gateway 统一对外暴露，通过 MessageBus 实现事件驱动通信。

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                            用户浏览器 / CasaOS-UI                                │
│                     (Vue.js 前端，访问 http://<host>:80)                          │
└──────────────────────────────────┬───────────────────────────────────────────────┘
                                   │ HTTP / WebSocket
                                   ▼
┌──────────────────────────────────────────────────────────────────────────────────┐
│                                                                                  │
│                        CasaOS-Gateway  (端口 80)                                 │
│                     唯一对外暴露的网络入口 / 反向代理                                │
│                                                                                  │
│    ┌─────────────────────────────────────────────────────────────────────────┐   │
│    │                       动态路由表 (Route Table)                          │   │
│    │                                                                         │   │
│    │  /v2/user_service/*   ──→  http://127.0.0.1:<port_A>                   │   │
│    │  /v2/local_storage/*  ──→  http://127.0.0.1:<port_B>                   │   │
│    │  /v2/app_management/* ──→  http://127.0.0.1:<port_C>                   │   │
│    │  /v2/message_bus/*    ──→  http://127.0.0.1:<port_D>                   │   │
│    │  /v1/sys, /v1/file... ──→  http://127.0.0.1:<port_E>  (CasaOS主服务)   │   │
│    │  /v2/casaos/*         ──→  http://127.0.0.1:<port_E>                   │   │
│    │  /v3/file             ──→  http://127.0.0.1:<port_E>                   │   │
│    │                                                                         │   │
│    └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                  │
│    Management API (内部端口，仅 localhost):                                       │
│      POST /v1/gateway/routes   ← 各服务启动时注册路由                              │
│      GET  /v1/gateway/port     ← 查询网关端口                                    │
│      PUT  /v1/gateway/port     ← 修改网关端口                                    │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
          │              │             │              │              │
          ▼              ▼             ▼              ▼              ▼
┌──────────────┐ ┌──────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────────────┐
│  CasaOS      │ │ UserService  │ │ Local      │ │ App        │ │  MessageBus      │
│  主服务       │ │              │ │ Storage    │ │ Management │ │                  │
│              │ │ 用户认证      │ │            │ │            │ │  事件/动作        │
│ 文件管理     │ │ JWT 签发      │ │ 磁盘管理   │ │ Docker应用  │ │  发布/订阅        │
│ 系统监控     │ │ JWKS 端点     │ │ USB管理    │ │ 应用商店    │ │                  │
│ Samba共享    │ │ 账号管理      │ │ MergerFS   │ │ 容器生命周期 │ │  REST + WS       │
│ 云存储挂载   │ │              │ │ 分区管理    │ │            │ │  + Unix Socket   │
│ ZeroTier     │ │              │ │            │ │            │ │                  │
│ WebSocket    │ │              │ │            │ │            │ │                  │
│              │ │              │ │            │ │            │ │                  │
│ 127.0.0.1:? │ │ 127.0.0.1:? │ │127.0.0.1:?│ │127.0.0.1:?│ │ 127.0.0.1:?      │
└──────┬───────┘ └──────┬───────┘ └─────┬──────┘ └─────┬──────┘ └────────┬─────────┘
       │                │               │              │                 │
       └────────────────┴───────────────┴──────────────┴─────────────────┘
                                        │
                              ┌─────────▼──────────┐
                              │  CasaOS-Common     │
                              │  (共享 Go 库)       │
                              │                    │
                              │ • Gateway 客户端    │
                              │ • JWT 验证工具      │
                              │ • MessageBus 客户端 │
                              │ • 服务发现          │
                              │ • 共享模型/工具      │
                              └────────────────────┘
```

### 2.2 微服务组件列表

| 服务 | 仓库 | 二进制文件 | 职责 |
|------|------|-----------|------|
| **CasaOS-Gateway** | IceWhaleTech/CasaOS-Gateway | `casaos-gateway` | 反向代理网关，唯一对外暴露端口(80)，动态路由管理 |
| **CasaOS** (主服务) | IceWhaleTech/CasaOS | `casaos` | 文件管理、系统监控、Samba 共享、云存储、ZeroTier |
| **CasaOS-UserService** | IceWhaleTech/CasaOS-UserService | `casaos-user-service` | 用户注册/登录、JWT 签发、JWKS 公钥端点 |
| **CasaOS-LocalStorage** | IceWhaleTech/CasaOS-LocalStorage | `casaos-local-storage` | 磁盘管理、分区管理、USB 自动挂载、MergerFS |
| **CasaOS-AppManagement** | IceWhaleTech/CasaOS-AppManagement | `casaos-app-management` | Docker 应用生命周期、应用商店、Compose 编排 |
| **CasaOS-MessageBus** | IceWhaleTech/CasaOS-MessageBus | `casaos-message-bus` | 事件/动作 发布订阅、WebSocket 实时推送 |
| **CasaOS-Common** | IceWhaleTech/CasaOS-Common | (Go 库) | 共享库：Gateway 客户端、JWT 工具、MessageBus 客户端、服务发现 |
| **CasaOS-CLI** | IceWhaleTech/CasaOS-CLI | `casaos-cli` | 命令行诊断和测试工具 |

### 2.3 服务启动顺序与依赖

```
casaos-gateway                          ← 最先启动，绑定端口 80
    │
    ├── casaos-message-bus              ← 依赖 Gateway
    │       │
    │       ├── casaos-user-service     ← 依赖 MessageBus
    │       │
    │       ├── casaos-local-storage    ← 依赖 MessageBus
    │       │
    │       ├── casaos-app-management   ← 依赖 MessageBus + Docker
    │       │
    │       └── casaos (主服务)          ← 依赖 MessageBus + rclone
    │
    └── (所有服务启动时向 Gateway 注册路由)
```

所有服务以 `systemd` 方式管理，使用 `Type=notify` 通知就绪，`Restart=always` 保证高可用。

### 2.4 服务发现机制

CasaOS 采用 **基于文件的服务发现** 模式，运行时目录为 `/var/run/casaos/`：

| 文件名 | 写入者 | 内容 | 读取者 |
|--------|--------|------|--------|
| `management.url` | Gateway | 管理 API 地址 (如 `http://127.0.0.1:34703`) | 所有服务 (注册路由) |
| `gateway.url` | Gateway | 网关监听地址 (如 `http://[::]:80`) | — |
| `message-bus.url` | MessageBus | MessageBus API 地址 | 所有服务 (发布事件) |
| `user-service.url` | UserService | UserService 地址 | CasaOS (获取 JWKS 公钥) |
| `app-management.url` | AppManagement | AppManagement 地址 | CasaOS-Common |
| `casaos.url` | CasaOS 主服务 | CasaOS 主服务地址 | 其他服务 (发送通知) |

服务发现实现位于 `CasaOS-Common/external/common.go`：

```go
func getAddress(addressFile string) (string, error) {
    buf, err := os.ReadFile(addressFile)
    if err != nil {
        return "", err
    }
    return string(buf), nil
}
```

### 2.5 认证架构

```
用户登录
    │
    │  POST /v2/user_service/users/login
    ▼
┌──────────────────┐
│  UserService     │
│                  │
│  1. 验证用户凭据  │
│  2. 生成 JWT     │──→  ECDSA P-256 签名
│  3. 返回 Token   │
└──────────────────┘
         │
         │  JWT Token
         ▼
┌──────────────────────────────────────────────┐
│  后续请求: Authorization: Bearer <JWT>        │
│                                              │
│  各服务的 JWT 中间件:                          │
│  1. 从 Header 或 Query 中提取 token           │
│  2. 从 UserService 的 JWKS 端点获取公钥        │
│     GET http://<user-service>/.well-known/jwks.json │
│  3. 用 ECDSA 公钥验证签名                     │
│  4. localhost 请求跳过认证                     │
└──────────────────────────────────────────────┘
```

JWKS 公钥获取实现位于 `CasaOS-Common/external/user_service.go`，带 10 秒缓存。

### 2.6 事件驱动通信

```
                         CasaOS-MessageBus
                    ┌──────────────────────────┐
                    │                          │
  发布事件 ─────────│─→  Event Store           │
  (REST / Unix Socket)  │                     │
                    │    ┌──────────────┐      │
                    │    │ Event Types  │      │
                    │    │ Action Types │      │
                    │    └──────────────┘      │
                    │           │               │
  订阅事件 ─────────│─→  WebSocket 推送  ──────│──→ 前端 / 其他服务
                    │                          │
                    └──────────────────────────┘

事件发布方式:
  1. REST API:  POST /v2/message_bus/event/{source_id}/{name}
  2. Unix Socket: /tmp/message-bus.sock (本地高性能通信)
```

CasaOS 主服务注册的事件类型：

| 事件名称 | Source ID | 说明 |
|----------|-----------|------|
| `casaos:system:utilization` | `casaos` | 系统资源利用率（每 5 秒推送） |
| `casaos:file:recover` | `casaos` | 云存储 OAuth 回调完成 |
| `casaos:file:operate` | `casaos` | 文件操作进度（复制/移动） |

### 2.7 完整请求流程示例

以 `GET /v2/casaos/health/services` 为例：

```
1. 浏览器发送请求
   GET http://192.168.1.100:80/v2/casaos/health/services
   Header: Authorization: Bearer eyJhbGciOiJFUzI1NiI...

2. CasaOS-Gateway (端口 80) 接收请求
   ├─ 匹配路由表: /v2/casaos → http://127.0.0.1:43821
   └─ 反向代理转发到 CasaOS 主服务

3. CasaOS 主服务 (127.0.0.1:43821) 处理请求
   ├─ HandlerMultiplexer 根据路径前缀 "v2" 路由到 V2 Router
   ├─ CORS 中间件
   ├─ JWT 中间件
   │   ├─ 提取 Authorization Header 中的 Token
   │   ├─ GET http://<user-service>/.well-known/jwks.json 获取公钥
   │   └─ ECDSA 验签成功，提取 user_id
   ├─ OpenAPI 请求验证中间件
   └─ GetHealthServices Handler
       ├─ 调用 systemctl 列出所有 casaos-* 服务状态
       └─ 返回 JSON: { running: [...], not_running: [...] }

4. 响应原路返回: CasaOS → Gateway → 浏览器
```

---

## 3. 系统功能模块总览

CasaOS 的功能分布在多个微服务中，以下是按服务划分的模块概览：

```
CasaOS 生态系统
│
├── CasaOS-Gateway ──── 网关与路由管理
│
├── CasaOS (主服务)
│   ├── 系统管理模块 (System)
│   │   ├── 硬件信息采集 (CPU/内存/磁盘/网络)
│   │   ├── 系统资源实时监控
│   │   ├── 版本管理与在线更新
│   │   ├── 电源控制 (重启/关机)
│   │   ├── SSH WebSocket 终端
│   │   └── 日志管理
│   │
│   ├── 文件管理模块 (File)
│   │   ├── 文件/目录浏览与操作
│   │   ├── 分块文件上传
│   │   ├── 单文件/批量打包下载
│   │   ├── 文件复制/移动 (异步任务)
│   │   ├── 图片缩略图生成
│   │   └── WebSocket 实时通信
│   │
│   ├── 网络存储模块 (Samba/CIFS)
│   │   ├── Samba 本地共享管理
│   │   ├── 远程 CIFS 连接管理
│   │   └── 网络挂载自动恢复
│   │
│   ├── 云存储模块 (Cloud Storage)
│   │   ├── Google Drive / OneDrive / Dropbox
│   │   ├── OAuth 2.0 认证回调
│   │   └── rclone 挂载管理
│   │
│   ├── 通知模块 (Notify)
│   ├── 健康检查模块 (Health)
│   ├── ZeroTier 网络模块
│   └── 设备发现模块 (Peer)
│
├── CasaOS-UserService ──── 用户管理与认证
│   ├── 用户注册/登录
│   ├── JWT Token 签发
│   ├── JWKS 公钥发布
│   └── 密码管理
│
├── CasaOS-LocalStorage ──── 本地存储管理
│   ├── 磁盘列表与信息
│   ├── USB 设备管理
│   ├── MergerFS 磁盘合并
│   └── 分区管理
│
├── CasaOS-AppManagement ──── 应用管理
│   ├── 应用商店
│   ├── Docker Compose 应用安装/卸载
│   ├── 容器生命周期管理
│   └── 镜像管理
│
└── CasaOS-MessageBus ──── 消息总线
    ├── 事件类型注册
    ├── 事件发布
    ├── WebSocket 事件订阅
    └── 动作触发
```

---

## 4. 各功能模块详细介绍

### 4.1 CasaOS-Gateway — 网关服务

**仓库**: IceWhaleTech/CasaOS-Gateway

Gateway 是整个 CasaOS 系统的唯一网络入口，实现动态 API 路由。

**工作原理**:
1. Gateway 启动后绑定外部端口（默认 80）和内部管理端口（随机）
2. 将管理端口地址写入 `/var/run/casaos/management.url`
3. 其他微服务启动时读取该文件，调用 `POST /v1/gateway/routes` 注册自身路由
4. Gateway 维护一个动态路由表，根据请求路径前缀转发到对应后端服务

**路由注册模型** (定义在 `CasaOS-Common/model/gateway.go`):

```go
type Route struct {
    Path   string `json:"path" binding:"required"`   // 路由前缀，如 "/v2/casaos"
    Target string `json:"target" binding:"required"`  // 后端地址，如 "http://127.0.0.1:43821"
}
```

**路由注册客户端** (定义在 `CasaOS-Common/external/gateway.go`):

```go
type ManagementService interface {
    CreateRoute(route *model.Route) error      // 注册路由
    ChangePort(request *model.ChangePortRequest) error  // 修改网关端口
    GetPort() (error, string)                   // 查询网关端口
}
```

### 4.2 CasaOS 主服务 — 系统管理模块 (SystemService)

**源文件**: `service/system.go`

提供系统级信息采集和管理控制功能。

| 功能 | 方法 | 实现方式 |
|------|------|----------|
| CPU 使用率 | `GetCpuPercent()` | gopsutil `cpu.Percent()` |
| CPU 信息 | `GetCpuInfo()` | gopsutil `cpu.Info()` |
| CPU 核心数 | `GetCpuCoreNum()` | gopsutil `cpu.Counts()` |
| CPU 温度 | `GetCPUTemperature()` | 读取 `/sys/devices/virtual/thermal/thermal_zone*/temp` |
| CPU 功耗 | `GetCPUPower()` | 读取 `/sys/class/powercap/intel-rapl/*/energy_uj` |
| 内存信息 | `GetMemInfo()` | gopsutil `mem.VirtualMemory()` |
| 磁盘信息 | `GetDiskInfo()` | gopsutil `disk.Usage("/")` |
| 网络统计 | `GetNetInfo()` | gopsutil `net.IOCounters()` |
| 物理网卡 | `GetNet(physics)` | Shell 脚本 `helper.sh GetNetCard` |
| 主机信息 | `GetSysInfo()` | gopsutil `host.Info()` |
| 设备信息 | `GetDeviceInfo()` | 聚合 IP/端口/主机名/设备型号/Hash |
| 设备树 | `GetDeviceTree()` | Shell 脚本 `helper.sh GetDeviceTree` |
| 系统更新 | `UpdateSystemVersion()` | 执行远程更新脚本 `curl \| bash` |
| 重启 | `SystemReboot()` | `init 6` |
| 关机 | `SystemShutdown()` | `init 0` |
| 日志 | `GetCasaOSLogs()` | 读取日志文件 |
| 系统入口 | `GetSystemEntry()` | 读取各模块 `entry.json` 聚合 |
| 目录操作 | `GetDirPath()` / `MkdirAll()` / `RenameFile()` / `CreateFile()` | `os` 标准库 |

**定时任务**: 每 5 秒通过 `cron` 执行 `SendAllHardwareStatusBySocket`，经由 MessageBus 的 Unix Socket 发布 `casaos:system:utilization` 事件，前端通过 WebSocket 订阅实现实时仪表盘。

### 4.3 CasaOS 主服务 — 文件管理模块

**源文件**: `route/v1/file.go`, `service/file.go`, `service/file_upload.go`

| 功能 | 说明 |
|------|------|
| 目录浏览 | 列出文件/目录，返回名称、大小、类型、修改时间、是否可写、是否为挂载点 |
| 分块上传 | 基于 `sync.Map` 追踪上传状态，写入 `.tmp` 文件，分块完成后重命名 |
| 文件下载 | 单文件直接响应；批量文件打包为 zip/tar/tar.gz 后流式下载 |
| 文件创建 | 创建空文件，检测路径冲突 |
| 文件编辑 | 读取文件内容 / 写入新内容 |
| 重命名 | 文件和目录的重命名 |
| 删除 | 支持批量删除 |
| 复制/移动 | 异步执行，基于 `FileOperate` 模型追踪进度，支持任务取消 |
| 目录大小/计数 | 递归计算目录大小和文件数量 |
| 图片缩略图 | 使用 `disintegration/imaging` 库，支持 EXIF 方向校正 |
| WebSocket | 用于对等设备文件传输的实时通道 |

**文件操作进度模型**:

```go
type FileOperate struct {
    Type          string     // "copy" 或 "move"
    Item          []FileItem // 源文件列表
    TotalSize     int64      // 总大小
    ProcessedSize int64      // 已处理大小
    To            string     // 目标路径
    Style         string     // 冲突处理策略
    Finished      bool       // 是否完成
}
```

### 4.4 CasaOS 主服务 — 网络存储模块 (Samba/CIFS)

**源文件**: `service/shares.go`, `service/connections.go`

**Samba 共享管理 (SharesService)**:
- CRUD 操作管理本机 Samba 共享目录，存储在 SQLite `o_shares` 表
- 生成 `/etc/samba/smb.casa.conf` 配置文件，通过 `include` 引入主 `smb.conf`
- 配置变更后自动重启 `smbd` 服务
- 支持匿名访问设置

**网络连接管理 (ConnectionsService)**:
- CRUD 操作管理远程 CIFS/SMB 连接，存储在 SQLite `o_connections` 表
- 使用 `go-smb2` 库验证连接凭据
- 使用 `unix.Mount` 挂载远程共享到本地目录
- 启动时自动恢复之前的网络挂载 (`route/init.go` 中的 `InitNetworkMount`)

### 4.5 CasaOS 主服务 — 云存储模块

**源文件**: `drivers/google_drive/`, `drivers/onedrive/`, `drivers/dropbox/`, `service/storage.go`

通过 `rclone` 实现云存储到本地文件系统的挂载。

| 云服务 | 认证 | 操作 |
|--------|------|------|
| Google Drive | OAuth 2.0 | List / Link / MakeDir / Move / Rename / Remove / Put(分块) |
| OneDrive | OAuth 2.0 | GetUserInfo / GetInfo / GetSpaceSize |
| Dropbox | OAuth 2.0 | List / Link / GetUserInfo / MakeDir / Move / Rename / Remove / Put |

**驱动接口** (定义在 `internal/driver/driver.go`):

```go
type Driver interface {
    Meta    // 配置、初始化、销毁
    Reader  // 文件列表
    User    // 用户信息
    Other(ctx context.Context, args model.OtherArgs) (interface{}, error)
}
```

**OAuth 回调流程**:
1. 前端引导用户到云服务 OAuth 页面
2. 回调到 `GET /v1/recover/:type`（type 为 GoogleDrive/Dropbox/Onedrive）
3. Handler 创建 rclone 配置，挂载云存储
4. 发布 `casaos:file:recover` 事件通知前端

**StorageService 接口**:

```go
type StorageService interface {
    MountStorage(mountPoint, fs string) error
    UnmountStorage(mountPoint string) error
    GetStorages() (string, error)
    CreateConfig(data map[string]string, name string, t string) error
    CheckAndMountAll() error
    GetConfig() (string, error)
    DeleteConfigByName(name string) error
}
```

### 4.6 CasaOS 主服务 — 其他模块

**通知模块 (NotifyService)**: CRUD 管理通知记录（`o_notify` 表），通过 MessageBus 发布 `casaos:file:operate` 事件通知文件操作进度。

**健康检查模块 (HealthService)**: 通过 `systemctl` 列出所有 `casaos-*` 服务运行状态；通过 `/proc/net/` 读取 TCP/UDP 端口占用；打包日志下载。

**ZeroTier 模块**: 将请求代理转发到本地 ZeroTier 服务 API；查询节点信息和网络状态。

**设备发现模块 (PeerService)**: 从 HTTP 请求中提取 User-Agent 和 IP，管理对等设备记录（`PeerDriveDBModel` 表）。

**CasaService**: 从远程 API (`api.casaos.io`) 获取最新版本信息，缓存 20 分钟。

**OtherService**: 搜索引擎代理，并行查询 Bing、Google、百度、DuckDuckGo、Startpage。

### 4.7 CasaOS-UserService — 用户管理与认证

**仓库**: IceWhaleTech/CasaOS-UserService  
**API 基路径**: `/v2/user_service`

| 功能 | 说明 |
|------|------|
| 用户注册 | 创建新用户账号 |
| 用户登录 | 验证凭据，签发 ECDSA JWT Token |
| 当前用户 | 获取当前登录用户信息 |
| 密码修改 | 修改当前用户密码 |
| 初始化状态 | 检查系统是否已完成初始化设置 |
| JWKS 端点 | `/.well-known/jwks.json` 发布公钥供其他服务验证 Token |

### 4.8 CasaOS-LocalStorage — 本地存储管理

**仓库**: IceWhaleTech/CasaOS-LocalStorage  
**API 基路径**: `/v2/local_storage`

| 功能 | 说明 |
|------|------|
| 磁盘列表 | 列出系统中的所有磁盘及其分区信息 |
| USB 管理 | 列出 USB 存储设备，支持自动挂载 |
| 存储管理 | 添加/删除存储设备到存储池 |
| MergerFS | 使用 MergerFS 将多个磁盘合并为统一的 `/DATA` 虚拟目录 |
| 合并初始化 | 初始化和配置 MergerFS 合并策略 |

### 4.9 CasaOS-AppManagement — 应用管理

**仓库**: IceWhaleTech/CasaOS-AppManagement  
**API 基路径**: `/v2/app_management`

| 功能 | 说明 |
|------|------|
| 应用商店 | 浏览和搜索可安装的应用 |
| Compose 安装 | 安装 Docker Compose 编排的应用 |
| Compose 管理 | 启动/停止/更新/卸载已安装的 Compose 应用 |
| 容器管理 | 列出/创建/删除/启停单独的 Docker 容器 |
| 镜像管理 | 列出本地 Docker 镜像 |
| 全局设置 | 管理应用管理的全局配置 |

### 4.10 CasaOS-MessageBus — 消息总线

**仓库**: IceWhaleTech/CasaOS-MessageBus  
**API 基路径**: `/v2/message_bus`

| 功能 | 说明 |
|------|------|
| 事件类型注册 | 服务启动时注册自身可发布的事件类型 |
| 事件发布 | 通过 REST 或 Unix Socket 发布事件 |
| 动作类型注册 | 注册可触发的动作类型 |
| 动作触发 | 通过 REST 触发动作 |
| WebSocket 订阅 | 前端/服务通过 WebSocket 实时订阅事件流 |

---

## 5. API 接口梳理

> 参考 https://wiki.casaos.io/en/contribute/development 的 API 章节，CasaOS 提供以下 API：
> - UserService API
> - LocalStorage API
> - AppManagement API
> - MessageBus API
> - CasaOS 主服务 API (V1 + V2)

### 5.1 CasaOS-Gateway 管理 API

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/v1/gateway/routes` | 注册新路由（各服务启动时调用） |
| GET | `/v1/gateway/port` | 查询当前网关端口 |
| PUT | `/v1/gateway/port` | 修改网关端口 |
| GET | `/ping` | 健康检查 |

### 5.2 CasaOS-UserService API (`/v2/user_service`)

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/v2/user_service/users/login` | 用户登录，返回 JWT |
| POST | `/v2/user_service/users/register` | 用户注册 |
| GET | `/v2/user_service/users/current` | 获取当前用户信息 |
| PUT | `/v2/user_service/users/current/password` | 修改密码 |
| GET | `/v2/user_service/users/name` | 检查用户名是否存在 |
| GET | `/v2/user_service/users/status` | 获取系统初始化状态 |
| DELETE | `/v2/user_service/users/current` | 删除用户账号 |
| GET | `/.well-known/jwks.json` | JWKS 公钥端点 |

### 5.3 CasaOS-LocalStorage API (`/v2/local_storage`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/v2/local_storage/disks` | 列出所有磁盘 |
| GET | `/v2/local_storage/disks/usb` | 列出 USB 存储设备 |
| GET | `/v2/local_storage/storage` | 获取存储概览 |
| POST | `/v2/local_storage/storage` | 添加存储 |
| DELETE | `/v2/local_storage/storage` | 删除存储 |
| GET | `/v2/local_storage/merge/init` | 获取 MergerFS 合并状态 |
| POST | `/v2/local_storage/merge/init` | 初始化 MergerFS 合并 |
| PUT | `/v2/local_storage/merge` | 更新合并配置 |

### 5.4 CasaOS-AppManagement API (`/v2/app_management`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/v2/app_management/info` | 获取通用信息 |
| POST | `/v2/app_management/convert` | 转换 appfile 为 compose 格式 |
| GET | `/v2/app_management/global` | 获取全局设置 |
| PUT | `/v2/app_management/global` | 更新全局设置 |
| GET | `/v2/app_management/appstore` | 浏览应用商店 |
| GET | `/v2/app_management/compose` | 列出已安装的 Compose 应用 |
| POST | `/v2/app_management/compose` | 安装 Compose 应用 |
| GET | `/v2/app_management/compose/:id` | 获取应用详情 |
| PUT | `/v2/app_management/compose/:id` | 更新应用 |
| DELETE | `/v2/app_management/compose/:id` | 卸载应用 |
| PUT | `/v2/app_management/compose/:id/status` | 启动/停止应用 |
| GET | `/v2/app_management/container` | 列出容器 |
| POST | `/v2/app_management/container` | 创建容器 |
| DELETE | `/v2/app_management/container/:id` | 删除容器 |
| PUT | `/v2/app_management/container/:id/status` | 启停容器 |
| GET | `/v2/app_management/image` | 列出镜像 |

### 5.5 CasaOS-MessageBus API (`/v2/message_bus`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/v2/message_bus/event_types` | 列出已注册的事件类型 |
| POST | `/v2/message_bus/event_types` | 注册事件类型 |
| GET | `/v2/message_bus/action_types` | 列出已注册的动作类型 |
| POST | `/v2/message_bus/action_types` | 注册动作类型 |
| POST | `/v2/message_bus/event/:source_id/:name` | 发布事件 |
| POST | `/v2/message_bus/action/:source_id/:name` | 触发动作 |
| GET | `/v2/message_bus/subscribe` | WebSocket 订阅事件/动作流 |

### 5.6 CasaOS 主服务 V1 API

> V1 API 使用 Echo 框架手动路由，受 JWT 认证保护（localhost 请求除外）。

#### 公开接口（无需认证）

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/ping` | — | 健康检查 |
| GET | `/v1/sys/version/current` | — | 返回当前版本号 |
| GET | `/v1/sys/debug` | `GetSystemConfigDebug` | 系统调试信息 |
| GET | `/v1/sys/version/check` | `GetSystemCheckVersion` | 版本更新检查 |
| GET | `/v1/recover/:type` | `GetRecoverStorage` | 云存储 OAuth 回调 |

#### 系统管理 (`/v1/sys`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/sys/version` | `GetSystemCheckVersion` | 版本检查 |
| POST | `/v1/sys/update` | `SystemUpdate` | 触发系统更新 |
| GET | `/v1/sys/hardware` | `GetSystemHardwareInfo` | 硬件信息 |
| GET | `/v1/sys/wsssh` | `WsSsh` | WebSocket SSH 终端 |
| POST | `/v1/sys/ssh-login` | `PostSshLogin` | SSH 登录验证 |
| GET | `/v1/sys/logs` | `GetCasaOSErrorLogs` | 错误日志 |
| POST | `/v1/sys/stop` | `PostKillCasaOS` | 停止 CasaOS |
| GET | `/v1/sys/utilization` | `GetSystemUtilization` | 系统资源利用率 |
| GET | `/v1/sys/proxy` | `GetSystemProxy` | 代理 URL |
| PUT | `/v1/sys/state/:state` | `PutSystemState` | 关机/重启 |
| GET | `/v1/sys/entry` | `GetSystemEntry` | 模块入口配置 |

#### 端口管理 (`/v1/port`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/port/` | `GetPort` | 获取可用端口 |
| GET | `/v1/port/state/:port` | `PortCheck` | 检查端口可用性 |

#### 文件管理 (`/v1/file`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/file` | `GetDownloadSingleFile` | 下载单个文件 |
| POST | `/v1/file` | `PostCreateFile` | 创建文件 |
| PUT | `/v1/file` | `PutFileContent` | 更新文件内容 |
| PUT | `/v1/file/name` | `RenamePath` | 重命名 |
| GET | `/v1/file/content` | `GetFilerContent` | 读取文件内容 |
| POST | `/v1/file/upload` | `PostFileUpload` | 上传文件 |
| GET | `/v1/file/upload` | `GetFileUpload` | 检查分块上传状态 |
| GET | `/v1/file/ws` | `ConnectWebSocket` | WebSocket 连接 |
| GET | `/v1/file/peers` | `GetPeers` | 已连接对等设备 |

#### 目录管理 (`/v1/folder`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/folder` | `DirPath` | 列出目录内容 |
| POST | `/v1/folder` | `MkdirAll` | 创建目录 |
| PUT | `/v1/folder/name` | `RenamePath` | 重命名目录 |
| GET | `/v1/folder/size` | `GetSize` | 目录大小 |
| GET | `/v1/folder/count` | `GetFileCount` | 文件数量 |

#### 批量操作 (`/v1/batch`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/batch` | `GetDownloadFile` | 批量下载 (zip/tar) |
| DELETE | `/v1/batch` | `DeleteFile` | 批量删除 |
| POST | `/v1/batch/task` | `PostOperateFileOrDir` | 创建复制/移动任务 |
| DELETE | `/v1/batch/:id/task` | `DeleteOperateFileOrDir` | 取消操作任务 |

#### 图片 (`/v1/image`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/image` | `GetFileImage` | 缩略图/原图 |

#### Samba (`/v1/samba`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/samba/connections` | `GetSambaConnectionsList` | 列出网络连接 |
| POST | `/v1/samba/connections` | `PostSambaConnectionsCreate` | 创建网络连接 |
| DELETE | `/v1/samba/connections/:id` | `DeleteSambaConnections` | 删除网络连接 |
| GET | `/v1/samba/shares` | `GetSambaSharesList` | 列出共享 |
| POST | `/v1/samba/shares` | `PostSambaSharesCreate` | 创建共享 |
| DELETE | `/v1/samba/shares/:id` | `DeleteSambaShares` | 删除共享 |
| GET | `/v1/samba/shares/status` | `GetSambaStatus` | Samba 服务状态 |

#### 通知 (`/v1/notify`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| POST | `/v1/notify/:path` | `PostNotifyMessage` | 发送通知 |
| POST | `/v1/notify/system_status` | `PostSystemStatusNotify` | 系统状态通知 |

#### 云存储 (`/v1/cloud`, `/v1/driver`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/cloud` | `ListStorages` | 列出已挂载云存储 |
| DELETE | `/v1/cloud` | `UmountStorage` | 卸载云存储 |
| GET | `/v1/driver` | `ListDriverInfo` | 可用驱动列表 |

#### 其他

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/other/search` | `GetSearchResult` | 搜索引擎代理 |
| ANY | `/v1/zt/*url` | `ZerotierProxy` | ZeroTier API 代理 |

### 5.7 CasaOS 主服务 V2 API (`/v2/casaos`)

> V2 API 基于 OpenAPI 3.0 规范 (`api/casaos/openapi.yaml`)，使用 `oapi-codegen` 生成服务端代码，集成 OpenAPI 请求验证中间件。

| 方法 | 路径 | OperationId | 说明 |
|------|------|-------------|------|
| GET | `/v2/casaos/health/services` | `getHealthServices` | 各 casaos-* 服务运行状态 |
| GET | `/v2/casaos/health/ports` | `getHealthPorts` | TCP/UDP 端口占用 |
| GET | `/v2/casaos/health/logs` | `getHealthlogs` | 日志压缩包下载 |
| GET | `/v2/casaos/file/test` | `getFileTest` | 文件方法测试 |
| GET | `/v2/casaos/file/upload` | `checkUploadChunk` | 检查分块上传状态 |
| POST | `/v2/casaos/file/upload` | `postUploadFile` | 分块上传文件 |
| GET | `/v2/casaos/zt/info` | `getZerotierInfo` | ZeroTier 节点信息 |
| PUT | `/v2/casaos/zt/:network_id/status` | `setZerotierNetworkStatus` | 设置 ZeroTier 网络状态 |

### 5.8 V3 文件服务与文档接口

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/v3/file?token=xxx&path=xxx` | Token 鉴权的文件直接访问 |
| GET | `/doc/v2/casaos` | Redoc API 文档页面 |
| GET | `/doc/v2/casaos/openapi.yaml` | OpenAPI YAML 规范文件 |

### 5.9 通用响应格式

**V1 API**:
```json
{
    "success": 200,
    "message": "ok",
    "data": { ... }
}
```

**V2 API** (OpenAPI 规范):
```json
{
    "message": "",
    "data": { ... }
}
```

---

## 6. 代码框架结构

### 6.1 CasaOS 主服务目录结构

```
CasaOS/                                    # 主服务仓库
├── main.go                                # 入口：初始化、路由注册、HTTP 服务启动
├── go.mod / go.sum                        # Go 模块依赖
├── Makefile                               # 构建 (前端 + 后端)
├── .goreleaser.yaml                       # 多架构发布 (amd64/arm64/arm-7)
│
├── api/                                   # API 定义
│   ├── index.html                         # Redoc 文档模板 (嵌入二进制)
│   └── casaos/
│       └── openapi.yaml                   # OpenAPI 3.0 规范 (V2 API)
│
├── build/                                 # 构建与部署
│   ├── scripts/
│   │   ├── setup/                         # 安装脚本 (Debian/Arch)
│   │   └── migration/                     # 版本迁移脚本
│   └── sysroot/
│       ├── etc/casaos/casaos.conf.sample  # 配置文件模板 (嵌入二进制)
│       └── usr/
│           ├── lib/systemd/system/        # systemd 服务单元
│           └── share/casaos/              # Shell 辅助脚本
│
├── cmd/                                   # 独立CLI工具
│   ├── message-bus-docgen/                # MessageBus 文档生成
│   └── migration-tool/                    # 版本迁移工具
│
├── codegen/                               # 自动生成代码 (go generate)
│   ├── casaos_api.go                      # OpenAPI → Go (types + server + spec)
│   └── message_bus/api.go                 # MessageBus 客户端
│
├── common/                                # 常量与事件定义
│   ├── constants.go                       # VERSION = "0.4.15", SERVICENAME = "casaos"
│   └── message.go                         # EventTypes 定义 (注册到 MessageBus)
│
├── drivers/                               # 云存储驱动
│   ├── base/                              # 驱动基础: HTTP 客户端、通用类型
│   ├── google_drive/                      # Google Drive: drive/meta/types/util
│   ├── onedrive/                          # OneDrive: drive/meta/util
│   └── dropbox/                           # Dropbox: drive/meta/types/util
│
├── internal/                              # 内部包 (不对外暴露)
│   ├── conf/                              # 内部配置: Config/Database/Scheme/Log
│   ├── driver/                            # 驱动接口: Driver/Meta/Reader/User
│   ├── op/                                # 驱动注册: RegisterDriver/GetDriverNew
│   └── sign/                              # HMAC 签名
│
├── model/                                 # 数据传输对象 (DTO)
│   ├── sys_common.go                      # Result, BaseInfo, ServerModel, APPModel
│   ├── zima.go                            # Path, DeviceInfo
│   ├── file.go                            # FileOperate, FileItem, FileUpdate
│   ├── obj.go                             # Obj 接口, FileStreamer
│   ├── drive.go                           # Drive (云存储)
│   ├── connections.go                     # Connections (网络连接 DTO)
│   ├── share.go                           # Shares (共享 DTO)
│   ├── search.go                          # SearchEngine
│   ├── notify/                            # 通知模型
│   └── ...
│
├── pkg/                                   # 可复用工具包
│   ├── config/                            # INI 配置加载 (InitSetup)
│   ├── sqlite/db.go                       # SQLite + GORM 初始化
│   ├── cache/                             # 内存缓存 (go-cache)
│   ├── samba/                             # Samba 集成
│   ├── utils/                             # 工具集
│   │   ├── file/                          # 文件操作
│   │   ├── httper/                        # HTTP 请求 (用于 rclone API)
│   │   ├── ip_helper/                     # IP 地址工具
│   │   ├── common_err/                    # 错误码
│   │   └── port/                          # 端口工具
│   └── ...
│
├── route/                                 # 路由层 (HTTP Handler)
│   ├── v1.go                              # V1 路由注册 + 中间件
│   ├── v2.go                              # V2 路由注册 + OpenAPI 验证
│   ├── init.go                            # 初始化: 网络挂载、设备信息
│   ├── v1/                                # V1 Handler 实现
│   │   ├── system.go                      # 系统管理
│   │   ├── file.go                        # 文件管理
│   │   ├── samba.go                       # Samba
│   │   ├── cloud.go                       # 云存储
│   │   ├── driver.go                      # 驱动
│   │   ├── notify.go                      # 通知
│   │   ├── image.go                       # 图片
│   │   ├── ssh.go                         # SSH 终端
│   │   ├── zerotier.go                    # ZeroTier
│   │   ├── recover.go                     # OAuth 回调
│   │   ├── port.go                        # 端口
│   │   └── other.go                       # 搜索
│   └── v2/                                # V2 Handler 实现
│       ├── health.go                      # 健康检查
│       ├── file.go                        # 文件上传
│       └── zt.go                          # ZeroTier
│
├── service/                               # 业务逻辑层
│   ├── service.go                         # Repository 接口 + 服务注册中心
│   ├── system.go                          # SystemService
│   ├── notify.go                          # NotifyServer
│   ├── shares.go                          # SharesService (Samba 共享)
│   ├── connections.go                     # ConnectionsService + PeerService
│   ├── storage.go                         # StorageService (rclone 云存储)
│   ├── casa.go                            # CasaService (版本检查)
│   ├── health.go                          # HealthService
│   ├── rely.go                            # RelyService (依赖关系)
│   ├── other.go                           # OtherService (搜索)
│   ├── file.go                            # 文件复制/移动操作
│   ├── file_upload.go                     # 分块上传
│   ├── socket.go                          # Peer 辅助
│   └── model/                             # 数据库模型 (GORM)
│       ├── o_notify.go                    # AppNotify
│       ├── o_shares.go                    # SharesDBModel
│       ├── o_connections.go               # ConnectionsDBModel
│       ├── o_drive.go                     # PeerDriveDBModel
│       └── o_rely.go                      # RelyDBModel
│
└── types/                                 # 类型定义和枚举常量
```

### 6.2 CasaOS-Common 目录结构

```
CasaOS-Common/                             # 共享库仓库
├── main.go                                # go:generate (mod_management OpenAPI)
├── interfaces.go                          # MigrationTool 接口
├── go.mod
│
├── external/                              # 外部服务客户端接口 (核心)
│   ├── common.go                          # getAddress() / ping() - 服务发现基础
│   ├── gateway.go                         # ManagementService 接口 - 路由注册
│   ├── user_service.go                    # GetPublicKey() - JWKS 公钥获取
│   ├── message_bus.go                     # GetMessageBusAddress() / PublishEventInSocket()
│   ├── share.go                           # ShareService - Samba 共享
│   ├── notify.go                          # NotifyService - 通知
│   ├── app_manage.go                      # AppManageService - 应用管理
│   └── gpu.go                             # NVIDIA GPU 信息
│
├── middleware/
│   └── echo.go                            # Cors() - 通用 CORS 中间件
│
├── model/                                 # 共享数据模型
│   ├── sys_common.go                      # Result
│   ├── device.go                          # DeviceInfo
│   ├── gateway.go                         # Route, ChangePortRequest
│   ├── app_info.go                        # ComposeAppWithStoreInfo
│   └── notify/application.go             # Application 通知模型
│
├── codegen/mod_management/                # 模块管理 API 客户端 (生成代码)
│
├── pkg/mod_management/                    # 模块管理 SDK
│
└── utils/                                 # 工具函数
    ├── jwt/                               # JWT: 生成/解析/验证/JWKS/中间件
    │   ├── jwt.go                         # Claims, GenerateToken, Validate
    │   └── jwt_helper.go                  # JWKS, JWT() echo中间件, GenerateKeyPair
    ├── http/                              # HTTP 工具
    │   ├── methods.go                     # Get/Post/Put/Delete (带超时)
    │   └── multiplexer.go                 # HandlerMultiplexer (路径前缀路由)
    ├── logger/log.go                      # zap + lumberjack 日志
    ├── command/command.go                 # Shell 命令执行
    ├── exec/exec.go                       # 安全命令执行 (safetext)
    ├── file/file.go                       # 文件操作集
    ├── port/port.go                       # 端口可用性检测
    ├── ssh/helper.go                      # SSH + WebSocket 终端
    ├── systemctl/systemctl.go             # systemd 服务管理
    ├── constants/paths.go                 # 系统路径常量
    ├── common_err/e.go                    # 错误码定义
    ├── version/                           # 版本解析和迁移状态
    ├── random/random.go                   # 随机字符串
    ├── time/utils.go                      # 时区工具
    └── idevice/os_release.go              # OS 信息读取
```

### 6.3 分层架构

CasaOS 主服务采用三层架构：

```
┌────────────────────────────────────────────────────────────────┐
│                   Route 层 (路由 / HTTP Handler)                │
│                                                                │
│  route/v1.go ─→ route/v1/*.go     (Echo 手动路由, V1 API)      │
│  route/v2.go ─→ route/v2/*.go     (OpenAPI codegen, V2 API)   │
│                                                                │
│  职责: 请求解析、参数校验、调用 Service、构造 HTTP 响应           │
└───────────────────────────┬────────────────────────────────────┘
                            │ 调用
                            ▼
┌────────────────────────────────────────────────────────────────┐
│                   Service 层 (业务逻辑)                         │
│                                                                │
│  service/service.go ─→ Repository 接口 (12 个 Service)         │
│                                                                │
│  职责: 业务逻辑、数据库操作、外部服务调用 (Gateway/MessageBus)   │
└───────────────────────────┬────────────────────────────────────┘
                            │ 使用
                            ▼
┌────────────────────────────────────────────────────────────────┐
│                   Data 层 (数据访问)                             │
│                                                                │
│  service/model/*.go ─→ GORM 模型 (5 张表)                      │
│  pkg/sqlite/db.go   ─→ 数据库连接和迁移                         │
│  CasaOS-Common/external/* ─→ 外部服务客户端                     │
│                                                                │
│  职责: 数据持久化、外部 API 调用、文件系统操作                    │
└────────────────────────────────────────────────────────────────┘
```

### 6.4 服务注册中心

所有 Service 通过 `Repository` 接口统一管理（`service/service.go`）：

```go
type Repository interface {
    Casa() CasaService                          // 版本检查
    Connections() ConnectionsService            // Samba 连接管理
    Gateway() external.ManagementService        // Gateway 路由管理 (CasaOS-Common)
    Health() HealthService                      // 健康检查
    Notify() NotifyServer                       // 通知管理
    Rely() RelyService                          // 依赖关系
    Shares() SharesService                      // Samba 共享管理
    System() SystemService                      // 系统管理
    Storage() StorageService                    // 云存储 (rclone)
    MessageBus() *message_bus.ClientWithResponses // MessageBus 客户端 (CasaOS-Common)
    Peer() PeerService                          // 设备发现
    Other() OtherService                        // 搜索
}
```

`NewService(db, RuntimePath)` 在 `init()` 阶段一次性创建所有 Service 实例，通过 `service.MyService` 全局变量共享。Gateway 和 MessageBus 客户端由 `CasaOS-Common/external` 包提供，通过运行时路径发现对应服务地址。

### 6.5 启动流程

```
程序启动 (main.go)
  │
  ├── init() 阶段
  │   ├── 解析命令行参数 (-c 配置路径, -db 数据库路径, -v 版本)
  │   ├── config.InitSetup() → 加载嵌入的 casaos.conf.sample，创建/读取 INI 配置
  │   ├── logger.LogInit() → 初始化 zap + lumberjack 日志
  │   ├── sqlite.GetDb() → 打开 SQLite，执行 AutoMigrate (4 张表)
  │   ├── service.NewService(db, runtimePath)
  │   │   ├── external.NewManagementService() → 等待 management.url，连接 Gateway
  │   │   ├── NewConnectionsService(db) / NewSharesService(db) / ...
  │   │   └── NewSystemService() / NewHealthService() / ...
  │   ├── cache.Init() → 初始化 go-cache 内存缓存
  │   ├── GetCPUThermalZone() → 扫描 /sys 寻找 CPU 温度传感器
  │   └── route.InitFunction()
  │       ├── InitNetworkMount() → (goroutine) 恢复 Samba 挂载和 rclone 云存储
  │       └── InitInfo() → (goroutine) 生成设备基础信息 (Hash/Version/Channel)
  │
  └── main() 阶段
      ├── 创建 4 个 HTTP Handler:
      │   ├── V1 Router (Echo + CORS + Gzip + Recover + Logger + JWT)
      │   ├── V2 Router (Echo + CORS + Gzip + Logger + JWT + OpenAPI Validator)
      │   ├── V3 File Handler (Token 参数鉴权 + 文件直接下载)
      │   └── V2 Doc Router (Redoc HTML + OpenAPI YAML)
      │
      ├── HandlerMultiplexer 组合: {"v1": v1, "v2": v2, "v3": v3File, "doc": v2Doc}
      │
      ├── 启动 Cron (每 5 秒推送硬件状态到 MessageBus)
      │
      ├── net.Listen("tcp", "127.0.0.1:0") → 随机端口
      │
      ├── 向 Gateway 注册 17 条路由 (POST /v1/gateway/routes × 17)
      │   /v1/sys, /v1/port, /v1/file, /v1/folder, /v1/batch, /v1/image,
      │   /v1/samba, /v1/notify, /v1/driver, /v1/cloud, /v1/recover,
      │   /v1/other, /v1/zt, /v1/test, /v2/casaos, /doc/v2/casaos, /v3/file
      │
      ├── 向 MessageBus 注册 3 个事件类型 (重试 10 次)
      │
      ├── 写入 casaos.url (供其他服务发现)
      │
      ├── 执行 start.d/ 目录下的启动脚本
      │
      ├── daemon.SdNotify(SdNotifyReady) → 通知 systemd 就绪
      │
      └── http.Server.Serve(listener) → 开始服务
```

### 6.6 中间件链

**V1 Router**:
```
请求 → CORS → Gzip → Recover → Logger → JWT(跳过localhost) → Handler
```

**V2 Router**:
```
请求 → CORS → Gzip → Logger → JWT(跳过localhost) → OpenAPI Validator(跳过multipart) → Handler
```

JWT 中间件实现（`route/v1.go`）：
- Token 来源: `Authorization` Header 或 `token` Query 参数
- 验证方式: 调用 `CasaOS-Common/utils/jwt.Validate()`，从 UserService JWKS 端点获取公钥
- localhost 请求 (`127.0.0.1` / `::1`) 跳过认证
- 验证通过后将 `user_id` 注入请求 Header

---

## 7. 后端数据库实现

### 7.1 技术选型

| 项目 | 选型 | 说明 |
|------|------|------|
| 数据库引擎 | SQLite | 轻量嵌入式数据库，适合单机家庭服务器，无需独立数据库进程 |
| ORM 框架 | GORM | Go 语言最流行的 ORM，支持 AutoMigrate、关联、钩子等 |
| SQLite 驱动 | glebarez/sqlite | 纯 Go 实现的 SQLite 驱动，支持 CGO-free 编译 |

### 7.2 数据库初始化

数据库初始化在 `pkg/sqlite/db.go` 中实现，采用单例模式：

```go
var gdb *gorm.DB

func GetDb(dbPath string) *gorm.DB {
    if gdb != nil {
        return gdb
    }

    file.IsNotExistMkDir(dbPath)
    db, err := gorm.Open(sqlite.Open(dbPath+"/casaOS.db"), &gorm.Config{})
    if err != nil {
        panic("sqlite connect error")
    }

    c, _ := db.DB()
    c.SetMaxIdleConns(10)                    // 最大空闲连接数
    c.SetMaxOpenConns(1)                     // 最大打开连接数 (SQLite 单写者限制)
    c.SetConnMaxIdleTime(time.Second * 1000) // 连接最大空闲时间

    gdb = db

    // 自动建表/迁移
    err = db.AutoMigrate(
        &model2.AppNotify{},
        model2.SharesDBModel{},
        model2.ConnectionsDBModel{},
        model2.PeerDriveDBModel{},
    )

    // 清理旧版遗留表
    db.Exec("DROP TABLE IF EXISTS o_application")
    db.Exec("DROP TABLE IF EXISTS o_friend")
    db.Exec("DROP TABLE IF EXISTS o_person_download")
    db.Exec("DROP TABLE IF EXISTS o_person_down_record")

    return db
}
```

**设计要点**:
- **单例模式**: `gdb` 全局变量确保数据库连接只初始化一次
- **单写者**: `MaxOpenConns=1` 适配 SQLite 的并发写入限制
- **自动迁移**: GORM `AutoMigrate` 根据 struct 定义自动创建/更新表结构
- **向下兼容**: 启动时删除旧版本（v0.3.x 及之前）遗留的废弃表

**数据库文件位置**: `/var/lib/casaos/db/casaOS.db`（通过 `-db` 参数或配置文件指定）

### 7.3 数据库表结构

#### 7.3.1 通知表 `o_notify`

| 字段 | 类型 | 属性 | 说明 |
|------|------|------|------|
| custom_id | string | **PK** | 自定义唯一标识 |
| id | string | | 通知 ID |
| name | string | | 通知名称（如应用名） |
| icon | string | | 图标 URL |
| state | int | | 0=变动中未读, 1=未读, 2=已读 |
| type | int | | 通知类型 |
| class | int | | 通知分类 |
| message | string | | 通知内容 |
| created_at | string | | 创建时间 |
| updated_at | string | | 更新时间 |

**模型定义** (`service/model/o_notify.go`):
```go
type AppNotify struct {
    State     int    `json:"state"`
    Message   string `json:"message"`
    CreatedAt string `json:"created_at"`
    UpdatedAt string `json:"updated_at"`
    Id        string `json:"id"`
    Type      int    `json:"type"`
    Icon      string `json:"icon"`
    Name      string `json:"name"`
    Class     int    `json:"class"`
    CustomId  string `gorm:"column:custom_id;primary_key" json:"custom_id"`
}

func (p *AppNotify) TableName() string { return "o_notify" }
```

#### 7.3.2 Samba 共享表 `o_shares`

| 字段 | 类型 | 属性 | 说明 |
|------|------|------|------|
| id | uint | **PK**, 自增 | 共享 ID |
| name | string | | 共享名称 |
| path | string | | 共享目录路径 |
| anonymous | bool | | 是否允许匿名访问 |
| created | int64 | autoCreateTime | 创建时间戳 |
| updated | int64 | autoUpdateTime | 更新时间戳 |

**模型定义** (`service/model/o_shares.go`):
```go
type SharesDBModel struct {
    ID        uint   `gorm:"column:id;primary_key" json:"id"`
    Anonymous bool   `json:"anonymous"`
    Path      string `json:"path"`
    Name      string `json:"name"`
    Updated   int64  `gorm:"autoUpdateTime"`
    Created   int64  `gorm:"autoCreateTime"`
}

func (p *SharesDBModel) TableName() string { return "o_shares" }
```

#### 7.3.3 网络连接表 `o_connections`

| 字段 | 类型 | 属性 | 说明 |
|------|------|------|------|
| id | uint | **PK**, 自增 | 连接 ID |
| username | string | | 远程用户名 |
| password | string | | 远程密码 |
| host | string | | 远程主机地址 |
| port | string | | 远程端口 |
| status | string | | 连接状态 |
| directories | string | | 目录列表 (JSON 字符串数组) |
| mount_point | string | | 本地挂载点父目录 |
| created | int64 | autoCreateTime | 创建时间戳 |
| updated | int64 | autoUpdateTime | 更新时间戳 |

**模型定义** (`service/model/o_connections.go`):
```go
type ConnectionsDBModel struct {
    ID          uint   `gorm:"column:id;primary_key" json:"id"`
    Updated     int64  `gorm:"autoUpdateTime"`
    Created     int64  `gorm:"autoCreateTime"`
    Username    string `json:"username"`
    Password    string `json:"password"`
    Host        string `json:"host"`
    Port        string `json:"port"`
    Status      string `json:"status"`
    Directories string `json:"directories"`
    MountPoint  string `json:"mount_point"`
}

func (p *ConnectionsDBModel) TableName() string { return "o_connections" }
```

#### 7.3.4 对等设备表 `peer_drive_db_models`

| 字段 | 类型 | 属性 | 说明 |
|------|------|------|------|
| id | string | **PK** | 设备唯一标识 |
| user_agent | string | | 浏览器 User-Agent |
| display_name | string | | 显示名称 |
| device_name | string | | 设备名称 |
| model | string | | 设备型号 |
| ip | string | | IP 地址 |
| os | string | | 操作系统 |
| browser | string | | 浏览器 |
| online | bool | `gorm:"-"` | 是否在线（不持久化） |
| created | int64 | autoCreateTime | 创建时间戳 |
| updated | int64 | autoUpdateTime | 更新时间戳 |

**模型定义** (`service/model/o_drive.go`):
```go
type PeerDriveDBModel struct {
    ID          string `gorm:"column:id;primary_key" json:"id"`
    Updated     int64  `gorm:"autoUpdateTime"`
    Created     int64  `gorm:"autoCreateTime"`
    UserAgent   string `json:"user_agent"`
    DisplayName string `json:"display_name"`
    DeviceName  string `json:"device_name"`
    Model       string `json:"model"`
    IP          string `json:"ip"`
    OS          string `json:"os"`
    Browser     string `json:"browser"`
    Online      bool   `gorm:"-" json:"online"`
}
```

#### 7.3.5 依赖关系表 `o_rely`

| 字段 | 类型 | 属性 | 说明 |
|------|------|------|------|
| id | uint | **PK**, 自增 | 记录 ID |
| custom_id | string | | 自定义 ID |
| container_custom_id | string | | 容器自定义 ID |
| container_id | string | | 容器 ID |
| type | int | | 类型（暂未使用） |
| created_at | time.Time | `<-:create` | 创建时间 |
| updated_at | time.Time | `<-:create;<-:update` | 更新时间 |

> 注：`RelyDBModel` 定义了模型但**未在 AutoMigrate 中注册**，需手动创建表或在未来版本中补充。

### 7.4 数据访问模式

CasaOS 的数据库操作直接在 Service 层中通过 GORM 执行，没有独立的 Repository/DAO 层。DB 实例通过 `NewService(db, ...)` 注入到需要数据库的 Service 中：

```go
func NewService(db *gorm.DB, RuntimePath string) Repository {
    return &store{
        connections: NewConnectionsService(db),   // 需要 DB
        notify:      NewNotifyService(db),         // 需要 DB
        rely:        NewRelyService(db),            // 需要 DB
        shares:      NewSharesService(db),          // 需要 DB
        peer:        NewPeerService(db),            // 需要 DB
        system:      NewSystemService(),            // 不需要 DB
        health:      NewHealthService(),            // 不需要 DB
        casa:        NewCasaService(),              // 不需要 DB
        storage:     NewStorageService(),           // 不需要 DB (使用 rclone API)
        other:       NewOtherService(),             // 不需要 DB
        gateway:     gatewayManagement,             // CasaOS-Common Gateway 客户端
    }
}
```

典型的 CRUD 操作模式：

```go
// 查询列表
func (s *sharesService) GetSharesList() []model.SharesDBModel {
    var shares []model.SharesDBModel
    s.db.Find(&shares)
    return shares
}

// 创建记录
func (s *sharesService) CreateShare(share model.SharesDBModel) {
    s.db.Create(&share)
}

// 删除记录
func (s *sharesService) DeleteShare(id uint) {
    s.db.Delete(&model.SharesDBModel{}, id)
}
```

### 7.5 数据存储策略总览

CasaOS 采用混合存储策略：

| 存储方式 | 用途 | 位置 |
|----------|------|------|
| **SQLite** | 通知、Samba 共享/连接、设备信息 | `/var/lib/casaos/db/casaOS.db` |
| **INI 配置文件** | 应用配置 | `/etc/casaos/casaos.conf` |
| **JSON 文件** | 设备基础信息、应用排序、模块入口 | `/var/lib/casaos/` |
| **内存缓存** | CPU 温度路径、版本信息 | `go-cache` (进程内存) |
| **文件系统** | 用户数据、日志、运行时 URL | `/var/lib/casaos/`、`/var/log/casaos/`、`/var/run/casaos/` |
| **Samba 配置** | 共享配置 | `/etc/samba/smb.casa.conf` (include 到 smb.conf) |
| **rclone 配置** | 云存储配置 | rclone 自身配置文件 |

### 7.6 版本迁移机制

CasaOS 提供了完善的版本迁移框架：

**迁移接口** (定义在 `CasaOS-Common/interfaces.go`):
```go
type MigrationTool interface {
    IsMigrationNeeded() (bool, error)
    PreMigrate() error
    Migrate() error
    PostMigrate() error
}
```

**迁移工具**: `cmd/migration-tool/main.go` 作为独立二进制，在系统升级时执行。

**版本管理** (定义在 `CasaOS-Common/utils/version/`):
- `ParseVersion()` 解析语义化版本号
- `Compare()` 比较版本大小
- `GlobalMigrationStatus` 跟踪迁移状态

**Shell 迁移脚本**: `build/scripts/migration/script.d/` 中的脚本在包升级时自动执行，按版本号顺序运行迁移。

### 7.7 系统路径常量

定义在 `CasaOS-Common/utils/constants/paths.go`，所有微服务共享：

| 常量 | 值 | 用途 |
|------|-----|------|
| `DefaultConfigPath` | `/etc/casaos` | 配置文件目录 |
| `DefaultConstantPath` | `/usr/share/casaos` | 静态资源/脚本 |
| `DefaultDataPath` | `/var/lib/casaos` | 持久化数据 |
| `DefaultFilePath` | `/var/lib/casaos/files` | 用户文件 |
| `DefaultLogPath` | `/var/log/casaos` | 日志文件 |
| `DefaultRuntimePath` | `/var/run/casaos` | 运行时文件 (服务发现) |

---

> **文档版本**: 2.0  
> **CasaOS 版本**: v0.4.15 (commit 63f0148)  
> **CasaOS-Common 版本**: commit 909dcbd
