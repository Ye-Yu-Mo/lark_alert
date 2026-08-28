# Changelog

变更记录

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)

版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)

---

## 版本号说明

- **主版本号（Major）**：不兼容的 API 变更或架构重构
- **次版本号（Minor）**：向后兼容的功能新增（新模块、新页面、新接口）
- **修订号（Patch）**：向后兼容的问题修正、小优化、文档更新

---

## [0.2.0] - 2026-08-28

### Changed

- **强制告警上下文**：
  - `Card` 构造必须提供 `service`、`node`、`timestamp`、`content`
  - 新增 `Card::node()`、`Card::content()` / `message()` 方法
  - `Card::to_json()` 和 `LarkAlert::send_card()` 会校验必填字段，拒绝空值
  - 卡片布局增加「报警内容」和「报警节点」字段
- Python `Card()` 构造必须传入 `service`、`node`、`timestamp`、`content`

---

## [0.1.0] - 2026-08-28

### Added

#### M1: Rust 核心库与 Python 绑定

- 新增飞书自定义机器人 Webhook 客户端：
  - 支持 `text` 消息
  - 支持 `post` 富文本消息
  - 支持 `interactive card` 卡片消息
- 新增统一卡片样式：
  - 不使用 emoji
  - 仅用 header 颜色区分级别
  - `info=blue`、`success=green`、`warning=orange`、`error=red`、`critical=carmine`
  - 默认卡片元素：标题、摘要、双列字段、自定义元素、详情、底部 note
- 新增可选签名校验：
  - 配置 `secret` 后自动在请求体中附加 `timestamp` 和 `sign`
  - 签名算法与飞书自定义机器人官方文档一致
- 新增生产级网络行为：
  - 默认超时
  - 可配置重试次数
  - 指数退避
  - 连接池复用
- 新增错误类型：
  - 非法 Webhook URL
  - HTTP 请求失败
  - 飞书非 2xx 状态
  - 飞书业务错误码
  - 响应解析失败
  - 重试耗尽
  - 消息校验失败
- 新增 PyO3/maturin Python 绑定：
  - `LarkAlert`
  - `Card`
  - `Severity`
  - `TextMessage`
  - `PostMessage`
- 新增测试：
  - Rust 单元测试
  - Rust mock HTTP 集成测试
  - Python 绑定测试
- 新增文档：
  - README
  - 消息模型与统一样式文档
  - Python 示例

### Changed

- 无（首次发布）

### Fixed

- 无（首次发布）

### Security

- Webhook 签名校验可选开启；未配置 `secret` 时保持与飞书默认行为一致
