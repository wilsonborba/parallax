import "package:flutter/material.dart";
import "package:shared_preferences/shared_preferences.dart";

import "app/app.dart";
import "core/state/app_scope.dart";

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final prefs = await SharedPreferences.getInstance();
  final controller = AppController(prefs);
  runApp(ParallaxApp(controller: controller));
}
