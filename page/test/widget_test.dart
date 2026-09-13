import "package:flutter_test/flutter_test.dart";
import "package:page/app/app.dart";
import "package:page/core/state/app_scope.dart";
import "package:shared_preferences/shared_preferences.dart";

void main() {
  testWidgets("ParallaxApp renders smoke test", (WidgetTester tester) async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final controller = AppController(prefs);

    await tester.pumpWidget(ParallaxApp(controller: controller));
    await tester.pump(const Duration(milliseconds: 200));

    expect(find.text("PARALLAX"), findsWidgets);
  });
}
