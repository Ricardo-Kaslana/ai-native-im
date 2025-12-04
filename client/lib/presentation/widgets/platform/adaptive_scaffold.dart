import 'package:flutter/material.dart';
import '../../../core/utils/platform_utils.dart';

/// Adaptive scaffold that adjusts layout based on platform and screen size
class AdaptiveScaffold extends StatelessWidget {
  final Widget body;
  final Widget? sidebar;
  final Widget? detailPane;
  final PreferredSizeWidget? appBar;
  final Widget? floatingActionButton;
  final Widget? bottomNavigationBar;
  final double sidebarWidth;
  final double detailPaneMinWidth;

  const AdaptiveScaffold({
    super.key,
    required this.body,
    this.sidebar,
    this.detailPane,
    this.appBar,
    this.floatingActionButton,
    this.bottomNavigationBar,
    this.sidebarWidth = 300,
    this.detailPaneMinWidth = 400,
  });

  @override
  Widget build(BuildContext context) {
    final screenWidth = MediaQuery.of(context).size.width;
    final isWideScreen = screenWidth >= 900;
    final isExtraWideScreen = screenWidth >= 1200;

    // Desktop: Multi-column layout
    if (PlatformUtils.isDesktop && isWideScreen) {
      return Scaffold(
        appBar: appBar,
        body: Row(
          children: [
            // Sidebar
            if (sidebar != null)
              SizedBox(
                width: sidebarWidth,
                child: sidebar,
              ),
            if (sidebar != null)
              const VerticalDivider(width: 1),

            // Main content
            Expanded(child: body),

            // Detail pane (for extra wide screens)
            if (detailPane != null && isExtraWideScreen) ...[
              const VerticalDivider(width: 1),
              SizedBox(
                width: detailPaneMinWidth,
                child: detailPane,
              ),
            ],
          ],
        ),
        floatingActionButton: floatingActionButton,
      );
    }

    // Mobile/Tablet: Standard layout
    return Scaffold(
      appBar: appBar,
      body: body,
      floatingActionButton: floatingActionButton,
      bottomNavigationBar: bottomNavigationBar,
    );
  }
}

/// Responsive builder widget
class ResponsiveBuilder extends StatelessWidget {
  final Widget Function(BuildContext context, ScreenSize screenSize) builder;

  const ResponsiveBuilder({
    super.key,
    required this.builder,
  });

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final screenSize = ScreenSize.fromWidth(constraints.maxWidth);
        return builder(context, screenSize);
      },
    );
  }
}

/// Screen size categories
enum ScreenSize {
  compact,  // < 600
  medium,   // 600 - 900
  expanded, // 900 - 1200
  large,    // >= 1200
  ;

  factory ScreenSize.fromWidth(double width) {
    if (width < 600) return ScreenSize.compact;
    if (width < 900) return ScreenSize.medium;
    if (width < 1200) return ScreenSize.expanded;
    return ScreenSize.large;
  }

  bool get isCompact => this == ScreenSize.compact;
  bool get isMedium => this == ScreenSize.medium;
  bool get isExpanded => this == ScreenSize.expanded;
  bool get isLarge => this == ScreenSize.large;
  bool get isWide => isExpanded || isLarge;
}

/// Platform-adaptive navigation rail
class AdaptiveNavigationRail extends StatelessWidget {
  final int selectedIndex;
  final ValueChanged<int> onDestinationSelected;
  final List<NavigationRailDestination> destinations;

  const AdaptiveNavigationRail({
    super.key,
    required this.selectedIndex,
    required this.onDestinationSelected,
    required this.destinations,
  });

  @override
  Widget build(BuildContext context) {
    return NavigationRail(
      selectedIndex: selectedIndex,
      onDestinationSelected: onDestinationSelected,
      labelType: NavigationRailLabelType.all,
      destinations: destinations,
    );
  }
}
