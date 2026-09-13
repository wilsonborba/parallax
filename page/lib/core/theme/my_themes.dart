import "package:flutter/material.dart";
import "package:google_fonts/google_fonts.dart";

/// Central Asodya Design System Theme tokens and configurations.
abstract final class MyThemes {
  static const brandLime = Color(0xFFD7FF3F);
  static const brandPurple = Color(0xFF7027C8);
  static const brandPurpleLight = Color(0xFFB990FF);

  static const lightBackground = Color(0xFFF7F7F5);
  static const lightSurface = Color(0xFFFFFFFF);
  static const lightCard = Color(0xFFFFFFFF);
  static const lightText = Color(0xFF181818);
  static const lightMutedText = Color(0xFF5B5B57);
  static const lightBorder = Color(0xFFE5E7EB);

  static const darkBackground = Color(0xFF10110E);
  static const darkSurface = Color(0xFF191A16);
  static const darkCard = Color(0xFF191A16);
  static const darkText = Color(0xFFF5F5F0);
  static const darkMutedText = Color(0xFFC6C7BF);
  static const darkBorder = Color(0xFF262626);

  static ThemeData light() {
    final baseTextTheme = GoogleFonts.ubuntuTextTheme(ThemeData.light().textTheme);
    final scheme = const ColorScheme.light(
      primary: lightText,
      onPrimary: lightSurface,
      secondary: lightText,
      onSecondary: lightSurface,
      surface: lightSurface,
      onSurface: lightText,
      surfaceContainerHighest: lightBackground,
      onSurfaceVariant: lightMutedText,
      outline: lightBorder,
      outlineVariant: Color(0xFFD1D5DB),
    );

    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.light,
      scaffoldBackgroundColor: lightBackground,
      colorScheme: scheme,
      textTheme: baseTextTheme.apply(
        bodyColor: lightText,
        displayColor: lightText,
      ),
      cardTheme: CardThemeData(
        color: lightCard,
        surfaceTintColor: lightCard,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(16),
          side: const BorderSide(color: lightBorder, width: 1),
        ),
        elevation: 0,
      ),
      dividerColor: lightBorder,
    );
  }

  static ThemeData dark() {
    final baseTextTheme = GoogleFonts.ubuntuTextTheme(ThemeData.dark().textTheme);
    final scheme = const ColorScheme.dark(
      primary: darkText,
      onPrimary: darkBackground,
      secondary: darkText,
      onSecondary: darkBackground,
      surface: darkSurface,
      onSurface: darkText,
      surfaceContainerHighest: darkBackground,
      onSurfaceVariant: darkMutedText,
      outline: darkBorder,
      outlineVariant: Color(0xFF333333),
    );

    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      scaffoldBackgroundColor: darkBackground,
      colorScheme: scheme,
      textTheme: baseTextTheme.apply(
        bodyColor: darkText,
        displayColor: darkText,
      ),
      cardTheme: CardThemeData(
        color: darkCard,
        surfaceTintColor: darkCard,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(16),
          side: const BorderSide(color: darkBorder, width: 1),
        ),
        elevation: 0,
      ),
      dividerColor: darkBorder,
    );
  }
}
