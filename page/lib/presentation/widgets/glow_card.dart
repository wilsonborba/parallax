import "package:flutter/material.dart";

import "../../core/theme/my_themes.dart";

class GlowCard extends StatefulWidget {
  const GlowCard({
    super.key,
    required this.child,
    this.onTap,
    this.padding,
  });

  final Widget child;
  final VoidCallback? onTap;
  final EdgeInsetsGeometry? padding;

  @override
  State<GlowCard> createState() => _GlowCardState();
}

class _GlowCardState extends State<GlowCard> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;

    final accent = isDark ? MyThemes.brandLime : MyThemes.brandPurple;

    final borderColor = _hovered
        ? accent.withValues(alpha: isDark ? 0.45 : 0.35)
        : colorScheme.outline.withValues(alpha: isDark ? 0.4 : 0.7);

    final cardBg = theme.cardTheme.color ?? colorScheme.surface;

    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        padding: widget.padding ?? const EdgeInsets.all(24),
        decoration: BoxDecoration(
          color: cardBg,
          borderRadius: BorderRadius.circular(16),
          border: Border.all(color: borderColor, width: 1),
          boxShadow: [
            BoxShadow(
              color: _hovered
                  ? accent.withValues(alpha: isDark ? 0.08 : 0.06)
                  : Colors.black.withValues(alpha: isDark ? 0.15 : 0.03),
              blurRadius: _hovered ? 22 : 10,
              offset: const Offset(0, 4),
            ),
          ],
        ),
        transform: Matrix4.translationValues(0, _hovered ? -3.0 : 0.0, 0),
        child: Material(
          color: Colors.transparent,
          child: InkWell(
            onTap: widget.onTap,
            borderRadius: BorderRadius.circular(16),
            child: widget.child,
          ),
        ),
      ),
    );
  }
}
