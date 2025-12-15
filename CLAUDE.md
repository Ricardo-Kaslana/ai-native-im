# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an AI-native instant messaging application built with:
- **Backend**: Rust server using Axum framework with async/await
- **AI Integration**: Multi-provider LLM support (OpenAI, Anthropic, Google Gemini, SiliconFlow, Zhipu AI, Moonshot)
- **Real-time**: WebSocket communication for messaging and AI agent interactions
- **Database**: PostgreSQL with SQLx, Redis for caching, Qdrant for vector storage
- **Media**: Multi-cloud storage support (AWS S3, MinIO, Tencent COS, Aliyun OSS, Cloudflare R2)

## Common Development Commands

### Backend (Rust)

The project uses optimized build configurations for faster development:

```bash
cd server

# Quick syntax check (fastest)
make check

# Ultra-fast check with optimizations
make fast-check

# Optimized incremental build for development
make fast-build

# Standard build
make build

# Testing
make test-fast        # Run with cargo-nextest (5-10x faster, ~30-60s)
make test-quick       # Unit tests only (~60-90s)
make test             # Standard cargo test
make test-watch       # Continuous testing in watch mode

# Install required tools
make install-tools    # Install cargo-nextest, cargo-watch

# Clean build artifacts
make clean            # Clean all
make clean-cache      # Clean incremental cache only
```

### Development Workflow

1. **Fast Development Cycle**: Use `make fast-build` for incremental builds during development
2. **Testing**: Use `make test-fast` for quick feedback, `make test-watch` for continuous testing
3. **Environment Setup**: Copy `template/.env.example` to `.env` and configure your API keys

## Architecture

### Core Components

#### AI Agent System (`server/src/services/ai/`)
- **Agent Core**: Core agent functionality with personality and emotion modeling
- **Providers**: Multi-provider LLM management with automatic fallback
- **Memory**: Vector-based memory using Qdrant for semantic search
- **Learning**: Pattern recognition and user feedback integration
- **Tools**: Extensible tool system for agent capabilities
- **Speech**: STT/TTS integration for voice interactions

#### Message System (`server/src/services/message/`)
- **WebSocket Handler**: Real-time message delivery and presence
- **Message Router**: Routes messages to appropriate handlers
- **Media Processing**: Multi-format media upload/download with cloud storage

#### Authentication & Security
- JWT-based authentication with refresh tokens
- Security audit logging
- Password reset flows
- Rate limiting and quota management

### Database Schema

The database uses PostgreSQL with the following key entities:
- **Users**: Authentication, profiles, and security
- **Groups**: Group chats with member management
- **Conversations**: 1-on-1 and group conversations
- **Messages**: Text, media, and AI-generated content
- **AI Agents**: Personality, emotion, and learning data
- **Media Files**: Multi-cloud storage references

### API Design

The API uses Protocol Buffers (`protos/chat.proto`) for:
- RESTful endpoints for CRUD operations
- WebSocket payloads for real-time updates
- AI agent interaction protocols
- Media upload/download contracts

## Configuration

### Environment Variables (`.env`)

Key configuration sections:
- **Database**: PostgreSQL, Redis, Qdrant connections
- **AI Providers**: API keys for multiple LLM providers (fallback support)
- **Storage**: Multi-cloud media storage configuration
- **JWT**: Authentication token settings
- **Quotas**: Rate limiting and usage limits

### Build Optimization

The Rust project is optimized for fast compilation:
- Profile configurations in `Cargo.toml`
- Custom linker settings in `.cargo/config.toml`
- Incremental compilation enabled by default
- Parallel builds using all CPU cores

## Testing

The project uses a multi-tier testing strategy:
1. **Unit Tests**: Fast, isolated component tests
2. **Integration Tests**: Service and API integration
3. **Property Tests**: AI agent behavior validation
4. **AI Agent Tests**: Comprehensive agent functionality

## AI Agent Features

The AI agent system includes:
- **Personality**: Customizable traits (sarcasm, enthusiasm, expertise)
- **Emotion**: Rule-based and LLM-based emotion simulation
- **Learning**: Pattern recognition from user interactions
- **Proactive Messaging**: Initiates conversations based on context
- **Chain-of-Thought**: Structured reasoning for complex responses
- **Multimodal**: Text, image, and voice message processing

## Development Notes

- The Flutter client appears to be in development/deleted state - focus is on the Rust backend
- Use `make fast-build` for development builds to avoid long compilation times
- The project supports multiple AI providers with automatic fallback
- Vector embeddings use 1024 dimensions for SiliconFlow compatibility
- Media files support multiple cloud providers simultaneously for redundancy

## Security Considerations

- Never commit API keys or `.env` files
- Use different secrets for development and production
- Review SQL migrations before applying
- Validate all user inputs, especially for AI agent interactions
- Implement proper rate limiting for AI API calls