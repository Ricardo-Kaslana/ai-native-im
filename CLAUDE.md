# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AI-Native IM is a sophisticated instant messaging system with deep AI integration. The architecture consists of:

- **Server**: Rust-based backend using Axum framework
- **Client**: Flutter-based mobile/desktop application
- **Protocol**: Protocol Buffers for efficient binary serialization
- **Databases**: PostgreSQL (primary), Redis (cache/sessions), Qdrant (vector storage for AI)

The AI system supports 9 LLM providers with domestic (Chinese) provider priority: SiliconFlow, Zhipu AI, Moonshot (Kimi), Tongyi Qianwen, Wenxin Yiyan, plus OpenAI, Anthropic, Google Gemini.

**Current Version**: 3.0.0 (see CHANGELOG.md for version history)

## Development Commands

### Server (Rust)

```bash
# Build (ensures PROTOC is set correctly)
./server/build.bat              # Windows - sets PROTOC=.\protoc.exe
./server/build.ps1              # PowerShell alternative
cargo build                     # Direct build (requires PROTOC env var)

# Set PROTOC environment variable manually if needed
set PROTOC=.\protoc.exe         # Windows
export PROTOC=./protoc.exe      # Unix/Linux

# Development
cargo run                       # Start server on :8080
cargo test                      # Run all tests
cargo test --package ai         # Test AI package only
cargo test --test integration_tests  # Run specific test

# Code quality
cargo fmt --all                 # Format code
cargo clippy --all-targets      # Lint code
```

**Note**: The build script (`build.rs`) compiles Protocol Buffer definitions. It requires `protoc.exe` in the project root or `PROTOC` environment variable set.

### Client (Flutter)

```bash
# Dependencies
flutter pub get

# Code generation (run after model changes)
dart run build_runner build --delete-conflicting-outputs

# Run application
flutter run                     # Development mode
flutter run --release           # Production build

# Testing
flutter test                    # Unit tests
flutter drive                   # Integration tests
flutter run -d windows          # Run Windows

# Code quality
dart format .                   # Format code
dart analyze                    # Static analysis
```

### Database Migrations

```bash
# Migrations are in server/migrations/
# Run via SQL or sqlx CLI
sqlx migrate run                # Run pending migrations
```

## Architecture

### Server Structure (`server/src/`)

```
server/src/
├── api/                    # REST API endpoints
│   ├── auth.rs            # Authentication routes
│   ├── groups.rs          # Group management
│   ├── ai_agents.rs       # AI agent CRUD
│   ├── message_ws.rs      # WebSocket handler
│   └── ...
├── services/               # Business logic layer
│   ├── auth/              # JWT, password hashing, rate limiting
│   ├── ai/                # AI Agent system (complex - see below)
│   ├── message/           # WebSocket message routing
│   ├── group/             # Group service with invite codes
│   ├── media/             # File upload with multi-provider storage
│   ├── presence/          # User online/offline status
│   └── admin/             # Admin operations
├── models/                 # Database models
├── middleware/             # JWT verification middleware
├── proto.rs                # Generated Protocol Buffer code
└── main.rs                 # Entry point
```

### AI System Architecture (`server/src/services/ai/`)

The AI system is the core differentiator. Key components:

- **`system.rs`**: `AIAgentSystem` - unified initialization, supports 9 LLM providers with domestic priority
- **`agent_service.rs`**: `AIAgentService` - agent CRUD, personality management
- **`llm_client.rs`**: Abstraction over LLM providers (OpenAI-compatible, Anthropic, Google, etc.)
- **`embedding.rs`**: Vector embeddings for RAG (OpenAI/SiliconFlow)
- **`memory/`**: Vector-based long-term memory with Qdrant
- **`proactive/`**: Agent-initiated conversations based on silence/idle detection
- **`emotion/`**: 11-state emotion system affecting responses (hybrid rule-based + LLM)
- **`persona/`**: Personality customization (name, expertise, speech patterns, catchphrases)
- **`streaming/`**: Streaming responses for real-time AI chat
- **`trigger.rs`**: When agents respond (mentions, keywords, contextual, proactive)
- **`typing_simulator.rs`**: Human-like typing delays

LLM Provider Priority (domestic first):
1. SiliconFlow (recommended - cost-effective Chinese provider)
2. Zhipu AI (GLM models)
3. Moonshot (Kimi)
4. Tongyi Qianwen (Alibaba)
5. Wenxin Yiyan (Baidu)
6. OpenAI
7. Anthropic (Claude)
8. Google Gemini
9. Custom/local OpenAI-compatible endpoint

### Communication Protocol

**WebSocket Endpoints:**
- `WS /ws` - Presence WebSocket (user online/offline status)
- `WS /ws?token=<jwt>` - Message WebSocket (real-time messaging, AI interactions)

**Protocol Buffer Definitions** (`protos/chat.proto`):
- `ClientPacket` - Messages from client to server
- `ServerPacket` - Messages from server to client
- `ChatMessage` - Core message structure
- `AIAgentMessage` - AI-specific message with trigger type, confidence
- `PresenceUpdate`, `TypingIndicator`, `MediaUploadProgress`, etc.

### Message Flow

1. Client sends `ClientPacket` via WebSocket
2. `WebSocketHandler` validates and deserializes protobuf
3. `MessageRouter` routes to appropriate handler (user message, AI interaction, etc.)
4. `AIMessageRouter` intercepts messages mentioning AI agents
5. `AgentMessageHandler` processes AI requests (context building, memory retrieval)
6. `LLMClient` calls provider API
7. Response flows back via `AgentResponseWorker` with typing simulation
8. `MessageRouter` delivers to recipients (online users via WebSocket, offline via PostgreSQL)

