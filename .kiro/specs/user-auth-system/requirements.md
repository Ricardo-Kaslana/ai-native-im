# Requirements Document

## Introduction

本文档定义了 AI 原生即时通讯应用的用户认证与管理系统需求。该系统负责用户注册、登录、身份验证、会话管理、用户资料管理和安全控制。系统必须提供安全可靠的身份验证机制，同时保持简洁的用户体验，符合应用的极简设计理念。

## Glossary

- **User Auth System**: 用户认证系统，负责用户身份验证、授权和会话管理的核心服务
- **JWT Token**: JSON Web Token，用于无状态身份验证的加密令牌
- **Access Token**: 访问令牌，短期有效的 JWT，用于 API 请求认证
- **Refresh Token**: 刷新令牌，长期有效的令牌，用于获取新的访问令牌
- **Password Hash**: 密码哈希，使用 bcrypt 或 argon2 加密存储的密码
- **User Profile**: 用户资料，包含用户的个人信息和设置
- **Session**: 会话，用户登录后的活动状态
- **Rate Limiting**: 速率限制，防止暴力破解和滥用的请求频率控制
- **Two-Factor Authentication**: 双因素认证（2FA），额外的安全验证层
- **User ID**: 用户唯一标识符，UUID 格式
- **Username**: 用户名，用于登录和显示的唯一标识
- **Avatar**: 头像，用户的个人图片
- **Bio**: 个人简介，用户的自我描述文本
- **Online Status**: 在线状态，表示用户当前是否在线
- **Last Seen**: 最后在线时间，用户最后活跃的时间戳
- **Account Status**: 账户状态，包括 active（活跃）、suspended（暂停）、deleted（已删除）
- **Email Verification**: 邮箱验证，确认用户邮箱地址的有效性
- **Password Reset**: 密码重置，通过邮箱重置忘记的密码

## Requirements

### Requirement 1

**User Story:** 作为新用户，我希望能够快速注册账户，以便开始使用即时通讯功能。

#### Acceptance Criteria

1. WHEN a user submits registration information THEN the User Auth System SHALL validate that the username is unique and between 3 and 30 characters
2. WHEN validating registration THEN the User Auth System SHALL verify that the password is at least 8 characters and contains at least one uppercase letter, one lowercase letter, and one number
3. WHEN a user registers with valid information THEN the User Auth System SHALL hash the password using bcrypt with cost factor 12
4. WHEN password hashing completes THEN the User Auth System SHALL create a user record in the database with a unique UUID
5. WHEN user creation succeeds THEN the User Auth System SHALL generate an access token and refresh token pair and return them to the client

### Requirement 2

**User Story:** 作为已注册用户，我希望能够安全地登录我的账户，以便访问我的消息和联系人。

#### Acceptance Criteria

1. WHEN a user submits login credentials THEN the User Auth System SHALL retrieve the user record by username from the database
2. WHEN the user record is found THEN the User Auth System SHALL verify the password against the stored hash using bcrypt
3. WHEN password verification succeeds THEN the User Auth System SHALL generate a new access token valid for 15 minutes and a refresh token valid for 7 days
4. WHEN generating tokens THEN the User Auth System SHALL include user ID, username, and token expiration in the JWT payload
5. WHEN login fails due to invalid credentials THEN the User Auth System SHALL return an error without revealing whether the username or password was incorrect

### Requirement 3

**User Story:** 作为用户，我希望我的会话能够自动续期，以便不需要频繁重新登录。

#### Acceptance Criteria

1. WHEN an access token expires THEN the User Auth System SHALL accept a refresh token to generate a new access token
2. WHEN validating a refresh token THEN the User Auth System SHALL verify the token signature and expiration
3. WHEN a refresh token is valid THEN the User Auth System SHALL generate a new access token with 15 minutes validity
4. WHEN a refresh token is expired or invalid THEN the User Auth System SHALL reject the request and require full re-authentication
5. WHEN a refresh token is used THEN the User Auth System SHALL optionally rotate the refresh token for enhanced security

### Requirement 4

**User Story:** 作为用户，我希望能够更新我的个人资料，以便其他用户了解我的信息。

#### Acceptance Criteria

1. WHEN a user updates their profile THEN the User Auth System SHALL allow modification of username, avatar URL, and bio
2. WHEN updating username THEN the User Auth System SHALL validate uniqueness and length constraints
3. WHEN updating avatar THEN the User Auth System SHALL validate that the avatar field contains a valid URL or is empty
4. WHEN updating bio THEN the User Auth System SHALL validate that the bio does not exceed 500 characters
5. WHEN profile update succeeds THEN the User Auth System SHALL return the updated user profile to the client

### Requirement 5

**User Story:** 作为用户，我希望能够修改我的密码，以便保护我的账户安全。

#### Acceptance Criteria

1. WHEN a user requests password change THEN the User Auth System SHALL require the current password for verification
2. WHEN the current password is verified THEN the User Auth System SHALL validate the new password meets complexity requirements
3. WHEN the new password is valid THEN the User Auth System SHALL hash the new password using bcrypt with cost factor 12
4. WHEN password update succeeds THEN the User Auth System SHALL invalidate all existing refresh tokens for that user
5. WHEN password update completes THEN the User Auth System SHALL send a notification to the user's registered email

### Requirement 6

**User Story:** 作为用户，我希望能够重置忘记的密码，以便重新获得账户访问权限。

#### Acceptance Criteria

