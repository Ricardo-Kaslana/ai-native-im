import 'package:flutter/material.dart';

/// Custom page route with smooth transitions
class SmoothPageRoute<T> extends PageRouteBuilder<T> {
  final Widget page;
  final RouteSettings? routeSettings;

  SmoothPageRoute({
    required this.page,
    this.routeSettings,
  }) : super(
          settings: routeSettings,
          pageBuilder: (context, animation, secondaryAnimation) => page,
          transitionsBuilder: (context, animation, secondaryAnimation, child) {
            const begin = Offset(1.0, 0.0);
            const end = Offset.zero;
            const curve = Curves.easeInOutCubic;

            var tween = Tween(begin: begin, end: end).chain(
              CurveTween(curve: curve),
            );

            var offsetAnimation = animation.drive(tween);
            var fadeAnimation = animation.drive(
              Tween(begin: 0.0, end: 1.0).chain(
                CurveTween(curve: curve),
              ),
            );

            return SlideTransition(
              position: offsetAnimation,
              child: FadeTransition(
                opacity: fadeAnimation,
                child: child,
              ),
            );
          },
          transitionDuration: const Duration(milliseconds: 300),
          reverseTransitionDuration: const Duration(milliseconds: 250),
        );
}

/// Fade page route for modals and overlays
class FadePageRoute<T> extends PageRouteBuilder<T> {
  final Widget page;
  final RouteSettings? routeSettings;

  FadePageRoute({
    required this.page,
    this.routeSettings,
  }) : super(
          settings: routeSettings,
          pageBuilder: (context, animation, secondaryAnimation) => page,
          transitionsBuilder: (context, animation, secondaryAnimation, child) {
            return FadeTransition(
              opacity: animation.drive(
                Tween(begin: 0.0, end: 1.0).chain(
                  CurveTween(curve: Curves.easeOut),
                ),
              ),
              child: child,
            );
          },
          transitionDuration: const Duration(milliseconds: 200),
          reverseTransitionDuration: const Duration(milliseconds: 150),
        );
}

/// Scale page route for dialogs
class ScalePageRoute<T> extends PageRouteBuilder<T> {
  final Widget page;
  final RouteSettings? routeSettings;

  ScalePageRoute({
    required this.page,
    this.routeSettings,
  }) : super(
          settings: routeSettings,
          opaque: false,
          pageBuilder: (context, animation, secondaryAnimation) => page,
          transitionsBuilder: (context, animation, secondaryAnimation, child) {
            return ScaleTransition(
              scale: animation.drive(
                Tween(begin: 0.9, end: 1.0).chain(
                  CurveTween(curve: Curves.easeOutBack),
                ),
              ),
              child: FadeTransition(
                opacity: animation,
                child: child,
              ),
            );
          },
          transitionDuration: const Duration(milliseconds: 250),
          reverseTransitionDuration: const Duration(milliseconds: 200),
        );
}

/// Bottom sheet style page route
class BottomSheetPageRoute<T> extends PageRouteBuilder<T> {
  final Widget page;
  final RouteSettings? routeSettings;

  BottomSheetPageRoute({
    required this.page,
    this.routeSettings,
  }) : super(
          settings: routeSettings,
          opaque: false,
          pageBuilder: (context, animation, secondaryAnimation) => page,
          transitionsBuilder: (context, animation, secondaryAnimation, child) {
            const begin = Offset(0.0, 1.0);
            const end = Offset.zero;

            return SlideTransition(
              position: animation.drive(
                Tween(begin: begin, end: end).chain(
                  CurveTween(curve: Curves.easeOutCubic),
                ),
              ),
              child: child,
            );
          },
          transitionDuration: const Duration(milliseconds: 300),
          reverseTransitionDuration: const Duration(milliseconds: 250),
        );
}
