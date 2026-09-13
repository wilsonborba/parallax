// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get navBrand => 'PARALLAX';

  @override
  String get navBadge => 'EXPERIMENTAL v0.1';

  @override
  String get navArchitecture => 'Architecture';

  @override
  String get navFeatures => 'Features';

  @override
  String get navHowItWorks => 'How It Works';

  @override
  String get navGetStarted => 'Get Started';

  @override
  String get navProtocol => 'Protocol';

  @override
  String get navRoadmap => 'Roadmap';

  @override
  String get navThemeLight => 'Light Mode';

  @override
  String get navThemeDark => 'Dark Mode';

  @override
  String get navLanguage => 'Language';

  @override
  String get navGithub => 'GitHub';

  @override
  String get heroHeadline =>
      'Stream your Linux desktop into VR with ultra-low latency.';

  @override
  String get heroSubHeadline =>
      'Open-source. Experimental. Built for the future of spatial computing.';

  @override
  String get heroPrimaryCta => 'Get Started';

  @override
  String get heroSecondaryCta => 'View on GitHub';

  @override
  String get heroMicroText => 'Currently in early development (v0.1).';

  @override
  String get aboutTitle => 'What is Parallax?';

  @override
  String get aboutDescription =>
      'Parallax is an experimental remote-rendering pipeline for streaming a Linux desktop into a client device with a focus on VR/AR use cases. It captures an X11 display, encodes it as H.264, transmits it over UDP, and coordinates sessions over a lightweight TCP control channel.';

  @override
  String get aboutDetail1 =>
      'Linux host captures X11 display for real-time VR streaming.';

  @override
  String get aboutDetail2 =>
      'Frames are encoded to H.264 using hardware (VAAPI) or software fallback.';

  @override
  String get aboutDetail3 =>
      'Video frames stream over UDP for ultra-low latency delivery.';

  @override
  String get aboutDetail4 =>
      'TCP control channel manages pairing tokens and session setup.';

  @override
  String get aboutDetail5 =>
      'Early-stage and evolving — community contributions are welcome.';

  @override
  String get architectureTitle => 'Architecture Overview';

  @override
  String get architectureSubtitle =>
      'A minimal, low-latency pipeline between a Linux host daemon and an Android client with bi-directional control and one-way video streaming.';

  @override
  String get archHostTitle => 'Linux Host (Rust)';

  @override
  String get archHostSubtitle => 'prlx-hostd';

  @override
  String get archHostTooltip =>
      'Captures X11 frames, encodes H.264, and serves control sessions.';

  @override
  String get archX11Title => 'X11 Capture';

  @override
  String get archX11Subtitle => 'Display feed';

  @override
  String get archX11Tooltip =>
      'Grabs Linux desktop frames directly from the X11 display server.';

  @override
  String get archEncodeTitle => 'H.264 Encode';

  @override
  String get archEncodeSubtitle => 'VAAPI or software';

  @override
  String get archEncodeTooltip =>
      'Encodes frames into H.264 Annex B byte streams.';

  @override
  String get archUdpStreamTitle => 'UDP Video Stream';

  @override
  String get archUdpStreamSubtitle => 'Minimal framing';

  @override
  String get archUdpStreamTooltip =>
      'Sends UDP packets with a lightweight 24-byte header.';

  @override
  String get archAndroidTitle => 'Android Client';

  @override
  String get archAndroidSubtitle => 'Parallax Receiver';

  @override
  String get archAndroidTooltip => 'Jetpack Compose app for VR/AR streaming.';

  @override
  String get archQrTitle => 'QR Pairing';

  @override
  String get archQrSubtitle => 'Token workflow';

  @override
  String get archQrTooltip => 'Scans QR to join the control channel securely.';

  @override
  String get archTcpTitle => 'TCP Control';

  @override
  String get archTcpSubtitle => 'Session broker';

  @override
  String get archTcpTooltip =>
      'Coordinates session setup, pairing, and configuration.';

  @override
  String get archDecodeTitle => 'H.264 Decode';

  @override
  String get archDecodeSubtitle => 'Stream UI';

  @override
  String get archDecodeTooltip =>
      'Decodes H.264 frames for immersive rendering.';

  @override
  String get archFlowTcp => 'TCP control (bi-directional)';

  @override
  String get archFlowUdp => 'UDP video stream (host → client)';

  @override
  String get featuresTitle => 'Core features';

  @override
  String get featuresSubtitle =>
      'High-performance streaming primitives designed for VR latency budgets.';

  @override
  String get featUdpTitle => 'Low-latency UDP streaming';

  @override
  String get featUdpDesc =>
      'Minimal framing protocol for speed and resilience.';

  @override
  String get featPairingTitle => 'Secure pairing';

  @override
  String get featPairingDesc =>
      'Control channel uses pairing tokens and QR workflow.';

  @override
  String get featProtocolTitle => 'Open protocol';

  @override
  String get featProtocolDesc =>
      'UDP packet structure documented and extensible.';

  @override
  String get featHostUiTitle => 'Host UI';

  @override
  String get featHostUiDesc => 'Desktop UI for pairing and session control.';

  @override
  String get featAndroidClientTitle => 'Android client';

  @override
  String get featAndroidClientDesc =>
      'Jetpack Compose app that scans QR to connect.';

  @override
  String get howItWorksTitle => 'How it works';

  @override
  String get howItWorksSubtitle =>
      'From desktop display to VR headset in five synchronized pipeline steps.';

  @override
  String get step1Title => 'Capture the X11 display';

  @override
  String get step1Desc =>
      'prlx-hostd grabs the Linux desktop frames directly from X11.';

  @override
  String get step2Title => 'Encode with H.264';

  @override
  String get step2Desc =>
      'Hardware VAAPI encoding or software fallback for portability.';

  @override
  String get step3Title => 'Packetize via Parallax framing';

  @override
  String get step3Desc =>
      'Frames split into UDP packets with stream/frame metadata.';

  @override
  String get step4Title => 'Coordinate over TCP';

  @override
  String get step4Desc => 'Pairing token and session control flows over TCP.';

  @override
  String get step5Title => 'Android client decodes';

  @override
  String get step5Desc =>
      'Jetpack Compose client scans QR and renders the stream.';

  @override
  String get gettingStartedTitle => 'Getting started';

  @override
  String get gettingStartedSubtitle =>
      'Choose an installation method to deploy Parallax on your Linux host.';

  @override
  String get sample1Title => 'One-command install (Debian/Ubuntu)';

  @override
  String get sample1Caption =>
      'Installs dependencies, binaries, CLI command, and desktop launcher.';

  @override
  String get sample2Title => 'Cloudflare Pages installer URL';

  @override
  String get sample2Caption =>
      'Installer published as a Flutter web asset from page/install.sh.';

  @override
  String get sample3Title => 'Cargo install flow';

  @override
  String get sample3Caption =>
      'Alternative for users who prefer a cargo-based flow.';

  @override
  String get sample4Title => 'Repository install flow';

  @override
  String get sample4Caption =>
      'Manual alternative for users who want full step-by-step control.';

  @override
  String get copyCommand => 'Copy command';

  @override
  String get commandCopied => 'Copied to clipboard';

  @override
  String get protocolTitle => 'Protocol: UDP packet framing';

  @override
  String get protocolSubtitle =>
      'Parallax uses a fixed 24-byte header with a magic \"PRLX\" value, stream/frame identifiers, and flags for keyframes, config, and end-of-frame markers.';

  @override
  String get protocolHighlight1 =>
      'Fixed 24-byte header; big-endian network order.';

  @override
  String get protocolHighlight2 => 'Magic value \"PRLX\" validates framing.';

  @override
  String get protocolHighlight3 =>
      'Stream ID + Frame ID for reassembly and timing.';

  @override
  String get protocolHighlight4 =>
      'Flags for keyframe, config, and end-of-frame markers.';

  @override
  String get protocolHighlight5 =>
      'Payload types for video, audio, and control data.';

  @override
  String get protocolPayloadNotesTitle => 'Payload notes';

  @override
  String get protocolNote1 =>
      'H.264 payloads are Annex B byte streams (start-code delimited).';

  @override
  String get protocolNote2 =>
      'SPS/PPS can be inline with keyframes or config packets.';

  @override
  String get roadmapTitle => 'Project status & roadmap';

  @override
  String get roadmapSubtitle =>
      'Early experimental release (v0.1). Actively evolving; stability and latency work ongoing.';

  @override
  String get roadmapStatus1 => 'Early experimental release (v0.1).';

  @override
  String get roadmapStatus2 =>
      'Actively evolving; stability and latency work ongoing.';

  @override
  String get roadmapStatus3 => 'Contributions welcome from the community.';

  @override
  String get roadmapItem1Title => 'Optimize latency';

  @override
  String get roadmapItem1Desc =>
      'Tighten capture → encode → transport pipeline timings.';

  @override
  String get roadmapItem2Title => 'Improve hardware encoder support';

  @override
  String get roadmapItem2Desc =>
      'Broaden VAAPI/codec paths and smarter fallbacks.';

  @override
  String get roadmapItem3Title => 'Broaden client platforms';

  @override
  String get roadmapItem3Desc =>
      'Explore desktop and additional mobile clients.';

  @override
  String get roadmapItem4Title => 'VR-native rendering';

  @override
  String get roadmapItem4Desc => 'Integrate immersive VR playback and input.';

  @override
  String get communityTitle => 'Open-source & community';

  @override
  String get communityDescription =>
      'Parallax is MIT licensed and focused on research + experimentation. GitHub contributions and feedback are encouraged.';

  @override
  String get communityItem1 => 'MIT License';

  @override
  String get communityItem2 => 'Open protocol documentation';

  @override
  String get communityItem3 => 'Active GitHub issues & discussions';

  @override
  String get finalCtaHeadline => 'Build the future of VR streaming.';

  @override
  String get finalCtaSubHeadline =>
      'Parallax is open-source and evolving — join the community.';

  @override
  String get footerCopyright => '© 2026 Asodya. All rights reserved.';

  @override
  String get questSectionTitle => 'Target Hardware: Meta Quest Ecosystem';

  @override
  String get questSectionSubtitle =>
      'Engineered for Meta Quest 3, Quest 3S, and Android XR clients with low-latency UDP framing and high frame-rate immersion.';

  @override
  String get questOptimizedBadge => 'OPTIMIZED FOR META QUEST 3 & QUEST 3S';

  @override
  String get questCard1Title => 'Meta Quest 3S Wireless Pipeline';

  @override
  String get questCard1Desc =>
      'Stream full-resolution Linux X11 desktops over local Wi-Fi 6E with sub-20ms motion-to-photon latency.';

  @override
  String get questCard2Title => 'Spatial Room-Scale Workspace';

  @override
  String get questCard2Desc =>
      'Transform your terminal, IDE, and multi-monitor workflows into floating spatial displays anywhere in your room.';

  @override
  String get questCard3Title => 'Hardware-Accelerated Decoding';

  @override
  String get questCard3Desc =>
      'Snapdragon XR2 Gen 2 native H.264 decode pipeline with zero battery-draining CPU emulation.';

  @override
  String get questLinkSpecs => 'Meta Quest 3S Specs';

  @override
  String get questLinkDev => 'Meta Quest Developer Center';

  @override
  String get questLinkAdb => 'Sideloading & ADB Guide';

  @override
  String get questLinkSidequest => 'SideQuest Community';
}