1. WHEN a user requests password reset THEN the User Auth System SHALL generate a unique reset token valid for 1 hour
2. WHEN generating a reset token THEN the User Auth System SHALL send an email with the reset link to the user's registered email address
3. WHEN a user submits a reset token and new password THEN the User Auth System SHALL validate the token is not expired and has not been used
4. WHEN the reset token is valid THEN the User Auth System SHALL update the password and invalidate the reset token
5. WHEN password reset completes THEN the User Auth System SHALL invalidate all existing refresh tokens for that user

### Requirement 7

**User Story:** 作为系统管理员，我希望系统能够防止暴力破解攻击，以便保护用户账户安全。

#### Acceptance Criteria

1. WHEN a user attempts to login THEN the User Auth System SHALL track failed login attempts per username
2. WHEN failed login attempts exceed 5 within 15 minutes THEN the User Auth System SHALL temporarily lock the account for 15 minutes
3. WHEN an account is locked THEN the User Auth System SHALL reject all login attempts with a descriptive error message
4. WHEN the lockout period expires THEN the User Auth System SHALL automatically unlock the account and reset the failed attempt counter
5. WHEN a successful login occurs THEN the User Auth System SHALL reset the failed attempt counter for that user

### Requirement 8

**User Story:** 作为用户，我希望系统能够显示我的在线状态，以便我的联系人知道我是否可用。

#### Acceptance Criteria

1. WHEN a user establishes a WebSocket connection THEN the User Auth System SHALL set the user's online status to true
2. WHEN a user's WebSocket connection closes THEN the User Auth System SHALL set the user's online status to false and record the last seen timestamp
3. WHEN a user's online status changes THEN the User Auth System SHALL publish the status update to Redis for real-time notification
4. WHEN querying a user's profile THEN the User Auth System SHALL include the online status and last seen timestamp
5. WHEN a user has been inactive for more than 5 minutes THEN the User Auth System SHALL mark the user as away but keep the connection active

### Requirement 9

**User Story:** 作为开发者，我希望能够验证 JWT 令牌的有效性，以便保护 API 端点。

#### Acceptance Criteria

1. WHEN an API request includes an access token THEN the User Auth System SHALL verify the token signature using the secret key
2. WHEN verifying a token THEN the User Auth System SHALL check that the token has not expired
3. WHEN a token is valid THEN the User Auth System SHALL extract the user ID and username from the payload
4. WHEN a token is invalid or expired THEN the User Auth System SHALL reject the request with a 401 Unauthorized status
5. WHEN token verification succeeds THEN the User Auth System SHALL attach the user information to the request context for downstream handlers

### Requirement 10

**User Story:** 作为用户，我希望能够查看其他用户的公开资料，以便了解我的联系人信息。

#### Acceptance Criteria

1. WHEN a user requests another user's profile THEN the User Auth System SHALL return the public profile information including username, avatar, bio, and online status
2. WHEN retrieving a profile THEN the User Auth System SHALL not expose sensitive information such as email, password hash, or refresh tokens
3. WHEN the requested user does not exist THEN the User Auth System SHALL return a 404 Not Found error
4. WHEN retrieving multiple user profiles THEN the User Auth System SHALL support batch retrieval with a maximum of 100 users per request
5. WHEN querying profiles THEN the User Auth System SHALL use database indexes on user ID for optimal performance

### Requirement 11

**User Story:** 作为用户，我希望能够搜索其他用户，以便添加新的联系人。

#### Acceptance Criteria

1. WHEN a user searches by username THEN the User Auth System SHALL return matching users using case-insensitive partial matching
2. WHEN performing a search THEN the User Auth System SHALL limit results to 20 users per query
3. WHEN search results are returned THEN the User Auth System SHALL include username, avatar, and online status for each user
4. WHEN a search query is less than 2 characters THEN the User Auth System SHALL reject the request with a validation error
5. WHEN executing searches THEN the User Auth System SHALL use database indexes on username for efficient querying

### Requirement 12

**User Story:** 作为用户，我希望能够注销登录，以便在共享设备上保护我的隐私。

#### Acceptance Criteria

1. WHEN a user requests logout THEN the User Auth System SHALL invalidate the current refresh token
2. WHEN invalidating a refresh token THEN the User Auth System SHALL add it to a blacklist with expiration matching the token's original expiry
3. WHEN a blacklisted token is used THEN the User Auth System SHALL reject the request with an error
4. WHEN logout completes THEN the User Auth System SHALL return a success response to the client
5. WHEN a user logs out THEN the User Auth System SHALL close all active WebSocket connections for that user

### Requirement 13

**User Story:** 作为系统管理员，我希望能够管理用户账户状态，以便处理违规行为和账户问题。

#### Acceptance Criteria

1. WHEN an administrator suspends a user account THEN the User Auth System SHALL set the account status to suspended
2. WHEN a suspended user attempts to login THEN the User Auth System SHALL reject the login with a descriptive error message
3. WHEN an administrator reactivates a suspended account THEN the User Auth System SHALL set the account status to active
4. WHEN an administrator deletes a user account THEN the User Auth System SHALL set the account status to deleted and anonymize personal data
5. WHEN querying user information THEN the User Auth System SHALL filter out deleted accounts from search results

### Requirement 14

**User Story:** 作为开发者，我希望系统能够记录认证相关的安全事件，以便审计和问题排查。

#### Acceptance Criteria

1. WHEN a user logs in successfully THEN the User Auth System SHALL log the event with user ID, timestamp, and IP address
2. WHEN a login attempt fails THEN the User Auth System SHALL log the event with username, timestamp, IP address, and failure reason
3. WHEN a password is changed THEN the User Auth System SHALL log the event with user ID and timestamp
4. WHEN an account is locked due to failed attempts THEN the User Auth System SHALL log the event with user ID and lockout duration
5. WHEN security events are logged THEN the User Auth System SHALL store them in a separate audit log table for compliance and analysis
