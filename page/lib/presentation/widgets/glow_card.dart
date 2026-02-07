import 'package:flutter/material.dart';

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
    final colorScheme = Theme.of(context).colorScheme;
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 250),
        padding: widget.padding ?? const EdgeInsets.all(20),
        decoration: BoxDecoration(
          color: const Color(0xFF0E0E15),
          borderRadius: BorderRadius.circular(24),
          border: Border.all(
            color: _hovered ? colorScheme.primary.withOpacity(0.6) : Colors.white10,
          ),
          boxShadow: [
            BoxShadow(
              color: colorScheme.primary.withOpacity(_hovered ? 0.35 : 0.15),
              blurRadius: _hovered ? 28 : 18,
              offset: const Offset(0, 12),
            ),
          ],
        ),
        transform: Matrix4.identity()..translate(0, _hovered ? -6.0 : 0.0),
        child: Material(
          color: Colors.transparent,
          child: InkWell(
            onTap: widget.onTap,
            borderRadius: BorderRadius.circular(24),
            child: widget.child,
          ),
        ),
      ),
    );
  }
}
