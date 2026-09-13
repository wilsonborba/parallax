import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_pt.dart';
import 'app_localizations_th.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations? of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations);
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('pt'),
    Locale('pt', 'BR'),
    Locale('th'),
  ];

  /// No description provided for @navBrand.
  ///
  /// In en, this message translates to:
  /// **'PARALLAX'**
  String get navBrand;

  /// No description provided for @navBadge.
  ///
  /// In en, this message translates to:
  /// **'EXPERIMENTAL v0.1'**
  String get navBadge;

  /// No description provided for @navArchitecture.
  ///
  /// In en, this message translates to:
  /// **'Architecture'**
  String get navArchitecture;

  /// No description provided for @navFeatures.
  ///
  /// In en, this message translates to:
  /// **'Features'**
  String get navFeatures;

  /// No description provided for @navHowItWorks.
  ///
  /// In en, this message translates to:
  /// **'How It Works'**
  String get navHowItWorks;

  /// No description provided for @navGetStarted.
  ///
  /// In en, this message translates to:
  /// **'Get Started'**
  String get navGetStarted;

  /// No description provided for @navProtocol.
  ///
  /// In en, this message translates to:
  /// **'Protocol'**
  String get navProtocol;

  /// No description provided for @navRoadmap.
  ///
  /// In en, this message translates to:
  /// **'Roadmap'**
  String get navRoadmap;

  /// No description provided for @navThemeLight.
  ///
  /// In en, this message translates to:
  /// **'Light Mode'**
  String get navThemeLight;

  /// No description provided for @navThemeDark.
  ///
  /// In en, this message translates to:
  /// **'Dark Mode'**
  String get navThemeDark;

  /// No description provided for @navLanguage.
  ///
  /// In en, this message translates to:
  /// **'Language'**
  String get navLanguage;

  /// No description provided for @navGithub.
  ///
  /// In en, this message translates to:
  /// **'GitHub'**
  String get navGithub;

  /// No description provided for @heroHeadline.
  ///
  /// In en, this message translates to:
  /// **'Stream your Linux desktop into VR with ultra-low latency.'**
  String get heroHeadline;

  /// No description provided for @heroSubHeadline.
  ///
  /// In en, this message translates to:
  /// **'Open-source. Experimental. Built for the future of spatial computing.'**
  String get heroSubHeadline;

  /// No description provided for @heroPrimaryCta.
  ///
  /// In en, this message translates to:
  /// **'Get Started'**
  String get heroPrimaryCta;

  /// No description provided for @heroSecondaryCta.
  ///
  /// In en, this message translates to:
  /// **'View on GitHub'**
  String get heroSecondaryCta;

  /// No description provided for @heroMicroText.
  ///
  /// In en, this message translates to:
  /// **'Currently in early development (v0.1).'**
  String get heroMicroText;

  /// No description provided for @aboutTitle.
  ///
  /// In en, this message translates to:
  /// **'What is Parallax?'**
  String get aboutTitle;

  /// No description provided for @aboutDescription.
  ///
  /// In en, this message translates to:
  /// **'Parallax is an experimental remote-rendering pipeline for streaming a Linux desktop into a client device with a focus on VR/AR use cases. It captures an X11 display, encodes it as H.264, transmits it over UDP, and coordinates sessions over a lightweight TCP control channel.'**
  String get aboutDescription;

  /// No description provided for @aboutDetail1.
  ///
  /// In en, this message translates to:
  /// **'Linux host captures X11 display for real-time VR streaming.'**
  String get aboutDetail1;

  /// No description provided for @aboutDetail2.
  ///
  /// In en, this message translates to:
  /// **'Frames are encoded to H.264 using hardware (VAAPI) or software fallback.'**
  String get aboutDetail2;

  /// No description provided for @aboutDetail3.
  ///
  /// In en, this message translates to:
  /// **'Video frames stream over UDP for ultra-low latency delivery.'**
  String get aboutDetail3;

  /// No description provided for @aboutDetail4.
  ///
  /// In en, this message translates to:
  /// **'TCP control channel manages pairing tokens and session setup.'**
  String get aboutDetail4;

  /// No description provided for @aboutDetail5.
  ///
  /// In en, this message translates to:
  /// **'Early-stage and evolving — community contributions are welcome.'**
  String get aboutDetail5;

  /// No description provided for @architectureTitle.
  ///
  /// In en, this message translates to:
  /// **'Architecture Overview'**
  String get architectureTitle;

  /// No description provided for @architectureSubtitle.
  ///
  /// In en, this message translates to:
  /// **'A minimal, low-latency pipeline between a Linux host daemon and an Android client with bi-directional control and one-way video streaming.'**
  String get architectureSubtitle;

  /// No description provided for @archHostTitle.
  ///
  /// In en, this message translates to:
  /// **'Linux Host (Rust)'**
  String get archHostTitle;

  /// No description provided for @archHostSubtitle.
  ///
  /// In en, this message translates to:
  /// **'prlx-hostd'**
  String get archHostSubtitle;

  /// No description provided for @archHostTooltip.
  ///
  /// In en, this message translates to:
  /// **'Captures X11 frames, encodes H.264, and serves control sessions.'**
  String get archHostTooltip;

  /// No description provided for @archX11Title.
  ///
  /// In en, this message translates to:
  /// **'X11 Capture'**
  String get archX11Title;

  /// No description provided for @archX11Subtitle.
  ///
  /// In en, this message translates to:
  /// **'Display feed'**
  String get archX11Subtitle;

  /// No description provided for @archX11Tooltip.
  ///
  /// In en, this message translates to:
  /// **'Grabs Linux desktop frames directly from the X11 display server.'**
  String get archX11Tooltip;

  /// No description provided for @archEncodeTitle.
  ///
  /// In en, this message translates to:
  /// **'H.264 Encode'**
  String get archEncodeTitle;

  /// No description provided for @archEncodeSubtitle.
  ///
  /// In en, this message translates to:
  /// **'VAAPI or software'**
  String get archEncodeSubtitle;

  /// No description provided for @archEncodeTooltip.
  ///
  /// In en, this message translates to:
  /// **'Encodes frames into H.264 Annex B byte streams.'**
  String get archEncodeTooltip;

  /// No description provided for @archUdpStreamTitle.
  ///
  /// In en, this message translates to:
  /// **'UDP Video Stream'**
  String get archUdpStreamTitle;

  /// No description provided for @archUdpStreamSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Minimal framing'**
  String get archUdpStreamSubtitle;

  /// No description provided for @archUdpStreamTooltip.
  ///
  /// In en, this message translates to:
  /// **'Sends UDP packets with a lightweight 24-byte header.'**
  String get archUdpStreamTooltip;

  /// No description provided for @archAndroidTitle.
  ///
  /// In en, this message translates to:
  /// **'Android Client'**
  String get archAndroidTitle;

  /// No description provided for @archAndroidSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Parallax Receiver'**
  String get archAndroidSubtitle;

  /// No description provided for @archAndroidTooltip.
  ///
  /// In en, this message translates to:
  /// **'Jetpack Compose app for VR/AR streaming.'**
  String get archAndroidTooltip;

  /// No description provided for @archQrTitle.
  ///
  /// In en, this message translates to:
  /// **'QR Pairing'**
  String get archQrTitle;

  /// No description provided for @archQrSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Token workflow'**
  String get archQrSubtitle;

  /// No description provided for @archQrTooltip.
  ///
  /// In en, this message translates to:
  /// **'Scans QR to join the control channel securely.'**
  String get archQrTooltip;

  /// No description provided for @archTcpTitle.
  ///
  /// In en, this message translates to:
  /// **'TCP Control'**
  String get archTcpTitle;

  /// No description provided for @archTcpSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Session broker'**
  String get archTcpSubtitle;

  /// No description provided for @archTcpTooltip.
  ///
  /// In en, this message translates to:
  /// **'Coordinates session setup, pairing, and configuration.'**
  String get archTcpTooltip;

  /// No description provided for @archDecodeTitle.
  ///
  /// In en, this message translates to:
  /// **'H.264 Decode'**
  String get archDecodeTitle;

  /// No description provided for @archDecodeSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Stream UI'**
  String get archDecodeSubtitle;

  /// No description provided for @archDecodeTooltip.
  ///
  /// In en, this message translates to:
  /// **'Decodes H.264 frames for immersive rendering.'**
  String get archDecodeTooltip;

  /// No description provided for @archFlowTcp.
  ///
  /// In en, this message translates to:
  /// **'TCP control (bi-directional)'**
  String get archFlowTcp;

  /// No description provided for @archFlowUdp.
  ///
  /// In en, this message translates to:
  /// **'UDP video stream (host → client)'**
  String get archFlowUdp;

  /// No description provided for @featuresTitle.
  ///
  /// In en, this message translates to:
  /// **'Core features'**
  String get featuresTitle;

  /// No description provided for @featuresSubtitle.
  ///
  /// In en, this message translates to:
  /// **'High-performance streaming primitives designed for VR latency budgets.'**
  String get featuresSubtitle;

  /// No description provided for @featUdpTitle.
  ///
  /// In en, this message translates to:
  /// **'Low-latency UDP streaming'**
  String get featUdpTitle;

  /// No description provided for @featUdpDesc.
  ///
  /// In en, this message translates to:
  /// **'Minimal framing protocol for speed and resilience.'**
  String get featUdpDesc;

  /// No description provided for @featPairingTitle.
  ///
  /// In en, this message translates to:
  /// **'Secure pairing'**
  String get featPairingTitle;

  /// No description provided for @featPairingDesc.
  ///
  /// In en, this message translates to:
  /// **'Control channel uses pairing tokens and QR workflow.'**
  String get featPairingDesc;

  /// No description provided for @featProtocolTitle.
  ///
  /// In en, this message translates to:
  /// **'Open protocol'**
  String get featProtocolTitle;

  /// No description provided for @featProtocolDesc.
  ///
  /// In en, this message translates to:
  /// **'UDP packet structure documented and extensible.'**
  String get featProtocolDesc;

  /// No description provided for @featHostUiTitle.
  ///
  /// In en, this message translates to:
  /// **'Host UI'**
  String get featHostUiTitle;

  /// No description provided for @featHostUiDesc.
  ///
  /// In en, this message translates to:
  /// **'Desktop UI for pairing and session control.'**
  String get featHostUiDesc;

  /// No description provided for @featAndroidClientTitle.
  ///
  /// In en, this message translates to:
  /// **'Android client'**
  String get featAndroidClientTitle;

  /// No description provided for @featAndroidClientDesc.
  ///
  /// In en, this message translates to:
  /// **'Jetpack Compose app that scans QR to connect.'**
  String get featAndroidClientDesc;

  /// No description provided for @howItWorksTitle.
  ///
  /// In en, this message translates to:
  /// **'How it works'**
  String get howItWorksTitle;

  /// No description provided for @howItWorksSubtitle.
  ///
  /// In en, this message translates to:
  /// **'From desktop display to VR headset in five synchronized pipeline steps.'**
  String get howItWorksSubtitle;

  /// No description provided for @step1Title.
  ///
  /// In en, this message translates to:
  /// **'Capture the X11 display'**
  String get step1Title;

  /// No description provided for @step1Desc.
  ///
  /// In en, this message translates to:
  /// **'prlx-hostd grabs the Linux desktop frames directly from X11.'**
  String get step1Desc;

  /// No description provided for @step2Title.
  ///
  /// In en, this message translates to:
  /// **'Encode with H.264'**
  String get step2Title;

  /// No description provided for @step2Desc.
  ///
  /// In en, this message translates to:
  /// **'Hardware VAAPI encoding or software fallback for portability.'**
  String get step2Desc;

  /// No description provided for @step3Title.
  ///
  /// In en, this message translates to:
  /// **'Packetize via Parallax framing'**
  String get step3Title;

  /// No description provided for @step3Desc.
  ///
  /// In en, this message translates to:
  /// **'Frames split into UDP packets with stream/frame metadata.'**
  String get step3Desc;

  /// No description provided for @step4Title.
  ///
  /// In en, this message translates to:
  /// **'Coordinate over TCP'**
  String get step4Title;

  /// No description provided for @step4Desc.
  ///
  /// In en, this message translates to:
  /// **'Pairing token and session control flows over TCP.'**
  String get step4Desc;

  /// No description provided for @step5Title.
  ///
  /// In en, this message translates to:
  /// **'Android client decodes'**
  String get step5Title;

  /// No description provided for @step5Desc.
  ///
  /// In en, this message translates to:
  /// **'Jetpack Compose client scans QR and renders the stream.'**
  String get step5Desc;

  /// No description provided for @gettingStartedTitle.
  ///
  /// In en, this message translates to:
  /// **'Getting started'**
  String get gettingStartedTitle;

  /// No description provided for @gettingStartedSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Choose an installation method to deploy Parallax on your Linux host.'**
  String get gettingStartedSubtitle;

  /// No description provided for @sample1Title.
  ///
  /// In en, this message translates to:
  /// **'One-command install (Debian/Ubuntu)'**
  String get sample1Title;

  /// No description provided for @sample1Caption.
  ///
  /// In en, this message translates to:
  /// **'Installs dependencies, binaries, CLI command, and desktop launcher.'**
  String get sample1Caption;

  /// No description provided for @sample2Title.
  ///
  /// In en, this message translates to:
  /// **'Cloudflare Pages installer URL'**
  String get sample2Title;

  /// No description provided for @sample2Caption.
  ///
  /// In en, this message translates to:
  /// **'Installer published as a Flutter web asset from page/install.sh.'**
  String get sample2Caption;

  /// No description provided for @sample3Title.
  ///
  /// In en, this message translates to:
  /// **'Cargo install flow'**
  String get sample3Title;

  /// No description provided for @sample3Caption.
  ///
  /// In en, this message translates to:
  /// **'Alternative for users who prefer a cargo-based flow.'**
  String get sample3Caption;

  /// No description provided for @sample4Title.
  ///
  /// In en, this message translates to:
  /// **'Repository install flow'**
  String get sample4Title;

  /// No description provided for @sample4Caption.
  ///
  /// In en, this message translates to:
  /// **'Manual alternative for users who want full step-by-step control.'**
  String get sample4Caption;

  /// No description provided for @copyCommand.
  ///
  /// In en, this message translates to:
  /// **'Copy command'**
  String get copyCommand;

  /// No description provided for @commandCopied.
  ///
  /// In en, this message translates to:
  /// **'Copied to clipboard'**
  String get commandCopied;

  /// No description provided for @protocolTitle.
  ///
  /// In en, this message translates to:
  /// **'Protocol: UDP packet framing'**
  String get protocolTitle;

  /// No description provided for @protocolSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Parallax uses a fixed 24-byte header with a magic \"PRLX\" value, stream/frame identifiers, and flags for keyframes, config, and end-of-frame markers.'**
  String get protocolSubtitle;

  /// No description provided for @protocolHighlight1.
  ///
  /// In en, this message translates to:
  /// **'Fixed 24-byte header; big-endian network order.'**
  String get protocolHighlight1;

  /// No description provided for @protocolHighlight2.
  ///
  /// In en, this message translates to:
  /// **'Magic value \"PRLX\" validates framing.'**
  String get protocolHighlight2;

  /// No description provided for @protocolHighlight3.
  ///
  /// In en, this message translates to:
  /// **'Stream ID + Frame ID for reassembly and timing.'**
  String get protocolHighlight3;

  /// No description provided for @protocolHighlight4.
  ///
  /// In en, this message translates to:
  /// **'Flags for keyframe, config, and end-of-frame markers.'**
  String get protocolHighlight4;

  /// No description provided for @protocolHighlight5.
  ///
  /// In en, this message translates to:
  /// **'Payload types for video, audio, and control data.'**
  String get protocolHighlight5;

  /// No description provided for @protocolPayloadNotesTitle.
  ///
  /// In en, this message translates to:
  /// **'Payload notes'**
  String get protocolPayloadNotesTitle;

  /// No description provided for @protocolNote1.
  ///
  /// In en, this message translates to:
  /// **'H.264 payloads are Annex B byte streams (start-code delimited).'**
  String get protocolNote1;

  /// No description provided for @protocolNote2.
  ///
  /// In en, this message translates to:
  /// **'SPS/PPS can be inline with keyframes or config packets.'**
  String get protocolNote2;

  /// No description provided for @roadmapTitle.
  ///
  /// In en, this message translates to:
  /// **'Project status & roadmap'**
  String get roadmapTitle;

  /// No description provided for @roadmapSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Early experimental release (v0.1). Actively evolving; stability and latency work ongoing.'**
  String get roadmapSubtitle;

  /// No description provided for @roadmapStatus1.
  ///
  /// In en, this message translates to:
  /// **'Early experimental release (v0.1).'**
  String get roadmapStatus1;

  /// No description provided for @roadmapStatus2.
  ///
  /// In en, this message translates to:
  /// **'Actively evolving; stability and latency work ongoing.'**
  String get roadmapStatus2;

  /// No description provided for @roadmapStatus3.
  ///
  /// In en, this message translates to:
  /// **'Contributions welcome from the community.'**
  String get roadmapStatus3;

  /// No description provided for @roadmapItem1Title.
  ///
  /// In en, this message translates to:
  /// **'Optimize latency'**
  String get roadmapItem1Title;

  /// No description provided for @roadmapItem1Desc.
  ///
  /// In en, this message translates to:
  /// **'Tighten capture → encode → transport pipeline timings.'**
  String get roadmapItem1Desc;

  /// No description provided for @roadmapItem2Title.
  ///
  /// In en, this message translates to:
  /// **'Improve hardware encoder support'**
  String get roadmapItem2Title;

  /// No description provided for @roadmapItem2Desc.
  ///
  /// In en, this message translates to:
  /// **'Broaden VAAPI/codec paths and smarter fallbacks.'**
  String get roadmapItem2Desc;

  /// No description provided for @roadmapItem3Title.
  ///
  /// In en, this message translates to:
  /// **'Broaden client platforms'**
  String get roadmapItem3Title;

  /// No description provided for @roadmapItem3Desc.
  ///
  /// In en, this message translates to:
  /// **'Explore desktop and additional mobile clients.'**
  String get roadmapItem3Desc;

  /// No description provided for @roadmapItem4Title.
  ///
  /// In en, this message translates to:
  /// **'VR-native rendering'**
  String get roadmapItem4Title;

  /// No description provided for @roadmapItem4Desc.
  ///
  /// In en, this message translates to:
  /// **'Integrate immersive VR playback and input.'**
  String get roadmapItem4Desc;

  /// No description provided for @communityTitle.
  ///
  /// In en, this message translates to:
  /// **'Open-source & community'**
  String get communityTitle;

  /// No description provided for @communityDescription.
  ///
  /// In en, this message translates to:
  /// **'Parallax is MIT licensed and focused on research + experimentation. GitHub contributions and feedback are encouraged.'**
  String get communityDescription;

  /// No description provided for @communityItem1.
  ///
  /// In en, this message translates to:
  /// **'MIT License'**
  String get communityItem1;

  /// No description provided for @communityItem2.
  ///
  /// In en, this message translates to:
  /// **'Open protocol documentation'**
  String get communityItem2;

  /// No description provided for @communityItem3.
  ///
  /// In en, this message translates to:
  /// **'Active GitHub issues & discussions'**
  String get communityItem3;

  /// No description provided for @finalCtaHeadline.
  ///
  /// In en, this message translates to:
  /// **'Build the future of VR streaming.'**
  String get finalCtaHeadline;

  /// No description provided for @finalCtaSubHeadline.
  ///
  /// In en, this message translates to:
  /// **'Parallax is open-source and evolving — join the community.'**
  String get finalCtaSubHeadline;

  /// No description provided for @footerCopyright.
  ///
  /// In en, this message translates to:
  /// **'© 2026 Asodya. All rights reserved.'**
  String get footerCopyright;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'pt', 'th'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when language+country codes are specified.
  switch (locale.languageCode) {
    case 'pt':
      {
        switch (locale.countryCode) {
          case 'BR':
            return AppLocalizationsPtBr();
        }
        break;
      }
  }

  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'pt':
      return AppLocalizationsPt();
    case 'th':
      return AppLocalizationsTh();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
