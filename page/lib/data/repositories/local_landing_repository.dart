import '../../domain/entities/landing_data.dart';
import '../../domain/repositories/landing_repository.dart';

class LocalLandingRepository implements LandingRepository {
  @override
  LandingData fetchLandingData() {
    return const LandingData(
      hero: HeroContent(
        headline:
            'Parallax — Stream your Linux desktop into VR with ultra-low latency.',
        subHeadline:
            'Open-source. Experimental. Built for the future of spatial computing.',
        primaryCta: 'Get Started',
        secondaryCta: 'View on GitHub',
        microText: 'Currently in early development (v0.1).',
      ),
      about: AboutContent(
        title: 'What is Parallax?',
        description:
            'Parallax is an experimental remote-rendering pipeline for streaming a Linux desktop into a client device with a focus on VR/AR use cases. It captures an X11 display, encodes it as H.264, transmits it over UDP, and coordinates sessions over a lightweight TCP control channel.',
        details: [
          'Linux host captures X11 display for real-time VR streaming.',
          'Frames are encoded to H.264 using hardware (VAAPI) or software fallback.',
          'Video frames stream over UDP for ultra-low latency delivery.',
          'TCP control channel manages pairing tokens and session setup.',
          'Early-stage and evolving — community contributions are welcome.',
        ],
      ),
      architecture: ArchitectureContent(
        title: 'Architecture Overview',
        description:
            'A minimal, low-latency pipeline between a Linux host daemon and an Android client with bi-directional control and one-way video streaming.',
        hostNodes: [
          ArchitectureNode(
            title: 'Linux Host (Rust)',
            subtitle: 'prlx-hostd',
            tooltip:
                'Captures X11 frames, encodes H.264, and serves control sessions.',
          ),
          ArchitectureNode(
            title: 'X11 Capture',
            subtitle: 'Display feed',
            tooltip:
                'Grabs Linux desktop frames directly from the X11 display server.',
          ),
          ArchitectureNode(
            title: 'H.264 Encode',
            subtitle: 'VAAPI or software',
            tooltip: 'Encodes frames into H.264 Annex B byte streams.',
          ),
          ArchitectureNode(
            title: 'UDP Video Stream',
            subtitle: 'Minimal framing',
            tooltip: 'Sends UDP packets with a lightweight 24-byte header.',
          ),
        ],
        clientNodes: [
          ArchitectureNode(
            title: 'Android Client',
            subtitle: 'Parallax Receiver',
            tooltip: 'Jetpack Compose app for VR/AR streaming.',
          ),
          ArchitectureNode(
            title: 'QR Pairing',
            subtitle: 'Token workflow',
            tooltip: 'Scans QR to join the control channel securely.',
          ),
          ArchitectureNode(
            title: 'TCP Control',
            subtitle: 'Session broker',
            tooltip: 'Coordinates session setup, pairing, and configuration.',
          ),
          ArchitectureNode(
            title: 'H.264 Decode',
            subtitle: 'Stream UI',
            tooltip: 'Decodes H.264 frames for immersive rendering.',
          ),
        ],
        flows: [
          DataFlow(
            label: 'TCP control (bi-directional)',
            direction: FlowDirection.biDirectional,
          ),
          DataFlow(
            label: 'UDP video stream (host → client)',
            direction: FlowDirection.hostToClient,
          ),
        ],
      ),
      features: [
        FeatureItem(
          title: 'Low-latency UDP streaming',
          description: 'Minimal framing protocol for speed and resilience.',
          icon: '⚡',
        ),
        FeatureItem(
          title: 'Secure pairing',
          description: 'Control channel uses pairing tokens and QR workflow.',
          icon: '🔐',
        ),
        FeatureItem(
          title: 'Open protocol',
          description: 'UDP packet structure documented and extensible.',
          icon: '🧩',
        ),
        FeatureItem(
          title: 'Host UI',
          description: 'Desktop UI for pairing and session control.',
          icon: '🖥️',
        ),
        FeatureItem(
          title: 'Android client',
          description: 'Jetpack Compose app that scans QR to connect.',
          icon: '📱',
        ),
      ],
      steps: [
        StepItem(
          title: 'Capture the X11 display',
          description:
              'prlx-hostd grabs the Linux desktop frames directly from X11.',
        ),
        StepItem(
          title: 'Encode with H.264',
          description:
              'Hardware VAAPI encoding or software fallback for portability.',
        ),
        StepItem(
          title: 'Packetize via Parallax framing',
          description:
              'Frames split into UDP packets with stream/frame metadata.',
        ),
        StepItem(
          title: 'Coordinate over TCP',
          description: 'Pairing token and session control flows over TCP.',
        ),
        StepItem(
          title: 'Android client decodes',
          description:
              'Jetpack Compose client scans QR and renders the stream.',
        ),
      ],
      gettingStarted: [
        CodeSample(
          title: 'One-command install (Debian/Ubuntu)',
          command:
              'curl -fsSL https://raw.githubusercontent.com/asodya/parallax/main/install.sh | bash',
          caption:
              'Installs dependencies, binaries, CLI command, and desktop launcher.',
        ),
        CodeSample(
          title: 'Cloudflare Pages installer URL',
          command:
              'curl -fsSL https://parallax.asodya.com/assets/assets/install.sh | bash',
          caption:
              'Installer published as a Flutter web asset from page/assets/install.sh.',
        ),
        CodeSample(
          title: 'Cargo install flow',
          command: 'cargo install --path host\n./packaging/install-debian.sh',
          caption: 'Alternative for users who prefer a cargo-based flow.',
        ),
        CodeSample(
          title: 'Repository install flow',
          command:
              'git clone https://github.com/asodya/parallax.git\ncd parallax\n./packaging/install-debian.sh',
          caption:
              'Manual alternative for users who want full step-by-step control.',
        ),
      ],
      protocol: ProtocolContent(
        title: 'Protocol: UDP packet framing',
        summary:
            'Parallax uses a fixed 24-byte header with a magic “PRLX” value, stream/frame identifiers, and flags for keyframes, config, and end-of-frame markers.',
        highlights: [
          'Fixed 24-byte header; big-endian network order.',
          'Magic value “PRLX” validates framing.',
          'Stream ID + Frame ID for reassembly and timing.',
          'Flags for keyframe, config, and end-of-frame markers.',
          'Payload types for video, audio, and control data.',
        ],
        payloadNotes: [
          'H.264 payloads are Annex B byte streams (start-code delimited).',
          'SPS/PPS can be inline with keyframes or config packets.',
        ],
      ),
      roadmap: RoadmapContent(
        title: 'Project status & roadmap',
        statusLines: [
          'Early experimental release (v0.1).',
          'Actively evolving; stability and latency work ongoing.',
          'Contributions welcome from the community.',
        ],
        timeline: [
          RoadmapItem(
            title: 'Optimize latency',
            description:
                'Tighten capture → encode → transport pipeline timings.',
          ),
          RoadmapItem(
            title: 'Improve hardware encoder support',
            description: 'Broaden VAAPI/codec paths and smarter fallbacks.',
          ),
          RoadmapItem(
            title: 'Broaden client platforms',
            description: 'Explore desktop and additional mobile clients.',
          ),
          RoadmapItem(
            title: 'VR-native rendering',
            description: 'Integrate immersive VR playback and input.',
          ),
        ],
      ),
      community: CommunityContent(
        title: 'Open-source & community',
        description:
            'Parallax is MIT licensed and focused on research + experimentation. GitHub contributions and feedback are encouraged.',
        items: [
          'MIT License',
          'Open protocol documentation',
          'Active GitHub issues & discussions',
        ],
      ),
      finalCta: FinalCtaContent(
        headline: 'Build the future of VR streaming.',
        subHeadline:
            'Parallax is open-source and evolving — join the community.',
        primaryCta: 'Get Started',
        secondaryCta: 'View on GitHub',
      ),
    );
  }
}
