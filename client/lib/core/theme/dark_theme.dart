import 'package:flutter/material.dart';
import '../constants/colors.dart';
import '../constants/typography.dart';
import '../constants/spacing.dart';

/// Dark theme configuration (OLED-friendly)
ThemeData darkTheme = ThemeData(
  useMaterial3: true,
  brightness: Brightness.dark,
  primaryColor: AppColors.darkPrimary,
  scaffoldBackgroundColor: AppColors.darkBackground,

  colorScheme: const ColorScheme.dark(
    primary: AppColors.darkPrimary,
    onPrimary: Colors.black,
    secondary: AppColors.darkPrimary,
    surface: AppColors.darkSurface,
    onSurface: AppColors.darkText,
    error: AppColors.darkError,
    onError: Colors.black,
  ),

  textTheme: TextTheme(
    displayLarge: AppTypography.h1.copyWith(color: AppColors.darkText),
    displayMedium: AppTypography.h2.copyWith(color: AppColors.darkText),
    displaySmall: AppTypography.h3.copyWith(color: AppColors.darkText),
    bodyLarge: AppTypography.bodyLarge.copyWith(color: AppColors.darkText),
    bodyMedium: AppTypography.body.copyWith(color: AppColors.darkText),
    bodySmall: AppTypography.bodySmall.copyWith(color: AppColors.darkTextSecondary),
    labelLarge: AppTypography.button.copyWith(color: AppColors.darkText),
    labelMedium: AppTypography.label.copyWith(color: AppColors.darkText),
    labelSmall: AppTypography.caption.copyWith(color: AppColors.darkTextSecondary),
  ),

  appBarTheme: const AppBarTheme(
    backgroundColor: AppColors.darkBackground,
    foregroundColor: AppColors.darkText,
    elevation: 0,
    centerTitle: true,
    scrolledUnderElevation: 0.5,
  ),

  cardTheme: CardTheme(
    color: AppColors.darkSurface,
    elevation: 0,
    shape: RoundedRectangleBorder(
      borderRadius: BorderRadius.circular(Spacing.radiusMd),
    ),
  ),

  inputDecorationTheme: InputDecorationTheme(
    filled: true,
    fillColor: AppColors.darkSurface,
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
      borderSide: const BorderSide(color: AppColors.darkPrimary, width: 2),
    ),
    errorBorder: OutlineInputBorder(
      borderRadius: BorderRadius.circular(Spacing.radiusMd),
      borderSide: const BorderSide(color: AppColors.darkError),
    ),
    contentPadding: const EdgeInsets.symmetric(
      horizontal: Spacing.md,
      vertical: Spacing.sm,
    ),
  ),

  elevatedButtonTheme: ElevatedButtonThemeData(
    style: ElevatedButton.styleFrom(
      backgroundColor: AppColors.darkPrimary,
      foregroundColor: Colors.black,
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
      foregroundColor: AppColors.darkPrimary,
      textStyle: AppTypography.button,
    ),
  ),

  dividerTheme: const DividerThemeData(
    color: AppColors.darkDivider,
    thickness: 1,
    space: 0,
  ),

  bottomNavigationBarTheme: const BottomNavigationBarThemeData(
    backgroundColor: AppColors.darkBackground,
    selectedItemColor: AppColors.darkPrimary,
    unselectedItemColor: AppColors.darkTextSecondary,
    type: BottomNavigationBarType.fixed,
    elevation: 8,
  ),

  floatingActionButtonTheme: const FloatingActionButtonThemeData(
    backgroundColor: AppColors.darkPrimary,
    foregroundColor: Colors.black,
    elevation: 4,
  ),

  snackBarTheme: SnackBarThemeData(
    backgroundColor: AppColors.darkSurface,
    contentTextStyle: AppTypography.body.copyWith(color: AppColors.darkText),
    behavior: SnackBarBehavior.floating,
    shape: RoundedRectangleBorder(
      borderRadius: BorderRadius.circular(Spacing.radiusSm),
    ),
  ),
);
