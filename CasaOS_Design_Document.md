# CasaOS 系统设计文档

> **仓库地址**: https://github.com/IceWhaleTech/CasaOS  
> **分支**: main  
> **Commit**: 63f0148 (`update version (#2098)`)  
> **版本**: v0.4.15  
> **编写日期**: 2026-03-02

---

## 目录

1. [项目概述](#1-项目概述)
2. [系统架构图](#2-系统架构图)
3. [系统功能模块总览](#3-系统功能模块总览)
4. [各功能模块详细介绍](#4-各功能模块详细介绍)
5. [API 接口梳理](#5-api-接口梳理)
6. [代码框架结构介绍](#6-代码框架结构介绍)
7. [后端数据库实现介绍](#7-后端数据库实现介绍)

---

## 1. 项目概述

CasaOS 是由 IceWhale Technology 开发的开源家庭云操作系统，旨在为个人或家庭用户提供简洁易用的私有云解决方案。它运行在 Linux 设备（如 ZimaBoard、树莓派、x86 小主机等）上，提供文件管理、云存储挂载、Samba 网络共享、系统监控、ZeroTier 组网等能力。

### 技术栈

| 类别 | 技术选型 |
|------|----------|
| 编程语言 | Go 1.21 |
| Web 框架 | Echo v4 |
| ORM | GORM |
| 数据库 | SQLite |
| API 规范 | OpenAPI 3.0 (oapi-codegen) |
| 消息总线 | CasaOS-MessageBus (事件驱动) |
| 定时任务 | robfig/cron |
| 系统信息采集 | gopsutil v3 |
| 文件压缩 | mholt/archiver |
| 网络共享 | go-smb2 (Samba/CIFS) |
| 云存储 | Google Drive / OneDrive / Dropbox |
| 部署方式 | systemd 服务 |

---

## 2. 系统架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              用户 / 浏览器                                  │
│                          (CasaOS-UI 前端界面)                               │
└─────────────────────────────────┬───────────────────────────────────────────┘
                                  │ HTTP/WebSocket
                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          CasaOS-Gateway                                     │
│                     (反向代理 / 统一入口网关)                                │
│         ┌──────────┬──────────┬──────────┬──────────┬──────────┐           │
│         │ /v1/*    │ /v2/*    │ /v3/file │ /doc/*   │ 其他服务  │           │
│         └────┬─────┴────┬─────┴────┬─────┴────┬─────┴──────────┘           │
└──────────────┼──────────┼──────────┼──────────┼────────────────────────────┘
               │          │          │          │
               ▼          ▼          ▼          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      CasaOS 主服务 (本仓库)                                 │
│                   监听 127.0.0.1:随机端口                                   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                   HTTP Handler Multiplexer                          │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │    │
│  │  │ V1 Router│  │ V2 Router│  │ V3 File  │  │ V2 Doc Router    │   │    │
│  │  │ (Echo)   │  │ (OpenAPI)│  │ Handler  │  │ (Redoc UI)       │   │    │
│  │  └─────┬────┘  └─────┬────┘  └─────┬────┘  └──────────────────┘   │    │
│  └────────┼──────────────┼─────────────┼──────────────────────────────┘    │
│           │              │             │                                    │
│  ┌────────▼──────────────▼─────────────▼──────────────────────────────┐    │
│  │                     中间件层 (Middleware)                           │    │
│  │  ┌──────┐ ┌──────┐ ┌────────┐ ┌───────────┐ ┌─────────────────┐  │    │
│  │  │ CORS │ │ Gzip │ │Recover │ │  Logger   │ │ JWT Auth        │  │    │
│  │  └──────┘ └──────┘ └────────┘ └───────────┘ │(跳过 localhost) │  │    │
│  │                                              └─────────────────┘  │    │
│  └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────┐    │
│  │                     Service 层 (业务逻辑)                         │    │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────────┐    │    │
│  │  │  System   │ │   File    │ │  Storage  │ │  Connections  │    │    │
│  │  │  Service  │ │  Upload   │ │  Service  │ │   Service     │    │    │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────────┘    │    │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────────┐    │    │
│  │  │  Shares   │ │  Notify   │ │   Casa    │ │    Health     │    │    │
│  │  │  Service  │ │  Service  │ │  Service  │ │   Service     │    │    │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────────┘    │    │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐                      │    │
│  │  │   Peer    │ │   Other   │ │   Rely    │                      │    │
│  │  │  Service  │ │  Service  │ │  Service  │                      │    │
│  │  └───────────┘ └───────────┘ └───────────┘                      │    │
│  └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────┐    │
│  │                      数据层 (Data Layer)                          │    │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │    │
│  │  │  SQLite (GORM)  │  │  In-Memory      │  │  File System    │  │    │
│  │  │  casaOS.db      │  │  Cache          │  │  Config (INI)   │  │    │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘  │    │
│  └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────┐    │
│  │                   外部集成 (External Integration)                  │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐    │    │
│  │  │ Message  │ │ Gateway  │ │ ZeroTier │ │  Cloud Drivers   │    │    │
│  │  │   Bus    │ │ Service  │ │  Proxy   │ │ GDrive/OneDrive  │    │    │
│  │  │          │ │          │ │          │ │ Dropbox          │    │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────────┘    │    │
│  └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                  │
         ┌────────────────────────┼────────────────────────┐
         ▼                        ▼                        ▼
┌─────────────────┐  ┌──────────────────────┐  ┌──────────────────────┐
│  CasaOS-Common  │  │  CasaOS-MessageBus   │  │  CasaOS-UserService  │
│  (公共库/JWT)    │  │  (事件订阅/发布)       │  │  (用户认证/管理)      │
└─────────────────┘  └──────────────────────┘  └──────────────────────┘
```

### 架构说明

CasaOS 采用 **微服务网关架构**，核心设计思路如下:

1. **Gateway 统一入口**: 所有外部请求经由 CasaOS-Gateway 反向代理进入，按路径前缀分发到对应的后端服务。
2. **本地监听**: CasaOS 主服务绑定在 `127.0.0.1` 的随机端口上，不直接暴露给外部网络，安全性由 Gateway 保障。
3. **路由注册**: 启动时，CasaOS 向 Gateway 注册自身处理的路由路径列表（如 `/v1/sys`、`/v2/casaos` 等）。
4. **事件驱动**: 通过 CasaOS-MessageBus 实现跨服务的事件订阅与发布机制（如系统资源利用率变化、文件操作事件等）。
5. **JWT 认证**: 基于 ECDSA 的 JWT 令牌认证，公钥从 CasaOS-Common 运行时路径获取，本地请求 (localhost) 跳过认证。

---

## 3. 系统功能模块总览

```
CasaOS
├── 系统管理模块 (System)
│   ├── 硬件信息采集
│   ├── 系统资源监控 (CPU/内存/磁盘/网络)
│   ├── 系统版本管理与更新
│   ├── 系统电源控制 (重启/关机)
│   ├── SSH 终端 (WebSocket)
│   └── 日志管理
│
├── 文件管理模块 (File)
│   ├── 文件/目录浏览
│   ├── 文件上传 (分块上传)
│   ├── 文件下载 (单文件/批量打包)
│   ├── 文件创建/编辑/重命名/删除
│   ├── 文件/目录复制/移动
│   ├── 图片缩略图生成
│   └── WebSocket 实时通信
│
├── 网络存储模块 (Samba/CIFS)
│   ├── Samba 共享管理 (创建/删除/列表)
│   ├── 网络连接管理 (创建/删除/挂载)
│   └── Samba 服务状态监控
│
├── 云存储模块 (Cloud Storage)
│   ├── 云存储驱动管理
│   │   ├── Google Drive
│   │   ├── OneDrive
│   │   └── Dropbox
│   ├── 云存储挂载/卸载
│   └── OAuth 认证回调
│
├── 通知模块 (Notify)
│   ├── 应用通知管理
│   ├── 系统状态通知
│   └── MessageBus 事件发布
│
├── 健康检查模块 (Health)
│   ├── 服务状态检查
│   ├── 端口占用检查
│   └── 日志下载
│
├── ZeroTier 网络模块 (ZeroTier)
│   ├── ZeroTier 信息查询
│   ├── 网络状态管理
│   └── API 代理
│
├── 设备发现模块 (Peer)
│   ├── 对等设备发现
│   └── 设备信息管理
│
└── 其他模块
    ├── 搜索引擎代理
    └── 端口可用性检测
```

---

## 4. 各功能模块详细介绍

### 4.1 系统管理模块 (SystemService)

**源文件**: `service/system.go`

系统管理模块是 CasaOS 的核心模块，负责整个系统的信息采集、监控和控制。

| 功能 | 方法 | 说明 |
|------|------|------|
| 硬件信息 | `GetSystemHardwareInfo` | 采集 CPU 信息、内存容量、磁盘用量、网络接口等硬件详情 |
| CPU 监控 | `GetCpuPercent` / `GetCpuInfo` / `GetCpuCoreNum` | 获取 CPU 使用率、型号信息、核心数量 |
| CPU 温度 | `GetCPUTemperature` | 读取 `/sys/devices/virtual/thermal/thermal_zone*/temp` 获取 CPU 温度 |
| CPU 功耗 | `GetCPUPower` | 读取 Intel RAPL 接口获取 CPU 功耗 |
| 内存监控 | `GetMemInfo` | 获取总内存、可用内存、已用内存、使用率 |
| 磁盘监控 | `GetDiskInfo` | 获取根分区的磁盘使用情况 |
| 网络监控 | `GetNetInfo` / `GetNet` | 获取网络接口 IO 统计和物理网卡列表 |
| 系统信息 | `GetSysInfo` / `GetDeviceInfo` | 获取主机信息、设备名称、IP 地址、设备型号、序列号 |
| 设备树 | `GetDeviceTree` | 通过 shell 脚本获取设备树信息 |
| 目录操作 | `GetDirPath` / `MkdirAll` / `RenameFile` / `CreateFile` | 文件系统基础操作 |
| 版本管理 | `UpdateSystemVersion` | 执行远程更新脚本进行系统升级 |
| 电源控制 | `SystemReboot` / `SystemShutdown` | 通过 `init 6/0` 命令实现重启和关机 |
| 日志 | `GetCasaOSLogs` | 读取日志文件内容 |
| 系统入口 | `GetSystemEntry` / `GenreateSystemEntry` | 读取模块 entry.json 生成前端入口配置 |
| SSH 终端 | `WsSsh` | 通过 WebSocket 提供浏览器内 SSH 终端 |

**定时任务**: 系统使用 `cron` 每 5 秒执行 `SendAllHardwareStatusBySocket`，通过 WebSocket 推送硬件状态数据给前端。

### 4.2 文件管理模块

**源文件**: `route/v1/file.go`, `service/file.go`, `service/file_upload.go`

文件管理是 CasaOS 的核心用户功能，提供完整的文件生命周期管理。

| 功能 | 说明 |
|------|------|
| 目录浏览 | 列出指定路径下的文件和目录，返回名称、大小、类型、修改时间等信息 |
| 文件上传 | 支持分块上传（chunked upload），检查已上传分块避免重复传输 |
| 文件下载 | 单文件直接下载，批量文件打包为 zip/tar 后下载 |
| 文件创建 | 创建空文件 |
| 文件编辑 | 读取和写入文件内容 |
| 重命名 | 文件和目录的重命名操作 |
| 删除 | 支持批量删除文件和目录 |
| 复制/移动 | 异步执行的文件复制和移动操作，支持进度追踪和任务取消 |
| 目录创建 | 递归创建目录 |
| 目录大小 | 计算目录总大小 |
| 文件计数 | 统计目录下的文件数量 |
| 图片缩略图 | 使用 `disintegration/imaging` 库生成缩略图，支持 EXIF 方向校正 |
| WebSocket | 实时通信通道，用于文件操作进度推送 |

**文件操作模型** (`model.FileOperate`):

```go
type FileOperate struct {
    Type          string     // 操作类型: copy/move
    Item          []FileItem // 操作的文件列表
    TotalSize     int64      // 总大小
    ProcessedSize int64      // 已处理大小
    To            string     // 目标路径
    Style         string     // 冲突处理策略
    Finished      bool       // 是否完成
}
```

### 4.3 网络存储模块 (Samba/CIFS)

**源文件**: `service/shares.go`, `service/connections.go`, `pkg/samba/`

| 功能 | 说明 |
|------|------|
| Samba 共享 | 创建、删除、列出本机 Samba 共享目录 |
| 共享配置 | 管理 Samba 配置文件，支持匿名访问设置 |
| 网络连接 | 添加、删除远程 CIFS/SMB 网络连接 |
| 自动挂载 | 使用 `go-smb2` 库实现 SMB 协议连接和目录挂载 |
| 状态监控 | 检查 Samba 服务运行状态 |

**连接流程**: 用户创建网络连接 → 验证凭据 → 挂载远程共享到本地目录 → 文件管理模块统一管理已挂载目录。

### 4.4 云存储模块 (Cloud Storage)

**源文件**: `drivers/google_drive/`, `drivers/onedrive/`, `drivers/dropbox/`, `service/storage.go`

CasaOS 集成了三大主流云存储服务:

| 云服务 | 认证方式 | 功能 |
|--------|----------|------|
| Google Drive | OAuth 2.0 | 文件列表、上传、下载、挂载 |
| OneDrive | OAuth 2.0 | 文件列表、上传、下载、挂载 |
| Dropbox | OAuth 2.0 | 文件列表、上传、下载、挂载 |

**驱动架构** (`internal/driver/`):

```go
type Driver interface {
    Meta                         // 元信息: 配置、初始化、销毁
    Reader                       // 文件读取: 列出目录内容
    User                         // 用户信息
    Other(ctx, args) (interface{}, error)  // 扩展方法
}
```

**存储挂载**: 通过 `rclone` 工具实现云存储到本地文件系统的挂载，`StorageService` 负责 rclone 的配置生成和进程管理。

### 4.5 通知模块 (NotifyService)

**源文件**: `service/notify.go`

| 功能 | 说明 |
|------|------|
| 应用通知 | CRUD 操作管理应用级通知消息 |
| 系统通知 | 系统状态变更通知 |
| 消息总线 | 通过 CasaOS-MessageBus 发布事件，支持跨服务通知 |
| 文件操作通知 | 文件复制/移动的进度和完成通知 |

**事件类型** (注册到 MessageBus):

| 事件名称 | 说明 |
|----------|------|
| `casaos:system:utilization` | 系统资源利用率变化 |
| `casaos:file:recover` | 云存储 OAuth 回调 |
| `casaos:file:operate` | 文件操作进度 |

### 4.6 健康检查模块 (HealthService)

**源文件**: `service/health.go`, `route/v2/health.go`

| 功能 | 说明 |
|------|------|
| 服务状态 | 检查所有 `casaos-*` systemd 服务的运行状态，返回 running/not_running 列表 |
| 端口检查 | 扫描系统中 TCP/UDP 端口占用情况 |
| 日志打包 | 收集所有日志并打包为 zip 文件下载 |

### 4.7 ZeroTier 网络模块

**源文件**: `route/v1/zerotier.go`, `route/v2/zt.go`

| 功能 | 说明 |
|------|------|
| API 代理 | 将请求代理转发到本地 ZeroTier 服务 API |
| 网络信息 | 获取 ZeroTier 节点 ID、名称、在线状态 |
| 网络状态 | 设置 ZeroTier 网络的上线/下线状态 |

### 4.8 设备发现模块 (PeerService)

**源文件**: `service/connections.go`（Peer 部分）

| 功能 | 说明 |
|------|------|
| 设备发现 | 通过 User-Agent 和网络信息发现局域网内的对等设备 |
| 设备注册 | 将发现的设备信息持久化到数据库 |
| 设备查询 | 根据 ID、名称等条件查询已知设备 |

### 4.9 其他模块

**CasaService** (`service/casa.go`): 从远程 API (`api.casaos.io`) 获取最新版本信息，使用缓存避免频繁请求。

**OtherService** (`service/other.go`): 搜索引擎代理，支持 Bing、Google、百度等搜索引擎的搜索请求转发。

**RelyService** (`service/rely.go`): 管理应用间的依赖关系记录。

**端口服务**: 检测端口可用性和获取可用端口号。

---

## 5. API 接口梳理

### 5.1 V1 API 接口 (路径前缀: `/v1`)

> V1 API 使用 Echo 框架手动注册路由，受 JWT 认证保护（localhost 请求除外）。

#### 5.1.1 公开接口（无需认证）

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/ping` | inline | 健康检查 pong 响应 |
| GET | `/v1/sys/version/current` | inline | 返回当前版本号字符串 |
| GET | `/v1/sys/debug` | `GetSystemConfigDebug` | 获取系统调试信息 |
| GET | `/v1/sys/version/check` | `GetSystemCheckVersion` | 检查版本更新 |
| GET | `/v1/recover/:type` | `GetRecoverStorage` | 云存储 OAuth 回调处理 |

#### 5.1.2 系统管理接口 (`/v1/sys`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/sys/version` | `GetSystemCheckVersion` | 检查系统版本更新 |
| POST | `/v1/sys/update` | `SystemUpdate` | 触发系统更新 |
| GET | `/v1/sys/hardware` | `GetSystemHardwareInfo` | 获取硬件信息 |
| GET | `/v1/sys/wsssh` | `WsSsh` | WebSocket SSH 终端 |
| POST | `/v1/sys/ssh-login` | `PostSshLogin` | SSH 登录验证 |
| GET | `/v1/sys/logs` | `GetCasaOSErrorLogs` | 获取错误日志 |
| POST | `/v1/sys/stop` | `PostKillCasaOS` | 停止 CasaOS 服务 |
| GET | `/v1/sys/utilization` | `GetSystemUtilization` | 获取系统资源利用率 (CPU/内存/磁盘/网络) |
| GET | `/v1/sys/proxy` | `GetSystemProxy` | 获取代理 URL |
| PUT | `/v1/sys/state/:state` | `PutSystemState` | 设置系统状态 (关机=off / 重启=restart) |
| GET | `/v1/sys/entry` | `GetSystemEntry` | 获取系统模块入口配置 |

#### 5.1.3 端口管理接口 (`/v1/port`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/port/` | `GetPort` | 获取可用端口 |
| GET | `/v1/port/state/:port` | `PortCheck` | 检查指定端口是否可用 |

#### 5.1.4 文件管理接口 (`/v1/file`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/file` | `GetDownloadSingleFile` | 下载单个文件 |
| POST | `/v1/file` | `PostCreateFile` | 创建新文件 |
| PUT | `/v1/file` | `PutFileContent` | 更新文件内容 |
| PUT | `/v1/file/name` | `RenamePath` | 重命名文件 |
| GET | `/v1/file/content` | `GetFilerContent` | 读取文件内容 |
| POST | `/v1/file/upload` | `PostFileUpload` | 上传文件 |
| GET | `/v1/file/upload` | `GetFileUpload` | 检查分块上传状态 |
| GET | `/v1/file/ws` | `ConnectWebSocket` | 文件 WebSocket 连接 |
| GET | `/v1/file/peers` | `GetPeers` | 获取已连接的对等设备 |

#### 5.1.5 目录管理接口 (`/v1/folder`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/folder` | `DirPath` | 列出目录内容 |
| POST | `/v1/folder` | `MkdirAll` | 递归创建目录 |
| PUT | `/v1/folder/name` | `RenamePath` | 重命名目录 |
| GET | `/v1/folder/size` | `GetSize` | 获取目录大小 |
| GET | `/v1/folder/count` | `GetFileCount` | 获取文件数量 |

#### 5.1.6 批量文件操作接口 (`/v1/batch`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/batch` | `GetDownloadFile` | 批量下载文件 (zip/tar) |
| DELETE | `/v1/batch` | `DeleteFile` | 批量删除文件 |
| POST | `/v1/batch/task` | `PostOperateFileOrDir` | 创建文件复制/移动任务 |
| DELETE | `/v1/batch/:id/task` | `DeleteOperateFileOrDir` | 取消文件操作任务 |

#### 5.1.7 图片接口 (`/v1/image`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/image` | `GetFileImage` | 获取图片缩略图/原图 |

#### 5.1.8 Samba 接口 (`/v1/samba`)

**连接管理:**

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/samba/connections` | `GetSambaConnectionsList` | 列出所有网络连接 |
| POST | `/v1/samba/connections` | `PostSambaConnectionsCreate` | 创建网络连接 |
| DELETE | `/v1/samba/connections/:id` | `DeleteSambaConnections` | 删除网络连接 |

**共享管理:**

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/samba/shares` | `GetSambaSharesList` | 列出所有共享目录 |
| POST | `/v1/samba/shares` | `PostSambaSharesCreate` | 创建共享目录 |
| DELETE | `/v1/samba/shares/:id` | `DeleteSambaShares` | 删除共享目录 |
| GET | `/v1/samba/shares/status` | `GetSambaStatus` | Samba 服务状态 |

#### 5.1.9 通知接口 (`/v1/notify`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| POST | `/v1/notify/:path` | `PostNotifyMessage` | 发送通知消息 |
| POST | `/v1/notify/system_status` | `PostSystemStatusNotify` | 系统状态通知 |

#### 5.1.10 云存储接口 (`/v1/cloud`, `/v1/driver`)

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/cloud` | `ListStorages` | 列出已挂载的云存储 |
| DELETE | `/v1/cloud` | `UmountStorage` | 卸载云存储 |
| GET | `/v1/driver` | `ListDriverInfo` | 列出可用的云存储驱动 |

#### 5.1.11 其他接口

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| GET | `/v1/other/search` | `GetSearchResult` | 搜索引擎代理 |
| ANY | `/v1/zt/*url` | `ZerotierProxy` | ZeroTier API 代理 |

### 5.2 V2 API 接口 (路径前缀: `/v2/casaos`)

> V2 API 基于 OpenAPI 3.0 规范定义（`api/casaos/openapi.yaml`），使用 `oapi-codegen` 自动生成服务端代码，并集成请求验证中间件。

| 方法 | 路径 | OperationId | 说明 |
|------|------|-------------|------|
| GET | `/v2/casaos/health/services` | `getHealthServices` | 获取各 casaos-* 服务运行状态 |
| GET | `/v2/casaos/health/ports` | `getHealthPorts` | 获取系统 TCP/UDP 端口占用 |
| GET | `/v2/casaos/health/logs` | `getHealthlogs` | 下载日志压缩包 |
| GET | `/v2/casaos/file/test` | `getFileTest` | 文件方法测试端点 |
| GET | `/v2/casaos/file/upload` | `checkUploadChunk` | 检查分块上传状态 |
| POST | `/v2/casaos/file/upload` | `postUploadFile` | 上传文件（multipart/form-data） |
| GET | `/v2/casaos/zt/info` | `getZerotierInfo` | 获取 ZeroTier 节点信息 |
| PUT | `/v2/casaos/zt/:network_id/status` | `setZerotierNetworkStatus` | 设置 ZeroTier 网络状态 |

### 5.3 V3 文件服务接口

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/v3/file?token=xxx&path=xxx` | 基于 token 鉴权的文件直接访问 |

### 5.4 文档接口

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/doc/v2/casaos` | Redoc API 文档页面 |
| GET | `/doc/v2/casaos/openapi.yaml` | OpenAPI YAML 规范文件 |

### 5.5 通用响应格式

V1 API 使用统一的 JSON 响应格式:

```json
{
    "success": 200,
    "message": "ok",
    "data": { ... }
}
```

V2 API 使用 OpenAPI 定义的响应格式:

```json
{
    "message": "",
    "data": { ... }
}
```

---

## 6. 代码框架结构介绍

### 6.1 目录结构

```
CasaOS/
├── main.go                          # 应用入口：初始化、路由注册、HTTP 服务启动
├── go.mod / go.sum                  # Go 模块依赖管理
├── Makefile                         # 构建脚本 (前端 + 后端)
├── .goreleaser.yaml                 # 多架构发布配置 (amd64/arm64/arm-7)
│
├── api/                             # API 定义
│   ├── index.html                   # Redoc API 文档 HTML 模板
│   └── casaos/
│       └── openapi.yaml             # OpenAPI 3.0 规范定义
│
├── build/                           # 构建产物和部署文件
│   ├── scripts/
│   │   ├── setup/                   # 系统安装脚本 (Debian/Arch)
│   │   └── migration/               # 版本迁移脚本
│   └── sysroot/
│       ├── etc/casaos/
│       │   └── casaos.conf.sample   # 配置文件模板
│       └── usr/
│           ├── lib/systemd/system/  # systemd 服务单元文件
│           └── share/casaos/        # Shell 辅助脚本
│
├── cmd/                             # CLI 工具
│   ├── message-bus-docgen/          # MessageBus 文档生成器
│   └── migration-tool/              # 版本迁移工具
│
├── codegen/                         # 自动生成代码 (go generate)
│   ├── casaos_api.go                # OpenAPI -> Go 服务端代码
│   └── message_bus/
│       └── api.go                   # MessageBus 客户端代码
│
├── common/                          # 公共常量和消息定义
│   ├── constants.go                 # 版本号、服务名
│   └── message.go                   # MessageBus 事件类型定义
│
├── conf/                            # 配置文件样本
│   └── conf.conf.sample
│
├── drivers/                         # 云存储驱动实现
│   ├── base/                        # 驱动基础类型和 HTTP 客户端
│   │   ├── client.go
│   │   └── types.go
│   ├── google_drive/                # Google Drive 驱动
│   │   ├── drive.go                 #   驱动实现
│   │   ├── meta.go                  #   元信息定义
│   │   ├── types.go                 #   数据类型
│   │   └── util.go                  #   工具函数
│   ├── onedrive/                    # OneDrive 驱动
│   │   ├── drive.go
│   │   ├── meta.go
│   │   └── util.go
│   └── dropbox/                     # Dropbox 驱动
│       ├── drive.go
│       ├── meta.go
│       ├── types.go
│       └── util.go
│
├── internal/                        # 内部包 (不对外暴露)
│   ├── conf/                        # 内部配置结构
│   │   ├── config.go                #   Config/Database/Scheme/Log 结构体
│   │   ├── var.go                   #   全局变量 (版本、构建信息)
│   │   └── const.go                 #   配置键常量
│   ├── driver/                      # 驱动接口抽象
│   │   ├── driver.go                #   Driver/Meta/Reader/User 接口
│   │   └── item.go                  #   驱动配置项定义
│   ├── op/                          # 操作钩子和驱动注册
│   │   ├── driver.go                #   RegisterDriver / GetDriverNew
│   │   └── hook.go                  #   事件钩子
│   └── sign/                        # HMAC 签名
│       └── sign.go
│
├── interfaces/                      # 接口定义
│   └── migrationTool.go             # 迁移工具接口
│
├── model/                           # 数据传输对象 (DTO)
│   ├── sys_common.go                # 通用模型: Result, Path, APPModel, ServerModel 等
│   ├── file.go                      # 文件操作模型: FileOperate, FileItem, FileUpdate
│   ├── drive.go                     # 云存储模型: Drive
│   ├── search.go                    # 搜索模型: SearchEngine
│   ├── share.go                     # 共享模型: Shares
│   ├── connections.go               # 连接模型: Connections
│   ├── common.go                    # 分页模型: PageReq, PageResp
│   ├── req.go                       # 请求模型
│   ├── obj.go                       # 对象接口: Obj, Object, StorageA
│   ├── notify/                      # 通知模型
│   │   ├── file.go                  #   文件通知
│   │   ├── notify.go                #   通知模型
│   │   └── storage.go               #   存储通知
│   ├── system_app/                  # 系统应用模型
│   └── system_model/                # 系统模型
│
├── pkg/                             # 可复用工具包
│   ├── config/                      # 配置管理
│   │   ├── init.go                  #   INI 文件加载和映射
│   │   └── config.go                #   配置路径管理
│   ├── sqlite/                      # SQLite 数据库
│   │   └── db.go                    #   GORM 初始化、AutoMigrate
│   ├── cache/                       # 内存缓存 (go-cache)
│   │   └── init.go
│   ├── utils/                       # 工具函数集
│   │   ├── file/                    #   文件操作工具
│   │   ├── httper/                  #   HTTP 请求工具
│   │   ├── ip_helper/               #   IP 地址工具
│   │   ├── common_err/              #   错误码定义
│   │   ├── encryption/              #   加密工具
│   │   └── port/                    #   端口工具
│   ├── samba/                       # Samba 集成
│   ├── sign/                        # HMAC 签名
│   ├── github/                      # GitHub API 客户端
│   ├── ddns/                        # DDNS 枚举
│   ├── fs/                          # 文件系统工具
│   ├── generic_sync/                # 通用同步工具
│   ├── gredis/                      # Redis 连接池 (备用)
│   └── singleflight/                # 防击穿
│
├── route/                           # 路由层
│   ├── v1.go                        # V1 路由注册 (Echo + JWT)
│   ├── v2.go                        # V2 路由注册 (OpenAPI + 验证)
│   ├── init.go                      # 路由初始化 (网络挂载、基础信息)
│   └── v1/                          # V1 Handler 实现
│   │   ├── system.go                #   系统管理 Handler
│   │   ├── file.go                  #   文件管理 Handler
│   │   ├── samba.go                 #   Samba Handler
│   │   ├── cloud.go                 #   云存储 Handler
│   │   ├── driver.go                #   驱动 Handler
│   │   ├── notify.go                #   通知 Handler
│   │   ├── image.go                 #   图片 Handler
│   │   ├── zerotier.go              #   ZeroTier Handler
│   │   ├── port.go                  #   端口 Handler
│   │   ├── recover.go               #   OAuth 回调 Handler
│   │   └── other.go                 #   搜索 Handler
│   └── v2/                          # V2 Handler 实现
│       ├── health.go                #   健康检查 Handler
│       ├── file.go                  #   文件 Handler
│       └── zt.go                    #   ZeroTier Handler
│
├── service/                         # 业务逻辑层
│   ├── service.go                   # 服务注册中心 (Repository 接口)
│   ├── system.go                    # SystemService 实现
│   ├── notify.go                    # NotifyService 实现
│   ├── shares.go                    # SharesService 实现
│   ├── connections.go               # ConnectionsService + PeerService 实现
│   ├── storage.go                   # StorageService 实现 (rclone)
│   ├── casa.go                      # CasaService 实现 (版本检查)
│   ├── health.go                    # HealthService 实现
│   ├── rely.go                      # RelyService 实现
│   ├── other.go                     # OtherService 实现 (搜索)
│   ├── file.go                      # 文件操作辅助
│   ├── file_upload.go               # 分块上传服务
│   ├── socket.go                    # WebSocket/Peer 辅助
│   └── model/                       # 数据库模型 (GORM)
│       ├── o_notify.go              #   AppNotify 表
│       ├── o_shares.go              #   SharesDBModel 表
│       ├── o_connections.go         #   ConnectionsDBModel 表
│       ├── o_drive.go               #   PeerDriveDBModel 表
│       └── o_rely.go                #   RelyDBModel 表
│
└── types/                           # 类型定义和枚举
```

### 6.2 分层架构

CasaOS 后端采用经典的三层架构:

```
┌────────────────────────────────────────────────────────┐
│                   Route 层 (路由/控制器)                 │
│  route/v1/*.go, route/v2/*.go                          │
│  职责: 请求解析、参数校验、调用 Service、构造响应          │
└───────────────────────────┬────────────────────────────┘
                            │ 调用
                            ▼
┌────────────────────────────────────────────────────────┐
│                   Service 层 (业务逻辑)                  │
│  service/*.go                                           │
│  职责: 业务逻辑处理、数据库操作、外部服务调用              │
└───────────────────────────┬────────────────────────────┘
                            │ 使用
                            ▼
┌────────────────────────────────────────────────────────┐
│                   Data 层 (数据访问)                     │
│  service/model/*.go, pkg/sqlite/                        │
│  职责: 数据库模型定义、ORM 操作、数据持久化               │
└────────────────────────────────────────────────────────┘
```

### 6.3 服务注册中心

所有 Service 通过 `Repository` 接口统一管理，在 `main.go` 的 `init()` 阶段一次性初始化:

```go
type Repository interface {
    Casa() CasaService
    Connections() ConnectionsService
    Gateway() external.ManagementService
    Health() HealthService
    Notify() NotifyServer
    Rely() RelyService
    Shares() SharesService
    System() SystemService
    Storage() StorageService
    MessageBus() *message_bus.ClientWithResponses
    Peer() PeerService
    Other() OtherService
}
```

全局实例 `service.MyService` 在整个应用中共享使用。

### 6.4 启动流程

```
程序启动
  │
  ├─ init() 阶段
  │   ├─ 解析命令行参数 (-c 配置路径, -db 数据库路径, -v 版本)
  │   ├─ config.InitSetup() → 加载/创建 INI 配置文件
  │   ├─ logger.LogInit() → 初始化日志系统
  │   ├─ sqlite.GetDb() → 打开 SQLite 数据库，执行 AutoMigrate
  │   ├─ service.NewService() → 创建所有 Service 实例
  │   ├─ cache.Init() → 初始化内存缓存
  │   ├─ GetCPUThermalZone() → 检测 CPU 温度传感器路径
  │   └─ route.InitFunction() → 初始化网络挂载和基础设备信息
  │
  └─ main() 阶段
      ├─ 创建 V1/V2/V3/Doc 四个 HTTP Handler
      ├─ 组合为 HandlerMultiplexer
      ├─ 启动 Cron 定时任务 (每 5 秒推送硬件状态)
      ├─ 监听 127.0.0.1:随机端口
      ├─ 向 Gateway 注册所有路由路径
      ├─ 向 MessageBus 注册事件类型 (重试 10 次)
      ├─ 处理端口迁移 (v0.3.6 兼容)
      ├─ 写入运行时 URL 文件
      ├─ 执行 start.d 目录下的启动脚本
      ├─ 通知 systemd 服务已就绪
      └─ 启动 HTTP Server 开始服务
```

### 6.5 构建与发布

**本地构建** (`Makefile`):
- `make build-ui`: 编译前端 (CasaOS-UI, yarn)
- `make build-backend`: 编译后端 (CGO 静态链接 + UPX 压缩)
- `make build`: 完整构建

**发布** (`.goreleaser.yaml`):
- 支持架构: linux/amd64, linux/arm64, linux/arm-7
- 编译前: `go generate`（生成 OpenAPI 代码）、`go test`
- 编译后: UPX 压缩（amd64、arm-7）
- Ldflags 注入: commit hash、构建日期、云存储 OAuth 密钥
- 产物: GitHub Releases（草稿 + 预发布）

---

## 7. 后端数据库实现介绍

### 7.1 数据库技术选型

| 项目 | 选型 | 说明 |
|------|------|------|
| 数据库 | SQLite | 轻量嵌入式数据库，适合单机家庭服务器场景 |
| ORM | GORM | Go 语言最流行的 ORM 框架 |
| 驱动 | glebarez/sqlite | 纯 Go SQLite 驱动（CGO-free 可选） |

### 7.2 数据库初始化

数据库初始化在 `pkg/sqlite/db.go` 中实现，采用单例模式:

```go
func GetDb(dbPath string) *gorm.DB {
    if gdb != nil {
        return gdb
    }
    file.IsNotExistMkDir(dbPath)
    db, _ := gorm.Open(sqlite.Open(dbPath+"/casaOS.db"), &gorm.Config{})

    c, _ := db.DB()
    c.SetMaxIdleConns(10)       // 最大空闲连接数
    c.SetMaxOpenConns(1)        // 最大打开连接数 (SQLite 单写者)
    c.SetConnMaxIdleTime(time.Second * 1000)

    // 自动迁移
    db.AutoMigrate(
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

**关键设计点**:
- **单例模式**: `gdb` 全局变量保证数据库连接只初始化一次
- **连接池**: `MaxOpenConns=1` 适配 SQLite 的单写者特性
- **自动迁移**: 使用 GORM 的 `AutoMigrate` 自动创建/更新表结构
- **向下兼容**: 启动时删除旧版本遗留的废弃表

**数据库文件位置**: 默认路径为 `/var/lib/casaos/db/casaOS.db`

### 7.3 数据库表结构

#### 7.3.1 应用通知表 (`o_notify`)

```
表名: o_notify
模型: AppNotify

┌─────────────┬────────┬─────────────────────────┐
│ 字段         │ 类型   │ 说明                     │
├─────────────┼────────┼─────────────────────────┤
│ custom_id   │ string │ 主键，自定义 ID           │
│ id          │ string │ 通知 ID                  │
│ name        │ string │ 通知名称                  │
│ icon        │ string │ 图标                     │
│ state       │ int    │ 状态: 0=变动中 1=未读 2=已读 │
│ type        │ int    │ 通知类型                  │
│ class       │ int    │ 通知分类                  │
│ message     │ string │ 通知内容                  │
│ created_at  │ string │ 创建时间                  │
│ updated_at  │ string │ 更新时间                  │
└─────────────┴────────┴─────────────────────────┘
```

#### 7.3.2 Samba 共享表 (`o_shares`)

```
表名: o_shares
模型: SharesDBModel

┌─────────────┬────────┬───────────────────────────┐
│ 字段         │ 类型   │ 说明                       │
├─────────────┼────────┼───────────────────────────┤
│ id          │ uint   │ 主键，自增 ID               │
│ name        │ string │ 共享名称                    │
│ path        │ string │ 共享目录路径                 │
│ anonymous   │ bool   │ 是否允许匿名访问             │
│ created     │ int64  │ 创建时间 (自动生成)           │
│ updated     │ int64  │ 更新时间 (自动更新)           │
└─────────────┴────────┴───────────────────────────┘
```

#### 7.3.3 网络连接表 (`o_connections`)

```
表名: o_connections
模型: ConnectionsDBModel

┌─────────────┬────────┬───────────────────────────┐
│ 字段         │ 类型   │ 说明                       │
├─────────────┼────────┼───────────────────────────┤
│ id          │ uint   │ 主键，自增 ID               │
│ username    │ string │ 连接用户名                  │
│ password    │ string │ 连接密码                    │
│ host        │ string │ 远程主机地址                 │
│ port        │ string │ 远程端口                    │
│ status      │ string │ 连接状态                    │
│ directories │ string │ 目录列表 (JSON string array) │
│ mount_point │ string │ 本地挂载点父目录              │
│ created     │ int64  │ 创建时间 (自动生成)           │
│ updated     │ int64  │ 更新时间 (自动更新)           │
└─────────────┴────────┴───────────────────────────┘
```

#### 7.3.4 对等设备表 (PeerDriveDBModel)

```
表名: peer_drive_db_models (GORM 默认命名)
模型: PeerDriveDBModel

┌──────────────┬────────┬───────────────────────────┐
│ 字段          │ 类型   │ 说明                       │
├──────────────┼────────┼───────────────────────────┤
│ id           │ string │ 主键，设备唯一标识           │
│ user_agent   │ string │ 用户代理字符串               │
│ display_name │ string │ 显示名称                    │
│ device_name  │ string │ 设备名称                    │
│ model        │ string │ 设备型号                    │
│ ip           │ string │ IP 地址                    │
│ os           │ string │ 操作系统                    │
│ browser      │ string │ 浏览器                     │
│ online       │ bool   │ 是否在线 (不持久化, gorm:"-") │
│ created      │ int64  │ 创建时间 (自动生成)           │
│ updated      │ int64  │ 更新时间 (自动更新)           │
└──────────────┴────────┴───────────────────────────┘
```

#### 7.3.5 依赖关系表 (`o_rely`)

```
表名: o_rely
模型: RelyDBModel
注意: 此表定义了模型但未在 AutoMigrate 中注册

┌──────────────────────┬────────┬───────────────────────┐
│ 字段                  │ 类型   │ 说明                   │
├──────────────────────┼────────┼───────────────────────┤
│ id                   │ uint   │ 主键，自增 ID           │
│ custom_id            │ string │ 自定义 ID              │
│ container_custom_id  │ string │ 容器自定义 ID           │
│ container_id         │ string │ 容器 ID               │
│ type                 │ int    │ 类型 (暂未使用)         │
│ created_at           │ time   │ 创建时间               │
│ updated_at           │ time   │ 更新时间               │
└──────────────────────┴────────┴───────────────────────┘
```

### 7.4 数据库使用模式

CasaOS 的数据库操作直接在 Service 层中通过 GORM 执行，没有独立的 Repository/DAO 层。典型使用模式:

```go
// 查询 - 以 SharesService 为例
func (s *sharesService) GetSharesList() []model.SharesDBModel {
    var shares []model.SharesDBModel
    s.db.Find(&shares)
    return shares
}

// 创建
func (s *sharesService) CreateShare(share model.SharesDBModel) {
    s.db.Create(&share)
}

// 删除
func (s *sharesService) DeleteShare(id uint) {
    s.db.Delete(&model.SharesDBModel{}, id)
}
```

### 7.5 数据存储策略

CasaOS 采用混合存储策略:

| 存储方式 | 用途 |
|----------|------|
| **SQLite** | 结构化数据: 通知、共享配置、网络连接、设备信息 |
| **INI 文件** | 应用配置 (`casaos.conf`) |
| **JSON 文件** | 基础信息 (`baseinfo.conf`)、应用排序 (`app_order.json`)、模块入口 (`entry.json`) |
| **内存缓存** | 运行时数据: CPU 温度传感器路径、版本信息缓存 |
| **文件系统** | 用户数据、日志文件、运行时 URL 文件 |

### 7.6 数据库迁移机制

CasaOS 提供了独立的迁移工具框架:

- **迁移接口** (`interfaces/migrationTool.go`): 定义了 `IsMigrationNeeded`、`PreMigrate`、`Migrate`、`PostMigrate` 标准接口
- **迁移工具** (`cmd/migration-tool/main.go`): 独立的命令行工具，按版本顺序执行迁移
- **Shell 脚本** (`build/scripts/migration/`): 版本级别的迁移脚本，在系统升级时自动执行

---

> **文档版本**: 1.0  
> **对应代码版本**: CasaOS v0.4.15 (commit 63f0148)
