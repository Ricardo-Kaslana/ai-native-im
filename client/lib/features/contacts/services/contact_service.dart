import '../../../core/network/api_client.dart';
import '../models/contact.dart';
import '../../auth/models/user.dart';

/// 联系人服务
class ContactService {
  static final ContactService instance = ContactService._();
  ContactService._();

  final _api = ApiClient.instance;

  /// 获取联系人列表
  Future<List<Contact>> getContacts() async {
    return await _api.get<List<Contact>>(
      '/contacts',
      fromJson: (json) {
        // 后端返回 { success: true, data: [...] }，ApiClient 提取后是数组
        final List list;
        if (json is List) {
          list = json;
        } else if (json is Map) {
          list = json['contacts'] as List? ?? json['data'] as List? ?? [];
        } else {
          list = [];
        }

        return list
            .map((e) => Contact.fromJson(Map<String, dynamic>.from(e)))
            .toList();
      },
    );
  }

  /// 添加联系人
  Future<void> addContact(String contactId) async {
    await _api.postEmpty(
      '/contacts',
      data: {'contact_id': contactId},
    );
  }

  /// 删除联系人
  Future<void> deleteContact(String contactId) async {
    await _api.deleteEmpty('/contacts/$contactId');
  }

  /// 更新联系人备注
  Future<Contact> updateRemark(String contactId, String remark) async {
    return await _api.put<Contact>(
      '/contacts/$contactId',
      data: {'remark': remark},
      fromJson: (json) => Contact.fromJson(json as Map<String, dynamic>),
    );
  }

  /// 搜索用户（用于添加联系人）
  Future<List<User>> searchUsers(String query) async {
    return await _api.get<List<User>>(
      '/users/search',
      queryParameters: {'q': query},
      fromJson: (json) {
        // 后端返回 { success: true, data: [...] }，ApiClient 提取后是数组
        final List list;
        if (json is List) {
          list = json;
        } else if (json is Map) {
          list = json['users'] as List? ?? json['data'] as List? ?? [];
        } else {
          list = [];
        }
        return list
            .map((e) => User.fromJson(e as Map<String, dynamic>))
            .toList();
      },
    );
  }
}
