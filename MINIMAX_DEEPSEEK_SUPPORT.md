# MiniMax / DeepSeek 支持配置指南

本文档说明如何在 Claw Code 中启用 MiniMax 和 DeepSeek 大模型对接。

> 适用版本：`qixingouo/clawcode2`（已包含 MiniMax/DeepSeek 支持）

---

## 一、环境要求

- **Rust 1.70+**（若从源码编译）
- **Git**（用于克隆代码）
- **API Key**（MiniMax 或 DeepSeek 平台获取）

---

## 二、快速安装

### 方式一：从源码编译

```bash
# 1. 克隆仓库
git clone https://github.com/qixingouo/clawcode2.git
cd clawcode2/rust

# 2. 编译（Debug 模式，2-5 分钟）
cargo build --workspace

# 3. 验证
./target/debug/claw --version
```

### 方式二：使用 Rust 安装工具链

```bash
# 安装 Rust（如果尚未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# 克隆并编译
git clone https://github.com/qixingouo/clawcode2.git
cd clawcode2/rust
cargo build --workspace
```

---

## 三、配置 API Key

### MiniMax

```bash
export MINIMAX_API_KEY="your-minimax-api-key"
export MINIMAX_BASE_URL="https://api.minimax.chat/v1"  # 可选，使用默认
```

### DeepSeek

```bash
export DEEPSEEK_API_KEY="your-deepseek-api-key"
export DEEPSEEK_BASE_URL="https://api.deepseek.com/v1"  # 可选，使用默认
```

> **注意：** `.env` 文件同样支持，在项目根目录创建 `.env` 文件写入上述变量即可。

---

## 四、使用方式

### 使用 MiniMax 模型

```bash
# 非流式输出
./target/debug/claw prompt "你好，请介绍一下自己" --model minimax/abab6.5s

# 流式输出（实时显示）
./target/debug/claw prompt "用 Rust 写一个快速排序" --model minimax/abab6.5s
```

### 使用 DeepSeek 模型

```bash
# 非流式输出
./target/debug/claw prompt "解释什么是尾递归优化" --model deepseek/deepseek-chat

# 指定输出 token 上限
./target/debug/claw prompt "写一个 HTTP 服务器" --model deepseek/deepseek-coder --max-tokens 4096
```

### 模型别名说明

| 前缀 | 说明 | 示例模型 |
|------|------|---------|
| `minimax/` | MiniMax API 路由 | `minimax/abab6.5s`, `minimax/moonshot` |
| `deepseek/` | DeepSeek API 路由 | `deepseek/deepseek-chat`, `deepseek/deepseek-coder` |

---

## 五、会话模式（REPL）

```bash
# 启动交互式会话
./target/debug/claw

# 在 REPL 中切换模型
/set model minimax/abab6.5s
```

---

## 六、信任域配置（安全策略）

Claw Code 默认使用 `danger-full-access` 权限模式，生产环境建议使用受限模式：

```bash
# 只读模式（安全）
./target/debug/claw --permission-mode read-only prompt "..."

# 请求确认模式
./target/debug/claw --permission-mode confirm prompt "..."
```

---

## 七、验证配置是否生效

```bash
# 1. 确认 API Key 已设置
echo $MINIMAX_API_KEY    # 或 $DEEPSEEK_API_KEY

# 2. 使用 doctor 命令检查状态
./target/debug/claw doctor

# 3. 测试 API 连通性
curl -X POST https://api.minimax.chat/v1/chat/completions \
  -H "Authorization: Bearer $MINIMAX_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"abab6.5s","messages":[{"role":"user","content":"hi"}],"max_tokens":10}'
```

---

## 八、常见问题

### Q: 提示 `MINIMAX_API_KEY is not set`

**原因：** 环境变量未设置或未生效。

**解决：**
```bash
# 确认变量已导出
export MINIMAX_API_KEY="your-key"
# 重新执行命令
```

### Q: 编译报 `link error` 或 `rustc not found`

**原因：** Rust 环境未安装或未正确加载。

**解决：**
```bash
# 加载 Rust 环境
. "$HOME/.cargo/env"
# 验证
rustc --version
```

### Q: API 返回 401 Unauthorized

**原因：** API Key 错误或已过期。

**解决：** 前往 MiniMax/DeepSeek 控制台重新获取 API Key。

### Q: 模型响应格式异常或 tool_calls 不工作

**原因：** 部分国内厂商 API 的 tool_calls 格式与标准 OpenAI 存在差异。

**解决：** 关注 [qixingouo/clawcode2](https://github.com/qixingouo/clawcode2) 的更新，或提交 Issue 反馈问题。

---

## 九、相关文件

| 文件路径 | 说明 |
|---------|------|
| `rust/crates/api/src/providers/mod.rs` | Provider 路由逻辑 |
| `rust/crates/api/src/providers/openai_compat.rs` | OpenAI 兼容客户端（含新增的 MiniMax/DeepSeek） |
| `rust/crates/api/src/types.rs` | 请求/响应类型定义 |

---

## 十、参考链接

- Claw Code 仓库：https://github.com/qixingouo/clawcode2
- MiniMax API 文档：https://www.minimaxi.com/document
- DeepSeek API 文档：https://platform.deepseek.com
- Rust 安装：https://rustup.rs
