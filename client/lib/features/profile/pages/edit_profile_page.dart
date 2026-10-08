import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:image_picker/image_picker.dart';
import '../../auth/models/user.dart';
import '../../auth/services/auth_service.dart';
import '../services/user_service.dart';
import '../services/avatar_service.dart';
import '../../shared/widgets/user_avatar.dart';

/// 编辑个人资料页面
class EditProfilePage extends StatefulWidget {
  const EditProfilePage({super.key});

  @override
  State<EditProfilePage> createState() => _EditProfilePageState();
}

class _EditProfilePageState extends State<EditProfilePage> {
  final _formKey = GlobalKey<FormState>();
  final _nicknameController = TextEditingController();
  final _bioController = TextEditingController();
  final _userService = UserService.instance;
  final _authService = AuthService.instance;
  final _avatarService = AvatarService.instance;

  User? _user;
  bool _isLoading = true;
  bool _isSaving = false;
  bool _isUploadingAvatar = false;
  String? _newAvatarUrl;

  @override
  void initState() {
    super.initState();
    _loadUser();
  }

  @override
  void dispose() {
    _nicknameController.dispose();
    _bioController.dispose();
    super.dispose();
  }

  Future<void> _loadUser() async {
    setState(() => _isLoading = true);
    try {
      final user = await _authService.getCurrentUser();
      if (mounted && user != null) {
        setState(() {
          _user = user;
          _nicknameController.text = user.displayName;
          _bioController.text = user.bio ?? '';
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() => _isLoading = false);
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('加载失败: $e')),
        );
      }
    }
  }

  Future<void> _save() async {
    if (!_formKey.currentState!.validate()) return;

    setState(() => _isSaving = true);

    try {
      await _userService.updateProfile(
        bio: _bioController.text.trim(),
        avatarUrl: _newAvatarUrl,
      );
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('保存成功')),
        );
        context.pop();
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('保存失败: $e')),
        );
      }
    } finally {
      if (mounted) {
        setState(() => _isSaving = false);
      }
    }
  }

  /// 上传头像
  Future<void> _uploadAvatar(ImageSource source) async {
    setState(() => _isUploadingAvatar = true);

    try {
      final avatarUrl = await _avatarService.pickAndUploadAvatar(
        source: source,
        onProgress: (sent, total) {
          debugPrint('Upload progress: $sent/$total');
        },
      );

      if (avatarUrl != null && mounted) {
        setState(() {
          _newAvatarUrl = avatarUrl;
          if (_user != null) {
            _user = _user!.copyWith(avatarUrl: avatarUrl);
          }
        });

        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('头像已选择，请点击保存按钮保存更改')),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('头像选择失败: $e')),
        );
      }
    } finally {
      if (mounted) {
        setState(() => _isUploadingAvatar = false);
      }
    }
  }

  /// 显示头像选择对话框
  void _showAvatarSourceDialog() {
    showModalBottomSheet(
      context: context,
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.photo_library),
              title: const Text('从相册选择'),
              onTap: () {
                Navigator.pop(context);
                _uploadAvatar(ImageSource.gallery);
              },
            ),
            ListTile(
              leading: const Icon(Icons.camera_alt),
              title: const Text('拍照'),
              onTap: () {
                Navigator.pop(context);
                _uploadAvatar(ImageSource.camera);
              },
            ),
            ListTile(
              leading: const Icon(Icons.cancel),
              title: const Text('取消'),
              onTap: () => Navigator.pop(context),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.pop(),
        ),
        title: const Text('编辑资料'),
        actions: [
          TextButton(
            onPressed: _isSaving ? null : _save,
            child: _isSaving
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Text('保存'),
          ),
        ],
      ),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : Form(
              key: _formKey,
              child: ListView(
                padding: const EdgeInsets.all(16),
                children: [
                  // 头像
                  Center(
                    child: Stack(
                      children: [
                        UserAvatar(
                          name: _user?.displayName ?? '',
                          avatarUrl: _newAvatarUrl ?? _user?.avatarUrl,
                          size: 96,
                        ),
                        Positioned(
                          right: 0,
                          bottom: 0,
                          child: _isUploadingAvatar
                              ? Container(
                                  decoration: BoxDecoration(
                                    color: theme.colorScheme.primary,
                                    shape: BoxShape.circle,
                                  ),
                                  child: const Padding(
                                    padding: EdgeInsets.all(12),
                                    child: SizedBox(
                                      width: 16,
                                      height: 16,
                                      child: CircularProgressIndicator(
                                        strokeWidth: 2,
                                        valueColor: AlwaysStoppedAnimation<Color>(Colors.white),
                                      ),
                                    ),
                                  ),
                                )
                              : Container(
                                  decoration: BoxDecoration(
                                    color: theme.colorScheme.primary,
                                    shape: BoxShape.circle,
                                  ),
                                  child: IconButton(
                                    icon: const Icon(Icons.camera_alt, size: 20),
                                    color: Colors.white,
                                    onPressed: _showAvatarSourceDialog,
                                  ),
                                ),
                        ),
                      ],
                    ),
                  ),
                  if (_newAvatarUrl != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 16),
                      child: Text(
                        '头像已更新，记得点击保存按钮',
                        style: theme.textTheme.bodySmall?.copyWith(
                          color: theme.colorScheme.primary,
                        ),
                      ),
                    ),
                  const SizedBox(height: 32),

                  // 用户名（不可编辑）
                  TextFormField(
                    initialValue: _user?.username ?? '',
                    decoration: const InputDecoration(
                      labelText: '用户名',
                      helperText: '用户名不可修改',
                    ),
                    enabled: false,
                  ),
                  const SizedBox(height: 16),

                  // // 昵称（后端不支持 nickname，暂时隐藏）
                  // TextFormField(
                  //   controller: _nicknameController,
                  //   decoration: const InputDecoration(
                  //     labelText: '昵称',
                  //     hintText: '设置一个昵称',
                  //   ),
                  //   maxLength: 20,
                  // ),
                  // const SizedBox(height: 16),

                  // 个人简介
                  TextFormField(
                    controller: _bioController,
                    decoration: const InputDecoration(
                      labelText: '个人简介',
                      hintText: '介绍一下自己',
                    ),
                    maxLines: 3,
                    maxLength: 100,
                  ),
                ],
              ),
            ),
    );
  }
}
