import "dart:ui";

import "package:flutter/material.dart";
import "package:url_launcher/url_launcher.dart";

import "../../core/state/app_scope.dart";
import "../../core/theme/my_themes.dart";
import "../../l10n/app_localizations.dart";
import "neon_button.dart";

class ParallaxHeader extends StatelessWidget {
  const ParallaxHeader({
    super.key,
    this.onNavigateToSection,
  });

  final void Function(String sectionKey)? onNavigateToSection;

  Future<void> _openGithub() async {
    final uri = Uri.parse("https://github.com/asodya/parallax");
    if (await canLaunchUrl(uri)) {
      await launchUrl(uri, mode: LaunchMode.externalApplication);
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;
    final colorScheme = theme.colorScheme;
    final l10n = AppLocalizations.of(context);
    final controller = AppScope.of(context);

    final screenWidth = MediaQuery.of(context).size.width;
    final showNavLinks = screenWidth >= 1024;
    final showGithubBtn = screenWidth >= 640;

    return ClipRect(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
        child: Container(
          height: 64,
          padding: const EdgeInsets.symmetric(horizontal: 24),
          decoration: BoxDecoration(
            color: (isDark ? const Color(0xFF10110E) : const Color(0xFFF7F7F5))
                .withValues(alpha: 0.90),
            border: Border(
              bottom: BorderSide(
                color: isDark ? const Color(0xFF262626) : const Color(0xFFE5E7EB),
                width: 1,
              ),
            ),
          ),
          child: Row(
            children: [
              // ── Logo & Brand ─────────────────────────────────────
              InkWell(
                onTap: () => onNavigateToSection?.call("hero"),
                borderRadius: BorderRadius.circular(8),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Image.asset(
                      "assets/img/logo.png",
                      width: 26,
                      height: 26,
                      fit: BoxFit.contain,
                    ),
                    const SizedBox(width: 10),
                    Text(
                      l10n?.navBrand ?? "PARALLAX",
                      style: theme.textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.w700,
                        letterSpacing: 1.2,
                      ),
                    ),
                    const SizedBox(width: 8),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        border: Border.all(
                          color: isDark
                              ? const Color(0xFF333333)
                              : const Color(0xFFD1D5DB),
                        ),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        "v0.1",
                        style: theme.textTheme.labelSmall?.copyWith(
                          color: colorScheme.onSurfaceVariant,
                          fontSize: 10,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ),
                  ],
                ),
              ),

              // ── Nav Links (Desktop — centred) ─────────────────────
              if (showNavLinks)
                Expanded(
                  child: Center(
                    child: SingleChildScrollView(
                      scrollDirection: Axis.horizontal,
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          _NavLink(
                            label: "Meta Quest",
                            onTap: () => onNavigateToSection?.call("quest"),
                            isDark: isDark,
                          ),
                          _NavLink(
                            label: l10n?.navArchitecture ?? "Architecture",
                            onTap: () => onNavigateToSection?.call("architecture"),
                            isDark: isDark,
                          ),
                          _NavLink(
                            label: l10n?.navFeatures ?? "Features",
                            onTap: () => onNavigateToSection?.call("features"),
                            isDark: isDark,
                          ),
                          _NavLink(
                            label: l10n?.navHowItWorks ?? "How It Works",
                            onTap: () => onNavigateToSection?.call("howItWorks"),
                            isDark: isDark,
                          ),
                          _NavLink(
                            label: l10n?.navGetStarted ?? "Get Started",
                            onTap: () => onNavigateToSection?.call("gettingStarted"),
                            isDark: isDark,
                          ),
                          _NavLink(
                            label: l10n?.navProtocol ?? "Protocol",
                            onTap: () => onNavigateToSection?.call("protocol"),
                            isDark: isDark,
                          ),
                        ],
                      ),
                    ),
                  ),
                )
              else
                const Spacer(),

              // ── Right Controls ────────────────────────────────────
              // Language Selector
              PopupMenuButton<String>(
                tooltip: l10n?.navLanguage ?? "Language",
                icon: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      Icons.translate_outlined,
                      size: 18,
                      color: colorScheme.onSurface,
                    ),
                    const SizedBox(width: 4),
                    Text(
                      controller.locale.languageCode.toUpperCase(),
                      style: theme.textTheme.labelMedium?.copyWith(
                        fontWeight: FontWeight.w600,
                        color: colorScheme.onSurface,
                      ),
                    ),
                  ],
                ),
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(12),
                  side: BorderSide(
                    color: isDark ? const Color(0xFF333333) : const Color(0xFFE5E7EB),
                  ),
                ),
                color: isDark ? const Color(0xFF191A16) : Colors.white,
                elevation: 8,
                onSelected: (lang) {
                  if (lang == "en") {
                    controller.setLocale(const Locale("en"));
                  } else if (lang == "pt") {
                    controller.setLocale(const Locale("pt", "BR"));
                  } else if (lang == "th") {
                    controller.setLocale(const Locale("th"));
                  }
                },
                itemBuilder: (context) => [
                  _buildPopupItem("en", "English (EN)", controller.locale.languageCode == "en", theme, isDark),
                  _buildPopupItem("pt", "Português (PT-BR)", controller.locale.languageCode == "pt", theme, isDark),
                  _buildPopupItem("th", "ไทย (TH)", controller.locale.languageCode == "th", theme, isDark),
                ],
              ),

              // Theme Toggle
              IconButton(
                icon: Icon(
                  isDark ? Icons.light_mode_outlined : Icons.dark_mode_outlined,
                  size: 20,
                  color: colorScheme.onSurface,
                ),
                tooltip: isDark
                    ? (l10n?.navThemeLight ?? "Light Mode")
                    : (l10n?.navThemeDark ?? "Dark Mode"),
                onPressed: () => controller.toggleThemeMode(),
              ),

              if (showGithubBtn) ...[
                const SizedBox(width: 8),
                NeonButton(
                  label: "GitHub",
                  isPrimary: false,
                  icon: const Icon(Icons.code_outlined, size: 16),
                  onPressed: _openGithub,
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  PopupMenuItem<String> _buildPopupItem(
    String value,
    String label,
    bool isSelected,
    ThemeData theme,
    bool isDark,
  ) {
    final accent = isDark ? MyThemes.brandLime : MyThemes.brandPurple;
    return PopupMenuItem<String>(
      value: value,
      child: Row(
        children: [
          Expanded(
            child: Text(
              label,
              style: theme.textTheme.bodyMedium?.copyWith(
                fontWeight: isSelected ? FontWeight.w700 : FontWeight.w400,
                color: isSelected ? accent : null,
              ),
            ),
          ),
          if (isSelected)
            Icon(Icons.check, size: 16, color: accent),
        ],
      ),
    );
  }
}

class _NavLink extends StatefulWidget {
  const _NavLink({
    required this.label,
    required this.onTap,
    required this.isDark,
  });

  final String label;
  final VoidCallback onTap;
  final bool isDark;

  @override
  State<_NavLink> createState() => _NavLinkState();
}

class _NavLinkState extends State<_NavLink> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final accent = widget.isDark ? MyThemes.brandLime : MyThemes.brandPurple;

    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 150),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            border: Border(
              bottom: BorderSide(
                color: _hovered ? accent : Colors.transparent,
                width: 1.5,
              ),
            ),
          ),
          child: Text(
            widget.label,
            style: theme.textTheme.bodyMedium?.copyWith(
              color: _hovered ? colorScheme.onSurface : colorScheme.onSurfaceVariant,
              fontWeight: _hovered ? FontWeight.w600 : FontWeight.w500,
            ),
          ),
        ),
      ),
    );
  }
}
