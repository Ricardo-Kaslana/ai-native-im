/// 应用配置
class AppConfig {
  AppConfig._();

  /// API 基础地址
  /// 优先级：环境变量 > .env 文件 > 默认值
  static String get apiBaseUrl {
    // 1. 检查编译时环境变量
    const envUrl = String.fromEnvironment('API_BASE_URL');
    if (envUrl.isNotEmpty) return envUrl;

    // 2. 检查运行时配置（可以从 .env 或远程配置加载）
    final runtimeUrl = _apiBaseUrlOverride;
    if (runtimeUrl != null) return runtimeUrl;

    // 3. 使用默认值
    return 'http://127.0.0.1:8080';
  }

  /// 运行时 API 地址覆盖（用于开发调试）
  static String? _apiBaseUrlOverride;

  /// 设置运行时 API 地址
  static void setApiBaseUrl(String url) {
    _apiBaseUrlOverride = url;
  }

  /// 重置为默认 API 地址
  static void resetApiBaseUrl() {
    _apiBaseUrlOverride = null;
  }

  /// WebSocket 基础地址
  static String get wsBaseUrl {
    // 1. 检查编译时环境变量
    const envWsUrl = String.fromEnvironment('WS_BASE_URL');
    if (envWsUrl.isNotEmpty) return envWsUrl;

    // 2. 检查运行时配置
    final runtimeWsUrl = _wsBaseUrlOverride;
    if (runtimeWsUrl != null) return runtimeWsUrl;

    // 3. 从 API URL 转换
    final api = apiBaseUrl;
    if (api.startsWith('https://')) {
      return api.replaceFirst('https://', 'wss://');
    } else if (api.startsWith('http://')) {
      return api.replaceFirst('http://', 'ws://');
    }
    return api;
  }

  /// 运行时 WebSocket 地址覆盖
  static String? _wsBaseUrlOverride;

  /// 设置运行时 WebSocket 地址
  static void setWsBaseUrl(String url) {
    _wsBaseUrlOverride = url;
  }

  /// 重置为默认 WebSocket 地址
  static void resetWsBaseUrl() {
    _wsBaseUrlOverride = null;
  }

  /// 应用名称
  static const String appName = 'AIIM';

  /// 应用版本
  static const String version = '1.0.0';

  /// 连接超时时间（秒）
  static const int connectTimeout = 30;

  /// 接收超时时间（秒）
  static const int receiveTimeout = 30;

  /// WebSocket 心跳间隔（秒）
  static const int wsHeartbeatInterval = 30;

  /// WebSocket 重连延迟（秒）
  static const int wsReconnectDelay = 5;

  /// 消息分页大小
  static const int messagePageSize = 50;

  /// 对话分页大小
  static const int conversationPageSize = 20;

  /// 是否为开发模式
  static bool get isDebug {
    const isDebug = bool.fromEnvironment('DEBUG', defaultValue: true);
    return isDebug;
  }

  /// 当前环境
  static AppEnvironment get environment {
    const env = String.fromEnvironment('ENVIRONMENT', defaultValue: 'production');
    return AppEnvironment.values.firstWhere(
      (e) => e.name == env,
      orElse: () => AppEnvironment.production,
    );
  }
}

/// 应用环境
enum AppEnvironment {
  development,
  staging,
  production;

  bool get isDevelopment => this == development;
  bool get isStaging => this == staging;
  bool get isProduction => this == production;
}
