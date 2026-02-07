import 'package:flutter/material.dart';

import '../../domain/entities/landing_data.dart';
import '../../domain/usecases/get_landing_data.dart';

class LandingViewModel extends ChangeNotifier {
  LandingViewModel(this._getLandingData) {
    _landingData = _getLandingData();
  }

  final GetLandingData _getLandingData;
  late final LandingData _landingData;

  LandingData get landingData => _landingData;
}
