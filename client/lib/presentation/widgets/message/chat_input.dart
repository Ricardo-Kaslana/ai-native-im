import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../../../core/constants/spacing.dart';

/// Chat input widget
class ChatInput extends StatefulWidget {
  final Function(String) onSend;
  final VoidCallback? onMediaTap;

  const ChatInput({
    super.key,
    required this.onSend,
    this.onMediaTap,
  });

  @override
  State<ChatInput> createState() => _ChatInputState();
}

class _ChatInputState extends State<ChatInput> {
  final _controller = TextEditingController();
  final _focusNode = FocusNode();
  bool _hasText = false;

  @override
  void initState() {
    super.initState();
    _controller.addListener(_onTextChanged);
  }

  @override
  void dispose() {
    _controller.removeListener(_onTextChanged);
    _controller.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  void _onTextChanged() {
    final hasText = _controller.text.trim().isNotEmpty;
    if (hasText != _hasText) {
      setState(() => _hasText = hasText);
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;

    return Container(
      padding: EdgeInsets.only(
        left: Spacing.md,
        right: Spacing.md,
        top: Spacing.sm,
        bottom: MediaQuery.of(context).padding.bottom + Spacing.sm,
      ),
      decoration: BoxDecoration(
        color: theme.scaffoldBackgroundColor,
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.05),
            blurRadius: 8,
            offset: const Offset(0, -2),
          ),
        ],
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          // Media button
          IconButton(
            icon: const Icon(Icons.add_photo_alternate_outlined),
            onPressed: widget.onMediaTap,
            color: theme.colorScheme.primary,
          ),

          // Text input
          Expanded(
            child: Container(
              constraints: const BoxConstraints(maxHeight: 120),
              decoration: BoxDecoration(
                color: isDark
                    ? const Color(0xFF1E1E1E)
                    : const Color(0xFFF5F5F5),
                borderRadius: BorderRadius.circular(Spacing.radiusXl),
              ),
              child: TextField(
                controller: _controller,
                focusNode: _focusNode,
                maxLines: null,
                textInputAction: TextInputAction.newline,
                decoration: const InputDecoration(
                  hintText: '输入消息...',
                  border: InputBorder.none,
                  contentPadding: EdgeInsets.symmetric(
                    horizontal: Spacing.md,
                    vertical: Spacing.sm,
                  ),
                ),
                onSubmitted: (_) => _sendMessage(),
              ),
            ),
          ),

          const SizedBox(width: Spacing.xs),

          // Send button
          AnimatedScale(
            scale: _hasText ? 1.0 : 0.0,
            duration: const Duration(milliseconds: 150),
            child: AnimatedOpacity(
              opacity: _hasText ? 1.0 : 0.0,
              duration: const Duration(milliseconds: 150),
              child: Container(
                decoration: BoxDecoration(
                  color: theme.colorScheme.primary,
                  shape: BoxShape.circle,
                ),
                child: IconButton(
                  icon: const Icon(Icons.send),
                  color: Colors.white,
                  onPressed: _sendMessage,
                ),
              ),
            ),
          ),

          // Voice button (when no text)
          AnimatedScale(
            scale: _hasText ? 0.0 : 1.0,
            duration: const Duration(milliseconds: 150),
            child: AnimatedOpacity(
              opacity: _hasText ? 0.0 : 1.0,
              duration: const Duration(milliseconds: 150),
              child: IconButton(
                icon: const Icon(Icons.mic_outlined),
                onPressed: () {
                  // TODO: Voice recording
                },
                color: theme.colorScheme.primary,
              ),
            ),
          ),
        ],
      ),
    );
  }

  void _sendMessage() {
    final text = _controller.text.trim();
    if (text.isNotEmpty) {
      HapticFeedback.lightImpact();
      widget.onSend(text);
      _controller.clear();
    }
  }
}
