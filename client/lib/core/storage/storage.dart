import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// 安全存储 - 用于敏感数据（Token等）
class SecureStorage {
  SecureStorage._();
  static final SecureStorage instance = SecureStorage._();

  final _storage = const FlutterSecureStorage(
    aOptions: AndroidOptions(encryptedSharedPreferences: true),
    iOptions: IOSOptions(accessibility: KeychainAccessibility.first_unlock),
  );

  // Token 相关
  static const _keyAccessToken = 'access_token';
  static const _keyRefreshToken = 'refresh_token';
  static const _keyUserId = 'user_id';

  // 记住我相关
  static const _keyRememberMe = 'remember_me';
  static const _keySavedUsername = 'saved_username';
  static const _keySavedPassword = 'saved_password';

  // 内存缓存（用于同步访问）
  String? _cachedUserId;
  String? _cachedAccessToken;

  Future<void> saveAccessToken(String token) async {
    _cachedAccessToken = token;
    await _storage.write(key: _keyAccessToken, value: token);
  }

  Future<String?> getAccessToken() async {
    _cachedAccessToken ??= await _storage.read(key: _keyAccessToken);
    return _cachedAccessToken;
  }

  /// 同步获取访问令牌（从缓存）
  String? getAccessTokenSync() {
    return _cachedAccessToken;
  }

  Future<void> saveRefreshToken(String token) async {
    await _storage.write(key: _keyRefreshToken, value: token);
  }

  Future<String?> getRefreshToken() async {
    return await _storage.read(key: _keyRefreshToken);
  }

  Future<void> saveUserId(String userId) async {
    _cachedUserId = userId;
    await _storage.write(key: _keyUserId, value: userId);
  }

  Future<String?> getUserId() async {
    _cachedUserId ??= await _storage.read(key: _keyUserId);
    return _cachedUserId;
  }

  /// 同步获取用户ID（从缓存）
  String? getUserIdSync() {
    return _cachedUserId;
  }

  /// 初始化缓存（应用启动时调用）
  Future<void> initCache() async {
    _cachedUserId = await _storage.read(key: _keyUserId);
    _cachedAccessToken = await _storage.read(key: _keyAccessToken);
  }

  Future<void> clearAuth() async {
    _cachedAccessToken = null;
    _cachedUserId = null;
    await _storage.delete(key: _keyAccessToken);
    await _storage.delete(key: _keyRefreshToken);
    await _storage.delete(key: _keyUserId);
    // 注意：不清除记住我的凭据，因为用户可能下次还想自动填充
  }

  Future<void> clearAll() async {
    _cachedAccessToken = null;
    _cachedUserId = null;
    await _storage.deleteAll();
  }

  // ========== 记住我功能 ==========

  /// 保存登录凭据
  Future<void> saveCredentials({
    required String username,
    required String password,
  }) async {
    await _storage.write(key: _keyRememberMe, value: 'true');
    await _storage.write(key: _keySavedUsername, value: username);
    await _storage.write(key: _keySavedPassword, value: password);
  }

  /// 获取保存的用户名
  Future<String?> getSavedUsername() async {
    final rememberMe = await _storage.read(key: _keyRememberMe);
    if (rememberMe != 'true') return null;
    return await _storage.read(key: _keySavedUsername);
  }

  /// 获取保存的密码
  Future<String?> getSavedPassword() async {
    final rememberMe = await _storage.read(key: _keyRememberMe);
    if (rememberMe != 'true') return null;
    return await _storage.read(key: _keySavedPassword);
  }

  /// 清除保存的凭据（登出时调用）
  Future<void> clearCredentials() async {
    await _storage.delete(key: _keyRememberMe);
    await _storage.delete(key: _keySavedUsername);
    await _storage.delete(key: _keySavedPassword);
  }
}

/// 普通存储 - 用于非敏感数据（设置等）
class AppStorage {
  AppStorage._();
  static AppStorage? _instance;
  static AppStorage get instance => _instance ??= AppStorage._();

  SharedPreferences? _prefs;

  Future<void> init() async {
    _prefs = await SharedPreferences.getInstance();
  }

  SharedPreferences get _preferences {
    if (_prefs == null) {
      throw StateError('AppStorage not initialized. Call init() first.');
    }
    return _prefs!;
  }

  // 主题设置
  static const _keyThemeMode = 'theme_mode';

  Future<void> setThemeMode(String mode) async {
    await _preferences.setString(_keyThemeMode, mode);
  }

  String getThemeMode() {
    return _preferences.getString(_keyThemeMode) ?? 'system';
  }

  // 语言设置
  static const _keyLocale = 'locale';

  Future<void> setLocale(String locale) async {
    await _preferences.setString(_keyLocale, locale);
  }

  String getLocale() {
    return _preferences.getString(_keyLocale) ?? 'zh';
  }

  // 通知设置
  static const _keyNotificationsEnabled = 'notifications_enabled';

  Future<void> setNotificationsEnabled(bool enabled) async {
    await _preferences.setBool(_keyNotificationsEnabled, enabled);
  }

  bool getNotificationsEnabled() {
    return _preferences.getBool(_keyNotificationsEnabled) ?? true;
  }

  // 消息提示音
  static const _keySoundEnabled = 'sound_enabled';

  Future<void> setSoundEnabled(bool enabled) async {
    await _preferences.setBool(_keySoundEnabled, enabled);
  }

  bool getSoundEnabled() {
    return _preferences.getBool(_keySoundEnabled) ?? true;
  }

  // 消息预览
  static const _keyMessagePreview = 'message_preview';

  Future<void> setMessagePreviewEnabled(bool enabled) async {
    await _preferences.setBool(_keyMessagePreview, enabled);
  }

  bool getMessagePreviewEnabled() {
    return _preferences.getBool(_keyMessagePreview) ?? true;
  }

  // ========== 通知设置扩展 ==========

  // 群组消息通知
  static const _keyGroupNotificationsEnabled = 'group_notifications_enabled';

  Future<void> setGroupNotificationsEnabled(bool enabled) async {
    await _preferences.setBool(_keyGroupNotificationsEnabled, enabled);
  }

  bool getGroupNotificationsEnabled() {
    return _preferences.getBool(_keyGroupNotificationsEnabled) ?? true;
  }

  // 私聊消息通知
  static const _keyPrivateNotificationsEnabled = 'private_notifications_enabled';

  Future<void> setPrivateNotificationsEnabled(bool enabled) async {
    await _preferences.setBool(_keyPrivateNotificationsEnabled, enabled);
  }

  bool getPrivateNotificationsEnabled() {
    return _preferences.getBool(_keyPrivateNotificationsEnabled) ?? true;
  }

  // 振动
  static const _keyVibrationEnabled = 'vibration_enabled';

  Future<void> setVibrationEnabled(bool enabled) async {
    await _preferences.setBool(_keyVibrationEnabled, enabled);
  }

  bool getVibrationEnabled() {
    return _preferences.getBool(_keyVibrationEnabled) ?? true;
  }

  // 提醒通知（@mention）
  static const _keyMentionNotificationsEnabled = 'mention_notifications_enabled';

  Future<void> setMentionNotificationsEnabled(bool enabled) async {
    await _preferences.setBool(_keyMentionNotificationsEnabled, enabled);
  }

  bool getMentionNotificationsEnabled() {
    return _preferences.getBool(_keyMentionNotificationsEnabled) ?? true;
  }
}
