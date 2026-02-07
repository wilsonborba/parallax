import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

import '../data/repositories/local_landing_repository.dart';
import '../domain/usecases/get_landing_data.dart';
import '../presentation/pages/landing_page.dart';
import '../presentation/viewmodels/landing_view_model.dart';

class ParallaxApp extends StatelessWidget {
  const ParallaxApp({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = ThemeData(
      brightness: Brightness.dark,
      scaffoldBackgroundColor: const Color(0xFF050507),
      textTheme: GoogleFonts.interTextTheme(ThemeData.dark().textTheme).apply(
        bodyColor: Colors.white,
        displayColor: Colors.white,
      ),
      colorScheme: const ColorScheme.dark(
        primary: Color(0xFFB86CFF),
        secondary: Color(0xFF7A4DFF),
        surface: Color(0xFF111116),
      ),
    );

    return MaterialApp(
      title: 'Parallax',
      debugShowCheckedModeBanner: false,
      theme: theme,
      home: LandingPage(
        viewModel: LandingViewModel(
          GetLandingData(LocalLandingRepository()),
        ),
      ),
    );
  }
}
