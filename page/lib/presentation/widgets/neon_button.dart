import 'package:flutter/material.dart';

class NeonButton extends StatefulWidget {
  const NeonButton({
    super.key,
    required this.label,
    required this.onPressed,
    this.isPrimary = true,
  });

  final String label;
  final VoidCallback onPressed;
  final bool isPrimary;

  @override
  State<NeonButton> createState() => _NeonButtonState();
}

class _NeonButtonState extends State<NeonButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final baseColor = widget.isPrimary ? colorScheme.primary : Colors.white;

    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 250),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(32),
          gradient: widget.isPrimary
              ? LinearGradient(
                  colors: [
                    baseColor,
                    colorScheme.secondary.withOpacity(0.8),
                  ],
                )
              : null,
          border: Border.all(
            color: widget.isPrimary ? Colors.transparent : Colors.white24,
          ),
          boxShadow: _hovered
              ? [
                  BoxShadow(
                    color: baseColor.withOpacity(0.4),
                    blurRadius: 24,
                    spreadRadius: 2,
                  ),
                ]
              : [
                  BoxShadow(
                    color: baseColor.withOpacity(0.2),
                    blurRadius: 16,
                  ),
                ],
        ),
        child: Material(
          color: Colors.transparent,
          child: InkWell(
            onTap: widget.onPressed,
            borderRadius: BorderRadius.circular(32),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 26, vertical: 14),
              child: Text(
                widget.label,
                style: Theme.of(context).textTheme.labelLarge?.copyWith(
                      color: widget.isPrimary ? Colors.black : Colors.white,
                      fontWeight: FontWeight.w600,
                      letterSpacing: 0.3,
                    ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
