import '../config/app_config.dart';

class UrlUtils {
  UrlUtils._();

  /// Processes a media URL to make it accessible from the client.
  ///
  /// - Replaces `localhost` with the actual server host from [AppConfig.apiBaseUrl]
  /// - Prepends the API base URL for relative paths
  static String processMediaUrl(String url) {
    if (url.startsWith('http://') || url.startsWith('https://')) {
      // Replace localhost with the actual server host
      if (url.contains('localhost')) {
        final baseUri = Uri.parse(AppConfig.apiBaseUrl);
        final originalUri = Uri.parse(url);
        return originalUri
            .replace(host: baseUri.host, port: baseUri.port)
            .toString();
      }
      return url;
    }
    final base = AppConfig.apiBaseUrl;
    if (url.startsWith('/')) {
      return '$base$url';
    }
    return '$base/$url';
  }
}
