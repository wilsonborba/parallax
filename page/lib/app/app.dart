import "package:flutter/material.dart";

import "../core/state/app_scope.dart";
import "../core/theme/my_themes.dart";
import "../data/repositories/local_landing_repository.dart";
import "../domain/usecases/get_landing_data.dart";
import "../l10n/app_localizations.dart";
import "../presentation/pages/landing_page.dart";
import "../presentation/viewmodels/landing_view_model.dart";

class ParallaxApp extends StatelessWidget {
  const ParallaxApp({super.key, required this.controller});

  final AppController controller;

  @override
  Widget build(BuildContext context) {
    return AppScope(
      controller: controller,
      child: ListenableBuilder(
        listenable: controller,
        builder: (context, _) {
          return MaterialApp(
            title: "Parallax",
            debugShowCheckedModeBanner: false,
            theme: MyThemes.light(),
            darkTheme: MyThemes.dark(),
            themeMode: controller.themeMode,
            locale: controller.locale,
            localizationsDelegates: AppLocalizations.localizationsDelegates,
            supportedLocales: AppLocalizations.supportedLocales,
            home: LandingPage(
              viewModel: LandingViewModel(
                GetLandingData(LocalLandingRepository()),
              ),
            ),
          );
        },
      ),
    );
  }
}
