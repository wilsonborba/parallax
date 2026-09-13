import "package:flutter/material.dart";

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

    if (widget.isPrimary) {
      bgColor = isDark
          ? (_hovered ? const Color(0xFFEEEEEE) : const Color(0xFFFFFFFF))
          : (_hovered ? const Color(0xFF2A2A2A) : const Color(0xFF181818));
      textColor = isDark ? const Color(0xFF10110E) : const Color(0xFFFFFFFF);
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
    }

    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 180),
        decoration: BoxDecoration(
          color: bgColor,
          borderRadius: BorderRadius.circular(10),
          border: border,
          boxShadow: widget.isPrimary && _hovered
              ? [
                  BoxShadow(
                    color: (isDark ? Colors.white : Colors.black).withValues(alpha: 0.15),
                    blurRadius: 12,
                    offset: const Offset(0, 3),
                  ),
                ]
              : null,
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
                    widget.icon!,
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
