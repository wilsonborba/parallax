import "dart:math";

import "package:flutter/foundation.dart";
import "package:flutter/material.dart";

class AnimatedBackground extends StatefulWidget {
  const AnimatedBackground({super.key, required this.scrollOffset});

  final ValueListenable<double> scrollOffset;

  @override
  State<AnimatedBackground> createState() => _AnimatedBackgroundState();
}

class _AnimatedBackgroundState extends State<AnimatedBackground>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;
  final List<_SpatialDust> _particles = List.generate(20, (index) => _SpatialDust(index));

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 14),
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;
    final bgColor = theme.scaffoldBackgroundColor;

    return AnimatedBuilder(
      animation: Listenable.merge([_controller, widget.scrollOffset]),
      builder: (context, _) {
        return CustomPaint(
          painter: _SpatialBackgroundPainter(
            progress: _controller.value,
            scrollOffset: widget.scrollOffset.value,
            particles: _particles,
            isDark: isDark,
            bgColor: bgColor,
          ),
          child: const SizedBox.expand(),
        );
      },
    );
  }
}

class _SpatialBackgroundPainter extends CustomPainter {
  _SpatialBackgroundPainter({
    required this.progress,
    required this.scrollOffset,
    required this.particles,
    required this.isDark,
    required this.bgColor,
  });

  final double progress;
  final double scrollOffset;
  final List<_SpatialDust> particles;
  final bool isDark;
  final Color bgColor;

  @override
  void paint(Canvas canvas, Size size) {
    final rect = Offset.zero & size;

    // 1. Fill base canvas
    final bgPaint = Paint()..color = bgColor;
    canvas.drawRect(rect, bgPaint);

    // 2. Spatial Studio Illumination (Top-center subtle glow that breathes)
    final spotlightCenter = Offset(
      size.width * 0.5,
      size.height * 0.15 - scrollOffset * 0.08,
    );

    final spotlightRadius = size.width * 0.65;
    final spotlightGradient = RadialGradient(
      center: Alignment.center,
      radius: 0.8,
      colors: isDark
          ? [
              const Color(0xFFFFFFFF).withValues(alpha: 0.035 + 0.015 * sin(progress * pi)),
              const Color(0xFF191A16).withValues(alpha: 0.01),
              Colors.transparent,
            ]
          : [
              const Color(0xFF181818).withValues(alpha: 0.025 + 0.01 * sin(progress * pi)),
              const Color(0xFFFFFFFF).withValues(alpha: 0.0),
              Colors.transparent,
            ],
      stops: const [0.0, 0.5, 1.0],
    );

    final spotlightPaint = Paint()
      ..shader = spotlightGradient.createShader(
        Rect.fromCircle(center: spotlightCenter, radius: spotlightRadius),
      );
    canvas.drawCircle(spotlightCenter, spotlightRadius, spotlightPaint);

    // 3. Ambient depth orb on right corner
    final rightOrbCenter = Offset(
      size.width * 0.85,
      size.height * 0.45 - scrollOffset * 0.05,
    );
    final rightOrbRadius = size.width * 0.4;
    final rightOrbGradient = RadialGradient(
      colors: isDark
          ? [
              const Color(0xFFD7FF3F).withValues(alpha: 0.012),
              Colors.transparent,
            ]
          : [
              const Color(0xFF181818).withValues(alpha: 0.015),
              Colors.transparent,
            ],
    );
    final rightOrbPaint = Paint()
      ..shader = rightOrbGradient.createShader(
        Rect.fromCircle(center: rightOrbCenter, radius: rightOrbRadius),
      );
    canvas.drawCircle(rightOrbCenter, rightOrbRadius, rightOrbPaint);

    // 4. Subtle spatial dust particles
    final particlePaint = Paint()
      ..color = (isDark ? Colors.white : Colors.black).withValues(alpha: isDark ? 0.06 : 0.04);

    for (final particle in particles) {
      final position = particle.position(size, progress, scrollOffset);
      canvas.drawCircle(position, particle.radius, particlePaint);
    }
  }

  @override
  bool shouldRepaint(covariant _SpatialBackgroundPainter oldDelegate) {
    return oldDelegate.progress != progress ||
        oldDelegate.scrollOffset != scrollOffset ||
        oldDelegate.isDark != isDark ||
        oldDelegate.bgColor != bgColor;
  }
}

class _SpatialDust {
  _SpatialDust(int seed)
      : _rng = Random(seed),
        radius = 0.8 + (seed % 3) * 0.5,
        speed = 0.12 + (seed % 4) * 0.04;

  final Random _rng;
  final double radius;
  final double speed;

  Offset position(Size size, double progress, double scrollOffset) {
    final dx = _rng.nextDouble() * size.width;
    final dy = _rng.nextDouble() * size.height;
    final float = sin(progress * pi * 2 + dx * 0.004) * 8 * speed;
    return Offset(dx, dy + float - scrollOffset * 0.02);
  }
}
