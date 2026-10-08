import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';
import '../config/app_config.dart';
import '../error/api_error.dart';
import '../storage/storage.dart';

/// API 客户端
class ApiClient {
  static ApiClient? _instance;
  static ApiClient get instance => _instance ??= ApiClient._();

  late final Dio _dio;

  /// 获取底层 Dio 实例（用于需要自定义响应处理的场景）
  Dio get dio => _dio;
  final SecureStorage _storage = SecureStorage.instance;

  /// 登出回调
  void Function()? onLogout;

  // Token 刷新锁（防止并发刷新）
  bool _isRefreshing = false;
  final List<Function()> _refreshQueue = [];

  ApiClient._() {
    _dio = Dio(BaseOptions(
      baseUrl: AppConfig.apiBaseUrl,
      connectTimeout: Duration(seconds: AppConfig.connectTimeout),
      receiveTimeout: Duration(seconds: AppConfig.receiveTimeout),
      headers: {'Content-Type': 'application/json'},
    ));

    _dio.interceptors.add(InterceptorsWrapper(
      onRequest: (options, handler) {
        debugPrint('API: ${options.method} ${options.uri}');
        if (options.data != null) {
          debugPrint('API: Request data: ${options.data}');
        }
        if (options.queryParameters.isNotEmpty) {
          debugPrint('API: Query params: ${options.queryParameters}');
        }
        _onRequest(options, handler);
      },
      onResponse: (response, handler) {
        debugPrint('API: Response ${response.statusCode} for ${response.requestOptions.uri}');
        if (kDebugMode) {
          debugPrint('API: Response data: ${response.data}');
        }
        _onResponse(response, handler);
      },
      onError: (error, handler) {
        debugPrint('API: Error ${error.type} for ${error.requestOptions.uri}');
        debugPrint('API: Error response: ${error.response?.statusCode} - ${error.response?.data}');
        _onError(error, handler);
      },
    ));
  }

  Future<void> _onRequest(RequestOptions options, RequestInterceptorHandler handler) async {
    final token = await _storage.getAccessToken();
    if (token != null) {
      options.headers['Authorization'] = 'Bearer $token';
    }
    handler.next(options);
  }

  void _onResponse(Response response, ResponseInterceptorHandler handler) {
    handler.next(response);
  }

  Future<void> _onError(DioException error, ErrorInterceptorHandler handler) async {
    if (error.response?.statusCode == 401) {
      // 尝试刷新 Token
      final refreshed = await _tryRefreshToken();
      if (refreshed) {
        // 重试原请求
        try {
          final response = await _retry(error.requestOptions);
          handler.resolve(response);
          return;
        } catch (_) {}
      }
      // 刷新失败，触发登出
      onLogout?.call();
    }
    handler.next(error);
  }

  Future<bool> _tryRefreshToken() async {
    // 如果正在刷新，等待完成
    if (_isRefreshing) {
      return false;
    }

    _isRefreshing = true;
    try {
      final refreshToken = await _storage.getRefreshToken();
      if (refreshToken == null) return false;

      final response = await Dio().post(
        '${AppConfig.apiBaseUrl}/auth/refresh',
        data: {'refresh_token': refreshToken},
        options: Options(
          validateStatus: (status) => status != null && status < 500,
        ),
      );

      final data = response.data;
      if (data != null && data['access_token'] != null) {
        await _storage.saveAccessToken(data['access_token']);
        if (data['refresh_token'] != null) {
          await _storage.saveRefreshToken(data['refresh_token']);
        }
        debugPrint('ApiClient: Token refreshed successfully');
        return true;
      }
      debugPrint('ApiClient: Token refresh failed - no access token in response');
      return false;
    } catch (e) {
      debugPrint('ApiClient: Token refresh error - $e');
      return false;
    } finally {
      _isRefreshing = false;
      // 执行队列中的等待请求
      for (final callback in _refreshQueue) {
        try {
          callback();
        } catch (_) {}
      }
      _refreshQueue.clear();
    }
  }

  Future<Response> _retry(RequestOptions requestOptions) async {
    final token = await _storage.getAccessToken();
    final options = Options(
      method: requestOptions.method,
      headers: {...requestOptions.headers, 'Authorization': 'Bearer $token'},
    );
    return _dio.request(
      requestOptions.path,
      data: requestOptions.data,
      queryParameters: requestOptions.queryParameters,
      options: options,
    );
  }

