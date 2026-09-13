import "package:flutter/material.dart";
import "package:shared_preferences/shared_preferences.dart";

class AppController extends ChangeNotifier {
  AppController(this._prefs) {
    _loadPreferences();
  }

  final SharedPreferences _prefs;

  ThemeMode _themeMode = ThemeMode.dark;
  Locale _locale = const Locale("en");

  ThemeMode get themeMode => _themeMode;
  Locale get locale => _locale;

  static const _kThemeModeKey = "parallax_theme_mode";
  static const _kLocaleKey = "parallax_locale";

  void _loadPreferences() {
    final themeStr = _prefs.getString(_kThemeModeKey);
    if (themeStr == "light") {
      _themeMode = ThemeMode.light;
    } else if (themeStr == "dark") {
      _themeMode = ThemeMode.dark;
    }

    final localeStr = _prefs.getString(_kLocaleKey);
    if (localeStr != null && localeStr.isNotEmpty) {
      if (localeStr.startsWith("pt")) {
        _locale = const Locale("pt", "BR");
      } else if (localeStr.startsWith("th")) {
        _locale = const Locale("th");
      } else {
        _locale = const Locale("en");
      }
    }
  }

  Future<void> setThemeMode(ThemeMode mode) async {
    _themeMode = mode;
    notifyListeners();
    await _prefs.setString(_kThemeModeKey, mode == ThemeMode.light ? "light" : "dark");
  }

  Future<void> toggleThemeMode() async {
    if (_themeMode == ThemeMode.light) {
      await setThemeMode(ThemeMode.dark);
    } else {
      await setThemeMode(ThemeMode.light);
    }
  }

  Future<void> setLocale(Locale newLocale) async {
    _locale = newLocale;
    notifyListeners();
    final str = newLocale.countryCode != null
        ? "${newLocale.languageCode}_${newLocale.countryCode}"
        : newLocale.languageCode;
    await _prefs.setString(_kLocaleKey, str);
  }
}

class AppScope extends InheritedNotifier<AppController> {
  const AppScope({
    super.key,
    required AppController controller,
    required super.child,
  }) : super(notifier: controller);

  static AppController of(BuildContext context) {
    final scope = context.dependOnInheritedWidgetOfExactType<AppScope>();
    assert(scope != null, "No AppScope found in context");
    return scope!.notifier!;
  }
}
