import 'package:flutter/material.dart';

/// Custom bouncing scroll physics with smoother feel
class SmoothBouncingScrollPhysics extends BouncingScrollPhysics {
  const SmoothBouncingScrollPhysics({super.parent});

  @override
  SmoothBouncingScrollPhysics applyTo(ScrollPhysics? ancestor) {
    return SmoothBouncingScrollPhysics(parent: buildParent(ancestor));
  }

  @override
  double get dragStartDistanceMotionThreshold => 3.5;

  @override
  double frictionFactor(double overscrollFraction) {
    return 0.52 * (1 - overscrollFraction * overscrollFraction);
  }

  @override
  SpringDescription get spring => const SpringDescription(
        mass: 0.5,
        stiffness: 100,
        damping: 1.1,
      );
}

/// Custom clamping scroll physics for lists that shouldn't bounce
class SmoothClampingScrollPhysics extends ClampingScrollPhysics {
  const SmoothClampingScrollPhysics({super.parent});

  @override
  SmoothClampingScrollPhysics applyTo(ScrollPhysics? ancestor) {
    return SmoothClampingScrollPhysics(parent: buildParent(ancestor));
  }

  @override
  double get dragStartDistanceMotionThreshold => 3.5;
}

/// Page scroll physics for smooth page snapping
class SmoothPageScrollPhysics extends PageScrollPhysics {
  const SmoothPageScrollPhysics({super.parent});

  @override
  SmoothPageScrollPhysics applyTo(ScrollPhysics? ancestor) {
    return SmoothPageScrollPhysics(parent: buildParent(ancestor));
  }

  @override
  SpringDescription get spring => const SpringDescription(
        mass: 0.8,
        stiffness: 100,
        damping: 1.2,
      );
}