  /// GET 请求
  Future<T> get<T>(
    String path, {
    Map<String, dynamic>? queryParameters,
    required T Function(dynamic) fromJson,
  }) async {
    try {
      final response = await _dio.get(path, queryParameters: queryParameters);
      return _handleResponse(response, fromJson);
    } on DioException catch (e) {
      throw _handleDioError(e);
    }
  }

  /// POST 请求
  Future<T> post<T>(
    String path, {
    dynamic data,
    required T Function(dynamic) fromJson,
  }) async {
    try {
      final response = await _dio.post(path, data: data);
      return _handleResponse(response, fromJson);
    } on DioException catch (e) {
      throw _handleDioError(e);
    }
  }

  /// POST 请求（无返回值）
  Future<void> postEmpty(String path, {dynamic data}) async {
    try {
      await _dio.post(path, data: data);
    } on DioException catch (e) {
      throw _handleDioError(e);
    }
  }

  /// PUT 请求
  Future<T> put<T>(
    String path, {
    dynamic data,
    required T Function(dynamic) fromJson,
  }) async {
    try {
      final response = await _dio.put(path, data: data);
      return _handleResponse(response, fromJson);
    } on DioException catch (e) {
      throw _handleDioError(e);
    }
  }

  /// PUT 请求（无返回值）
  Future<void> putEmpty(String path, {dynamic data}) async {
    try {
      await _dio.put(path, data: data);
    } on DioException catch (e) {
      throw _handleDioError(e);
    }
  }

  /// DELETE 请求
  Future<T> delete<T>(
    String path, {
    required T Function(dynamic) fromJson,
  }) async {
    try {
      final response = await _dio.delete(path);
      return _handleResponse(response, fromJson);
    } on DioException catch (e) {
      throw _handleDioError(e);
    }
  }

  /// DELETE 请求（无返回值）
  Future<void> deleteEmpty(String path) async {
    try {
      await _dio.delete(path);
    } on DioException catch (e) {
      throw _handleDioError(e);
    }
  }

  /// 上传文件
  Future<T> upload<T>(
    String path, {
    required FormData formData,
    required T Function(dynamic) fromJson,
    void Function(int, int)? onSendProgress,
  }) async {
    try {
      final response = await _dio.post(
        path,
        data: formData,
        onSendProgress: onSendProgress,
        options: Options(contentType: 'multipart/form-data'),
      );
      return _handleResponse(response, fromJson);
    } on DioException catch (e) {
      throw _handleDioError(e);
    }
  }

  T _handleResponse<T>(Response response, T Function(dynamic) fromJson) {
    final data = response.data;
    
    // 检查是否有 success 字段（后端统一响应格式）
    if (data is Map && data.containsKey('success')) {
      if (data['success'] == true) {
        return fromJson(data['data']);
      } else {
        final error = data['error'];
        throw ApiException(
          errorCode: ApiErrorCode.fromCode(error?['code']),
          message: error?['message'],
          statusCode: response.statusCode,
        );
      }
    }
    
    // 直接返回数据
    return fromJson(data);
  }

  ApiException _handleDioError(DioException error) {
    switch (error.type) {
      case DioExceptionType.connectionTimeout:
      case DioExceptionType.sendTimeout:
      case DioExceptionType.receiveTimeout:
        return ApiException.fromTimeout();
      case DioExceptionType.connectionError:
        return ApiException.fromNetworkError(error);
      case DioExceptionType.badResponse:
        final response = error.response;
        if (response != null) {
          final data = response.data;
          if (data is Map && data['error'] != null) {
            final errorData = data['error'];
            String? errorCodeStr;
            String? messageStr;

            // 安全提取错误码和消息
            if (errorData is Map) {
              errorCodeStr = _safeToString(errorData['code']);
              messageStr = _safeToString(errorData['message']);
            } else {
              // error 可能直接是字符串消息
              messageStr = _safeToString(errorData);
            }

            return ApiException(
              errorCode: ApiErrorCode.fromCode(errorCodeStr),
              message: messageStr,
              statusCode: response.statusCode,
            );
          }
          return ApiException(
            errorCode: ApiErrorCode.fromHttpStatus(response.statusCode ?? 500),
            statusCode: response.statusCode,
          );
        }
        return ApiException(errorCode: ApiErrorCode.unknown);
      default:
        return ApiException(
          errorCode: ApiErrorCode.unknown,
          originalError: error,
        );
    }
  }

  /// 安全转换为 String
  String? _safeToString(dynamic value) {
    if (value == null) return null;
    if (value is String) return value;
    if (value is int || value is double || value is num) return value.toString();
    return value.toString();
  }
}
