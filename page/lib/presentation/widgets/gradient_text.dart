import "package:flutter/material.dart";

class GradientText extends StatelessWidget {
  const GradientText(
    this.text, {
    super.key,
    this.gradient,
    this.style,
  });

  final String text;
  final Gradient? gradient;
  final TextStyle? style;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;

    final grad = gradient ??
        LinearGradient(
          colors: isDark
              ? const [Color(0xFFFFFFFF), Color(0xFFCCCCCC)]
              : const [Color(0xFF101010), Color(0xFF333333)],
        );

    return ShaderMask(
      shaderCallback: (bounds) => grad.createShader(bounds),
      child: Text(
        text,
        style: (style ?? theme.textTheme.displaySmall)?.copyWith(
          color: Colors.white,
        ),
      ),
    );
  }
}
