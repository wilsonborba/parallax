import "package:flutter/material.dart";

import "../../core/theme/my_themes.dart";

class NeonButton extends StatefulWidget {
  const NeonButton({
    super.key,
    required this.label,
    required this.onPressed,
    this.isPrimary = true,
    this.icon,
  });

  final String label;
  final VoidCallback onPressed;
  final bool isPrimary;
  final Widget? icon;

  @override
  State<NeonButton> createState() => _NeonButtonState();
}

class _NeonButtonState extends State<NeonButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;
    final colorScheme = theme.colorScheme;

    final Color bgColor;
    final Color textColor;
    final Border? border;
    List<BoxShadow>? shadows;

    if (widget.isPrimary) {
      if (isDark) {
        // Dark primary: white → off-white on hover
        bgColor = _hovered ? const Color(0xFFEEEEEE) : const Color(0xFFFFFFFF);
        textColor = const Color(0xFF10110E);
        shadows = _hovered
            ? [BoxShadow(color: MyThemes.brandLime.withValues(alpha: 0.18), blurRadius: 14, offset: const Offset(0, 3))]
            : null;
      } else {
        // Light primary: brand purple
        bgColor = _hovered ? const Color(0xFF5C1FAE) : MyThemes.brandPurple;
        textColor = Colors.white;
        shadows = _hovered
            ? [BoxShadow(color: MyThemes.brandPurple.withValues(alpha: 0.30), blurRadius: 14, offset: const Offset(0, 4))]
            : null;
      }
      border = null;
    } else {
      bgColor = _hovered
          ? colorScheme.onSurface.withValues(alpha: 0.05)
          : Colors.transparent;
      textColor = colorScheme.onSurface;
      border = Border.all(
        color: _hovered
            ? colorScheme.onSurface.withValues(alpha: 0.4)
            : colorScheme.outline.withValues(alpha: 0.6),
      );
      shadows = null;
    }

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 160),
        decoration: BoxDecoration(
          color: bgColor,
          borderRadius: BorderRadius.circular(10),
          border: border,
          boxShadow: shadows,
        ),
        child: Material(
          color: Colors.transparent,
          child: InkWell(
            onTap: widget.onPressed,
            borderRadius: BorderRadius.circular(10),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 22, vertical: 13),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  if (widget.icon != null) ...[
                    IconTheme(
                      data: IconThemeData(color: textColor, size: 16),
                      child: widget.icon!,
                    ),
                    const SizedBox(width: 8),
                  ],
                  Text(
                    widget.label,
                    style: theme.textTheme.labelLarge?.copyWith(
                      color: textColor,
                      fontWeight: FontWeight.w600,
                      letterSpacing: 0.2,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
