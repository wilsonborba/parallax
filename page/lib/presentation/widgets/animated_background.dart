import 'dart:math';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

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
      duration: const Duration(seconds: 12),
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: Listenable.merge([_controller, widget.scrollOffset]),
      builder: (context, _) {
        return CustomPaint(
          painter: _BackgroundPainter(
            progress: _controller.value,
            scrollOffset: widget.scrollOffset.value,
            particles: _particles,
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
  });

  final double progress;
  final double scrollOffset;
  final List<_Particle> particles;

  @override
  void paint(Canvas canvas, Size size) {
    final gradient = LinearGradient(
      begin: Alignment.topLeft,
      end: Alignment.bottomRight,
      colors: [
        const Color(0xFF2B0A3D),
        const Color(0xFF09030F),
        const Color(0xFF000000),
      ],
      stops: [0.0, 0.5 + 0.2 * sin(progress * pi), 1.0],
    );

    final rect = Offset.zero & size;
    final paint = Paint()..shader = gradient.createShader(rect);
    canvas.drawRect(rect, paint);

    final glowPaint = Paint()
      ..color = const Color(0xFFB86CFF).withOpacity(0.12)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 64);
    canvas.drawCircle(
      Offset(size.width * 0.2, size.height * 0.2 + scrollOffset * 0.08),
      220,
      glowPaint,
    );
    canvas.drawCircle(
      Offset(size.width * 0.8, size.height * 0.3 - scrollOffset * 0.05),
      160,
      glowPaint,
    );

    final networkPaint = Paint()
      ..color = Colors.white.withOpacity(0.06)
      ..strokeWidth = 1.2;

    for (var i = 0; i < 5; i++) {
      final offset = sin(progress * pi * 2 + i) * 24;
      final y = size.height * 0.2 + i * 80 + offset;
      final path = Path()
        ..moveTo(0, y)
        ..quadraticBezierTo(size.width * 0.4, y - 40, size.width * 0.8, y)
        ..quadraticBezierTo(size.width, y + 40, size.width, y + 80);
      canvas.drawPath(path, networkPaint);
    }

    final particlePaint = Paint()..color = const Color(0xFFB86CFF).withOpacity(0.25);
    for (final particle in particles) {
      final position = particle.position(size, progress, scrollOffset);
      canvas.drawCircle(position, particle.radius, particlePaint);
    }
  }

  @override
  bool shouldRepaint(covariant _BackgroundPainter oldDelegate) {
    return oldDelegate.progress != progress || oldDelegate.scrollOffset != scrollOffset;
  }
}

class _Particle {
  _Particle(int seed)
      : _rng = Random(seed),
        radius = 1.5 + (seed % 3) * 0.8,
        speed = 0.2 + (seed % 5) * 0.05;

  final Random _rng;
  final double radius;
  final double speed;

  Offset position(Size size, double progress, double scrollOffset) {
    final dx = _rng.nextDouble() * size.width;
    final dy = _rng.nextDouble() * size.height;
    final float = sin(progress * pi * 2 + dx * 0.005) * 14 * speed;
    return Offset(dx, dy + float - scrollOffset * 0.04);
  }
}
