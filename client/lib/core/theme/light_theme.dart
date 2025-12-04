import 'package:flutter/material.dart';
import '../constants/colors.dart';
import '../constants/typography.dart';
import '../constants/spacing.dart';

/// Light theme configuration
ThemeData lightTheme = ThemeData(
  useMaterial3: true,
  brightness: Brightness.light,
  primaryColor: AppColors.lightPrimary,
  scaffoldBackgroundColor: AppColors.lightBackground,

  colorScheme: const ColorScheme.light(
    primary: AppColors.lightPrimary,
    onPrimary: Colors.white,
    secondary: AppColors.lightPrimary,
    surface: AppColors.lightSurface,
    onSurface: AppColors.lightText,
    error: AppColors.lightError,
    onError: Colors.white,
  ),

  textTheme: TextTheme(
    displayLarge: AppTypography.h1.copyWith(color: AppColors.lightText),
    displayMedium: AppTypography.h2.copyWith(color: AppColors.lightText),
    displaySmall: AppTypography.h3.copyWith(color: AppColors.lightText),
    bodyLarge: AppTypography.bodyLarge.copyWith(color: AppColors.lightText),
    bodyMedium: AppTypography.body.copyWith(color: AppColors.lightText),
    bodySmall: AppTypography.bodySmall.copyWith(color: AppColors.lightTextSecondary),
    labelLarge: AppTypography.button.copyWith(color: AppColors.lightText),
    labelMedium: AppTypography.label.copyWith(color: AppColors.lightText),
    labelSmall: AppTypography.caption.copyWith(color: AppColors.lightTextSecondary),
  ),

  appBarTheme: const AppBarTheme(
    backgroundColor: AppColors.lightBackground,
    foregroundColor: AppColors.lightText,
    elevation: 0,
    centerTitle: true,
    scrolledUnderElevation: 0.5,
  ),

  cardTheme: CardTheme(
    color: AppColors.lightSurface,
    elevation: 0,
    shape: RoundedRectangleBorder(
      borderRadius: BorderRadius.circular(Spacing.radiusMd),
    ),
  ),

  inputDecorationTheme: InputDecorationTheme(
    filled: true,
    fillColor: AppColors.lightSurface,
    border: OutlineInputBorder(
      borderRadius: BorderRadius.circular(Spacing.radiusMd),
      borderSide: BorderSide.none,
    ),
    enabledBorder: OutlineInputBorder(
      borderRadius: BorderRadius.circular(Spacing.radiusMd),
      borderSide: BorderSide.none,
    ),
    focusedBorder: OutlineInputBorder(
      borderRadius: BorderRadius.circular(Spacing.radiusMd),
      borderSide: const BorderSide(color: AppColors.lightPrimary, width: 2),
    ),
    errorBorder: OutlineInputBorder(
      borderRadius: BorderRadius.circular(Spacing.radiusMd),
      borderSide: const BorderSide(color: AppColors.lightError),
    ),
    contentPadding: const EdgeInsets.symmetric(
      horizontal: Spacing.md,
      vertical: Spacing.sm,
    ),
  ),

  elevatedButtonTheme: ElevatedButtonThemeData(
    style: ElevatedButton.styleFrom(
      backgroundColor: AppColors.lightPrimary,
      foregroundColor: Colors.white,
      elevation: 0,
      padding: const EdgeInsets.symmetric(
        horizontal: Spacing.lg,
        vertical: Spacing.md,
      ),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(Spacing.radiusMd),
      ),
      textStyle: AppTypography.button,
    ),
  ),

  textButtonTheme: TextButtonThemeData(
    style: TextButton.styleFrom(
      foregroundColor: AppColors.lightPrimary,
      textStyle: AppTypography.button,
    ),
  ),

  dividerTheme: const DividerThemeData(
    color: AppColors.lightDivider,
    thickness: 1,
    space: 0,
  ),

  bottomNavigationBarTheme: const BottomNavigationBarThemeData(
    backgroundColor: AppColors.lightBackground,
    selectedItemColor: AppColors.lightPrimary,
    unselectedItemColor: AppColors.lightTextSecondary,
    type: BottomNavigationBarType.fixed,
    elevation: 8,
  ),

  floatingActionButtonTheme: const FloatingActionButtonThemeData(
    backgroundColor: AppColors.lightPrimary,
    foregroundColor: Colors.white,
    elevation: 4,
  ),

  snackBarTheme: SnackBarThemeData(
    backgroundColor: AppColors.lightText,
    contentTextStyle: AppTypography.body.copyWith(color: Colors.white),
    behavior: SnackBarBehavior.floating,
    shape: RoundedRectangleBorder(
      borderRadius: BorderRadius.circular(Spacing.radiusSm),
    ),
  ),
);
