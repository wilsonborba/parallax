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
  final List<_Particle> _particles = List.generate(24, (index) => _Particle(index));

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 16),
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
          painter: _BackgroundPainter(
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

class _BackgroundPainter extends CustomPainter {
  _BackgroundPainter({
    required this.progress,
    required this.scrollOffset,
    required this.particles,
    required this.isDark,
    required this.bgColor,
  });

  final double progress;
  final double scrollOffset;
  final List<_Particle> particles;
  final bool isDark;
  final Color bgColor;

  @override
  void paint(Canvas canvas, Size size) {
    // Fill base background
    final rect = Offset.zero & size;
    final bgPaint = Paint()..color = bgColor;
    canvas.drawRect(rect, bgPaint);

    // Architectural grid (40px spacing)
    final gridPaint = Paint()
      ..color = (isDark ? Colors.white : Colors.black).withValues(alpha: isDark ? 0.025 : 0.035)
      ..strokeWidth = 1.0;

    const gridSize = 48.0;
    final startY = -(scrollOffset * 0.15) % gridSize;

    for (double x = 0; x < size.width; x += gridSize) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), gridPaint);
    }
    for (double y = startY; y < size.height; y += gridSize) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), gridPaint);
    }

    // Subtle ambient glow
    final glowPaint = Paint()
      ..color = (isDark ? Colors.white : Colors.black).withValues(alpha: isDark ? 0.015 : 0.02)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 80);

    canvas.drawCircle(
      Offset(size.width * 0.25, size.height * 0.2 + scrollOffset * 0.05),
      260,
      glowPaint,
    );
    canvas.drawCircle(
      Offset(size.width * 0.75, size.height * 0.4 - scrollOffset * 0.04),
      200,
      glowPaint,
    );

    // Minimal floating particles
    final particlePaint = Paint()
      ..color = (isDark ? Colors.white : Colors.black).withValues(alpha: isDark ? 0.08 : 0.06);

    for (final particle in particles) {
      final position = particle.position(size, progress, scrollOffset);
      canvas.drawCircle(position, particle.radius, particlePaint);
    }
  }

  @override
  bool shouldRepaint(covariant _BackgroundPainter oldDelegate) {
    return oldDelegate.progress != progress ||
        oldDelegate.scrollOffset != scrollOffset ||
        oldDelegate.isDark != isDark ||
        oldDelegate.bgColor != bgColor;
  }
}

class _Particle {
  _Particle(int seed)
      : _rng = Random(seed),
        radius = 1.0 + (seed % 3) * 0.6,
        speed = 0.15 + (seed % 5) * 0.04;

  final Random _rng;
  final double radius;
  final double speed;

  Offset position(Size size, double progress, double scrollOffset) {
    final dx = _rng.nextDouble() * size.width;
    final dy = _rng.nextDouble() * size.height;
    final float = sin(progress * pi * 2 + dx * 0.005) * 10 * speed;
    return Offset(dx, dy + float - scrollOffset * 0.03);
  }
}
