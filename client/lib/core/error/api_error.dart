/// API 错误码枚举
enum ApiErrorCode {
  // 认证相关
  unauthorized('UNAUTHORIZED', '未登录或登录已过期'),
  invalidCredentials('INVALID_CREDENTIALS', '用户名或密码错误'),
  tokenExpired('TOKEN_EXPIRED', 'Token已过期'),
  tokenInvalid('TOKEN_INVALID', 'Token无效'),
  
  // 用户相关
  userNotFound('USER_NOT_FOUND', '用户不存在'),
  userAlreadyExists('USER_ALREADY_EXISTS', '用户已存在'),
  emailAlreadyExists('EMAIL_ALREADY_EXISTS', '邮箱已被使用'),
  usernameAlreadyExists('USERNAME_ALREADY_EXISTS', '用户名已被使用'),
  
  // 资源相关
  notFound('NOT_FOUND', '资源不存在'),
  forbidden('FORBIDDEN', '无权访问'),
  
  // 群组相关
  groupNotFound('GROUP_NOT_FOUND', '群组不存在'),
  notGroupMember('NOT_GROUP_MEMBER', '不是群组成员'),
  notGroupOwner('NOT_GROUP_OWNER', '不是群主'),
  
  // 对话相关
  conversationNotFound('CONVERSATION_NOT_FOUND', '对话不存在'),
  
  // 联系人相关
  contactNotFound('CONTACT_NOT_FOUND', '联系人不存在'),
  contactAlreadyExists('CONTACT_ALREADY_EXISTS', '已是好友'),
  
  // 网络相关
  networkError('NETWORK_ERROR', '网络连接失败'),
  serverError('SERVER_ERROR', '服务器错误'),
  timeout('TIMEOUT', '请求超时'),
  
  // 通用
  badRequest('BAD_REQUEST', '请求参数错误'),
  unknown('UNKNOWN', '未知错误');

  const ApiErrorCode(this.code, this.defaultMessage);
  
  final String code;
  final String defaultMessage;

  /// 从服务器错误码解析
  static ApiErrorCode fromCode(String? code) {
    if (code == null) return unknown;
    return ApiErrorCode.values.firstWhere(
      (e) => e.code == code,
      orElse: () => unknown,
    );
  }

  /// 从 HTTP 状态码解析
  static ApiErrorCode fromHttpStatus(int status) {
    switch (status) {
      case 400:
        return badRequest;
      case 401:
        return unauthorized;
      case 403:
        return forbidden;
      case 404:
        return notFound;
      case 408:
        return timeout;
      case >= 500:
        return serverError;
      default:
        return unknown;
    }
  }
}

/// API 异常
class ApiException implements Exception {
  final ApiErrorCode errorCode;
  final String? message;
  final int? statusCode;
  final dynamic originalError;

  ApiException({
    required this.errorCode,
    this.message,
    this.statusCode,
    this.originalError,
  });

  /// 显示消息
  String get displayMessage => message ?? errorCode.defaultMessage;

  /// 是否需要重新认证
  bool get requiresReauth =>
      errorCode == ApiErrorCode.unauthorized ||
      errorCode == ApiErrorCode.tokenExpired ||
      errorCode == ApiErrorCode.tokenInvalid;

  @override
  String toString() => 'ApiException(${errorCode.code}): $displayMessage';

  /// 从网络错误创建
  factory ApiException.fromNetworkError(dynamic error) {
    return ApiException(
      errorCode: ApiErrorCode.networkError,
      message: '网络连接失败，请检查网络设置',
      originalError: error,
    );
  }

  /// 从超时错误创建
  factory ApiException.fromTimeout() {
    return ApiException(
      errorCode: ApiErrorCode.timeout,
      message: '请求超时，请稍后重试',
    );
  }
}
