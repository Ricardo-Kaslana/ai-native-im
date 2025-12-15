# Gemini Project Context: AI Native IM Server

This document provides a comprehensive overview of the `ai-native-im` project, specifically focusing on its Rust-based backend server. It is intended to be used as a context file for AI-assisted development.

## 1. Project Overview

The project is an **AI-Native Instant Messaging (IM)** application designed to be lightweight, aesthetically pleasing, and cross-platform. Its core feature is a deeply integrated AI Agent within group chats, designed to behave like a "real group member."

The backend is a monolithic server written in Rust, providing a RESTful API for standard operations and WebSocket endpoints for real-time communication.

### Core Technologies:
- **Language**: Rust (2021 Edition)
- **Web Framework**: Axum (on top of Tokio)
- **Primary Database**: PostgreSQL (managed with `sqlx`)
- **Caching & Pub/Sub**: Redis
- **Vector Database**: Qdrant (for the AI Agent's memory)
- **API Protocols**: REST (JSON) and WebSocket
- **Schema Definition**: Protocol Buffers (`prost`) are used for some data serialization.

### System Architecture:
The server is composed of several key microservices-style modules within the monolith:
- **User Authentication System**: Manages registration, login (JWT-based), sessions, and user profiles.
- **Message System**: Handles real-time messaging via WebSockets, including message persistence and offline synchronization.
- **Group Management System**: Manages group lifecycle, members, and permissions.
- **AI Agent System**: The core differentiator. It uses a Retrieval-Augmented Generation (RAG) model with a Qdrant vector database to maintain conversational memory. It features a customizable persona and intelligent triggering logic.
- **Media Processing System**: Handles uploading, processing (compression, format conversion), and storage of media files (images/videos).

## 2. Building and Running the Project

The development environment relies on Docker to manage external services and a local Rust toolchain to run the main application.

### Environment Setup

1.  **Dependencies**: Ensure the following are installed:
    *   **Rust/Cargo**: Latest stable version.
    *   **Docker & Docker Compose**: To run infrastructure services.
    *   **`sqlx-cli`**: For database migrations. Install with `cargo install sqlx-cli`.

2.  **Environment Variables**: Create a `.env` file inside the `server/` directory. You can use `template/.env.example` as a starting point. The most critical variables are:
    ```env
    DATABASE_URL=postgresql://dev_user:dev_password@localhost:5432/ai_im_dev
    REDIS_URL=redis://localhost:6379
    QDRANT_URL=http://localhost:6333
    JWT_SECRET=a-long-and-secure-secret-key
    ```

### Running in Development

1.  **Start Infrastructure**: Launch the required background services (Postgres, Redis, Qdrant).
    ```bash
    # From the project root
    docker-compose -f docker-compose.dev.yml up -d
    ```

2.  **Run Database Migrations**: Apply the latest database schema.
    ```bash
    # From the project root
    cd server
    sqlx migrate run
    ```

3.  **Run the Server**: Start the Axum web server.
    ```bash
    # From the server/ directory
    cargo run
    ```
    The server will be available at `http://localhost:8080`.

## 3. Testing the Project

The project contains a suite of unit, integration, and property-based tests.

### Running Tests

To execute all tests for the server:
```bash
# From the project root
cd server
cargo test
```

## 4. Development Conventions

### Code Style
- The code follows standard Rust conventions and formatting, enforced by `rustfmt`.
- The application is structured modularly under `server/src/services/` and `server/src/api/`.

### Database
- All database schema changes are managed through `sqlx-cli` migration files located in `server/migrations/`.
- Do not make manual changes to the database schema; always create a new migration file.

### Commits
- Commit messages should be clear and descriptive, explaining the "what" and "why" of the change. (No specific format like Conventional Commits is enforced at a glance).

### API
- RESTful APIs are used for most state-changing operations.
- Real-time communication (e.g., chat messages, presence updates) is handled via WebSockets.
- Authentication is performed via JWT Bearer tokens in the `Authorization` header for REST endpoints and via a `token` query parameter for the WebSocket connection.
