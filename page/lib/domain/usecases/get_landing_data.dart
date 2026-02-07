import '../entities/landing_data.dart';
import '../repositories/landing_repository.dart';

class GetLandingData {
  const GetLandingData(this.repository);

  final LandingRepository repository;

  LandingData call() => repository.fetchLandingData();
}