### Database Schema

**Core Tables:**
- `users` - User accounts with soft delete, online status tracking
- `groups` - Group information
- `group_members` - Membership with role (MEMBER/ADMIN/OWNER)
- `group_invite_codes` - Invitation system with expiry and use limits
- `conversations` - Chat conversations (private/group)
- `messages` - Message history (TEXT/IMAGE/VIDEO/SYSTEM), supports AI sender
- `ai_agents` - Agent configurations with persona JSONB
- `user_ai_profiles` - Per-user AI settings and affinity scores

**AI-Specific Tables:**
- `agent_emotion_history` - Emotion state tracking over time
- `learning_patterns` - Learned user preferences
- `user_feedback` - Positive/negative feedback for improvement
- `summary_cache` - Cached conversation summaries
- `provider_usage` - Token usage tracking per provider

### Storage Architecture

Multi-provider storage abstraction (`services/media/providers/`):
- **RustFS** (default): Local filesystem with deduplication
- **AWS S3**: Production cloud storage
- **Cloudflare R2**: S3-compatible with zero egress fees
- **Tencent COS**: Alibaba Cloud OSS
- **MinIO**: Self-hosted S3-compatible

### Configuration

**Environment Variables** (see `template/.env.example`):

Required for basic operation:
```env
DATABASE_URL=postgresql://user:pass@localhost:5432/ai_im
REDIS_URL=redis://localhost:6379
JWT_SECRET=<32+ character secret>
```

For AI features (configure at least one):
```env
# Domestic providers (priority)
SILICONFLOW_API_KEY=sk-...
ZHIPU_API_KEY=...
MOONSHOT_API_KEY=...
TONGYI_API_KEY=...
WENXIN_API_KEY=...
WENXIN_SECRET_KEY=...

# International providers
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GOOGLE_API_KEY=...

# Vector database (for memory/RAG)
QDRANT_URL=http://localhost:6334
```

**Build Profiles** (`Cargo.toml`):
- `dev`: Optimized for fast compilation (dependencies at opt-level=2)
- `test`: Optimized for test execution
- `release`: Maximum performance
- `ci`: Balanced CI profile

### Client Structure (`client/lib/`)

```
client/lib/
├── core/
│   ├── config/           # Environment configuration (.env.dev/.staging/.prod)
│   ├── di/               # Riverpod providers
│   ├── network/          # API client, WebSocket, protobuf conversion
│   ├── services/         # Token, sync, file upload, QR code
│   └── storage/          # Drift database (SQLite)
├── features/
│   ├── auth/             # Authentication (login, register)
│   ├── chat/             # Chat interface, message composition
│   ├── groups/           # Group management, QR codes, invites
│   ├── conversations/    # Conversation list
│   └── media/            # Image/video capture, compression
├── shared/
│   ├── widgets/          # Reusable UI components
│   ├── validators/       # Form validation
│   └── extensions/       # Dart extensions
└── generated/            # Protocol Buffer Dart code (chat.pb.dart)
```

**State Management**: Riverpod with code generation (`riverpod_generator`)

**Local Storage**: Drift (SQLite) for offline message caching, secure storage for tokens

### Development Workflow

1. **Protocol Buffer Changes**:
   - Edit `protos/chat.proto`
   - Run `./server/build.bat` (regenerates Rust code)
   - Run `dart run build_runner build` (regenerates Dart code in client)

2. **Testing AI Features**:
   - Set `mock_mode=true` in `AIAgentSystemConfig::mock()` for unit tests
   - Property tests in `services/ai/property_tests.rs`
   - Integration tests in `services/ai/integration_tests.rs`

3. **Adding New AI Provider**:
   - Implement `LLMClient` trait in `services/ai/`
   - Add provider to `LLMProvider` enum
   - Add factory method in `LLMClientFactory`
   - Update `AIAgentSystemConfig` with new env vars
   - Add priority logic in `create_llm_client()`

### Key Patterns

- **Service Pattern**: All business logic in `services/` with trait abstractions
- **Repository Pattern**: Database access via SQLx with compile-time query checking
- **Dependency Injection**: Services built with `.with_*()` builders
- **Error Handling**: `AppError` enum with error codes for API responses
- **Soft Deletes**: Use `deleted_at` timestamp instead of DELETE
- **Timestamps**: All tables have `created_at`, `updated_at` with auto-update trigger
- **Async/Await**: Full async with Tokio runtime

### Important Notes

- **PROTOC Environment**: The build script requires `protoc.exe` in the project root or `PROTOC` environment variable pointing to the protoc binary. Use `./server/build.bat` (Windows) to ensure proper setup.
- **API Endpoints**: The server uses direct endpoints without `/api/` prefix (e.g., `/auth/login`, not `/api/v1/auth/login`). The client must match this.
- **Server Port**: Default is 8080 (not 3000). Ensure client connects to `http://localhost:8080`.
- **Dual WebSocket Design**: Presence (`/ws`) and messaging (`/ws?token=<jwt>`) are separate endpoints with different purposes.
- **LLM Provider Priority**: Domestic Chinese providers are tried first for better latency and cost. Configure at least one provider's API key.
- **Database Migrations**: Located in `server/migrations/`. Run with SQLx CLI or direct SQL execution.
