import "package:flutter/foundation.dart";
import "package:flutter/material.dart";
import "package:flutter/services.dart";
import "package:url_launcher/url_launcher.dart";

import "../../domain/entities/landing_data.dart";
import "../../l10n/app_localizations.dart";
import "../viewmodels/landing_view_model.dart";
import "../widgets/animated_background.dart";
import "../widgets/glow_card.dart";
import "../widgets/gradient_text.dart";
import "../widgets/neon_button.dart";
import "../widgets/parallax_header.dart";
import "../widgets/section_header.dart";

class LandingPage extends StatefulWidget {
  const LandingPage({super.key, required this.viewModel});

  final LandingViewModel viewModel;

  @override
  State<LandingPage> createState() => _LandingPageState();
}

class _LandingPageState extends State<LandingPage> {
  late final ScrollController _scrollController;
  final ValueNotifier<double> _scrollOffset = ValueNotifier<double>(0);

  final GlobalKey _heroKey = GlobalKey();
  final GlobalKey _aboutKey = GlobalKey();
  final GlobalKey _architectureKey = GlobalKey();
  final GlobalKey _featuresKey = GlobalKey();
  final GlobalKey _howItWorksKey = GlobalKey();
  final GlobalKey _gettingStartedKey = GlobalKey();
  final GlobalKey _protocolKey = GlobalKey();
  final GlobalKey _roadmapKey = GlobalKey();
  final GlobalKey _communityKey = GlobalKey();

  @override
  void initState() {
    super.initState();
    _scrollController = ScrollController()
      ..addListener(() => _scrollOffset.value = _scrollController.offset);
  }

  @override
  void dispose() {
    _scrollController.dispose();
    _scrollOffset.dispose();
    super.dispose();
  }

  void _scrollToSection(String sectionKey) {
    GlobalKey? targetKey;
    switch (sectionKey) {
      case "hero":
        targetKey = _heroKey;
        break;
      case "about":
        targetKey = _aboutKey;
        break;
      case "architecture":
        targetKey = _architectureKey;
        break;
      case "features":
        targetKey = _featuresKey;
        break;
      case "howItWorks":
        targetKey = _howItWorksKey;
        break;
      case "gettingStarted":
        targetKey = _gettingStartedKey;
        break;
      case "protocol":
        targetKey = _protocolKey;
        break;
      case "roadmap":
        targetKey = _roadmapKey;
        break;
      case "community":
        targetKey = _communityKey;
        break;
    }

    if (targetKey?.currentContext != null) {
      Scrollable.ensureVisible(
        targetKey!.currentContext!,
        duration: const Duration(milliseconds: 600),
        curve: Curves.easeInOutCubic,
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;
    final colorScheme = theme.colorScheme;
    final fallbackData = widget.viewModel.landingData;

    return Scaffold(
      body: Stack(
        children: [
          // Background with architectural grid & subtle particles
          Positioned.fill(
            child: AnimatedBackground(scrollOffset: _scrollOffset),
          ),

          // Scrollable content
          Positioned.fill(
            child: SingleChildScrollView(
              controller: _scrollController,
              padding: const EdgeInsets.only(top: 64),
              child: Column(
                children: [
                  HeroSection(
                    key: _heroKey,
                    l10n: l10n,
                    fallback: fallbackData.hero,
                    onGetStarted: () => _scrollToSection("gettingStarted"),
                  ),
                  SectionWrapper(
                    key: _aboutKey,
                    scrollOffset: _scrollOffset,
                    child: AboutSection(l10n: l10n, fallback: fallbackData.about),
                  ),
                  SectionWrapper(
                    key: _architectureKey,
                    scrollOffset: _scrollOffset,
                    child: ArchitectureSection(
                      l10n: l10n,
                      fallback: fallbackData.architecture,
                    ),
                  ),
                  SectionWrapper(
                    key: _featuresKey,
                    scrollOffset: _scrollOffset,
                    child: FeaturesSection(
                      l10n: l10n,
                      fallbackItems: fallbackData.features,
                    ),
                  ),
                  SectionWrapper(
                    key: _howItWorksKey,
                    scrollOffset: _scrollOffset,
                    child: HowItWorksSection(
                      l10n: l10n,
                      fallbackSteps: fallbackData.steps,
                    ),
                  ),
                  SectionWrapper(
                    key: _gettingStartedKey,
                    scrollOffset: _scrollOffset,
                    child: GettingStartedSection(
                      l10n: l10n,
                      fallbackSamples: fallbackData.gettingStarted,
                    ),
                  ),
                  SectionWrapper(
                    key: _protocolKey,
                    scrollOffset: _scrollOffset,
                    child: ProtocolSection(
                      l10n: l10n,
                      fallback: fallbackData.protocol,
                    ),
                  ),
                  SectionWrapper(
                    key: _roadmapKey,
                    scrollOffset: _scrollOffset,
                    child: RoadmapSection(
                      l10n: l10n,
                      fallback: fallbackData.roadmap,
                    ),
                  ),
                  SectionWrapper(
                    key: _communityKey,
                    scrollOffset: _scrollOffset,
                    child: CommunitySection(
                      l10n: l10n,
                      fallback: fallbackData.community,
                    ),
                  ),
                  SectionWrapper(
                    scrollOffset: _scrollOffset,
                    child: FinalCtaSection(
                      l10n: l10n,
                      fallback: fallbackData.finalCta,
                    ),
                  ),
                  // Footer
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 32),
                    decoration: BoxDecoration(
                      border: Border(
                        top: BorderSide(
                          color: isDark ? const Color(0xFF262626) : const Color(0xFFE5E7EB),
                        ),
                      ),
                    ),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text(
                          l10n?.footerCopyright ?? "© 2026 Asodya. All rights reserved.",
                          style: theme.textTheme.bodySmall?.copyWith(
                            color: colorScheme.onSurfaceVariant,
                          ),
                        ),
                        Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Text(
                              "ASODYA ECOSYSTEM",
                              style: theme.textTheme.labelSmall?.copyWith(
                                letterSpacing: 1.5,
                                color: colorScheme.onSurfaceVariant,
                                fontWeight: FontWeight.w700,
                              ),
                            ),
                          ],
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),

          // Sticky Navigation Header
          Positioned(
            top: 0,
            left: 0,
            right: 0,
            child: ParallaxHeader(onNavigateToSection: _scrollToSection),
          ),
        ],
      ),
    );
  }
}

class HeroSection extends StatelessWidget {
  const HeroSection({
    super.key,
    required this.l10n,
    required this.fallback,
    required this.onGetStarted,
  });

  final AppLocalizations? l10n;
  final HeroContent fallback;
  final VoidCallback onGetStarted;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;

    final headline = l10n?.heroHeadline ?? fallback.headline;
    final subHeadline = l10n?.heroSubHeadline ?? fallback.subHeadline;
    final primaryCta = l10n?.heroPrimaryCta ?? fallback.primaryCta;
    final secondaryCta = l10n?.heroSecondaryCta ?? fallback.secondaryCta;
    final microText = l10n?.heroMicroText ?? fallback.microText;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 64),
      constraints: const BoxConstraints(maxWidth: 1040),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          // Tag pill
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
            decoration: BoxDecoration(
              color: isDark
                  ? const Color(0xFF191A16)
                  : const Color(0xFFEAEAEA),
              borderRadius: BorderRadius.circular(20),
              border: Border.all(
                color: isDark ? const Color(0xFF262626) : const Color(0xFFD1D5DB),
              ),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: 7,
                  height: 7,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: isDark ? const Color(0xFFD7FF3F) : Colors.black,
                  ),
                ),
                const SizedBox(width: 8),
                Text(
                  "EXPERIMENTAL SPATIAL STREAMING",
                  style: theme.textTheme.labelSmall?.copyWith(
                    fontWeight: FontWeight.w700,
                    letterSpacing: 1.2,
                    color: colorScheme.onSurface,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 28),

          GradientText(
            "Parallax",
            style: theme.textTheme.displayLarge?.copyWith(
              fontWeight: FontWeight.w800,
              letterSpacing: -1.5,
              fontSize: 64,
            ),
          ),
          const SizedBox(height: 20),

          Text(
            headline,
            textAlign: TextAlign.center,
            style: theme.textTheme.headlineMedium?.copyWith(
              fontWeight: FontWeight.w700,
              color: colorScheme.onSurface,
              height: 1.25,
            ),
          ),
          const SizedBox(height: 16),

          Text(
            subHeadline,
            textAlign: TextAlign.center,
            style: theme.textTheme.titleMedium?.copyWith(
              color: colorScheme.onSurfaceVariant,
              height: 1.6,
            ),
          ),
          const SizedBox(height: 32),

          Wrap(
            spacing: 16,
            runSpacing: 12,
            alignment: WrapAlignment.center,
            children: [
              NeonButton(
                label: primaryCta,
                icon: const Icon(Icons.arrow_forward, size: 16),
                onPressed: onGetStarted,
              ),
              NeonButton(
                label: secondaryCta,
                isPrimary: false,
                icon: const Icon(Icons.code, size: 16),
                onPressed: () => _openGithub(context),
              ),
            ],
          ),
          const SizedBox(height: 16),

          Text(
            microText,
            style: theme.textTheme.bodySmall?.copyWith(
              color: colorScheme.onSurfaceVariant,
              letterSpacing: 0.3,
            ),
          ),
          const SizedBox(height: 48),

          // Quick install cards
          Wrap(
            spacing: 16,
            runSpacing: 16,
            alignment: WrapAlignment.center,
            children: [
              _HeroInstallCard(
                title: l10n?.sample1Title ?? "One command (curl + bash)",
                command: "curl -fsSL https://parallax.asodya.com/assets/install.sh | bash",
                copyTooltip: l10n?.copyCommand ?? "Copy command",
                copiedToast: l10n?.commandCopied ?? "Copied to clipboard",
              ),
              _HeroInstallCard(
                title: l10n?.sample3Title ?? "Cargo flow",
                command: "cargo install --path host\n./packaging/install-debian.sh",
                copyTooltip: l10n?.copyCommand ?? "Copy command",
                copiedToast: l10n?.commandCopied ?? "Copied to clipboard",
              ),
              _HeroInstallCard(
                title: l10n?.sample4Title ?? "Repository flow",
                command: "git clone https://github.com/asodya/parallax.git\ncd parallax\n./packaging/install-debian.sh",
                copyTooltip: l10n?.copyCommand ?? "Copy command",
                copiedToast: l10n?.commandCopied ?? "Copied to clipboard",
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _HeroInstallCard extends StatelessWidget {
  const _HeroInstallCard({
    required this.title,
    required this.command,
    required this.copyTooltip,
    required this.copiedToast,
  });

  final String title;
  final String command;
  final String copyTooltip;
  final String copiedToast;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;
    final colorScheme = theme.colorScheme;

    return SizedBox(
      width: 300,
      child: GlowCard(
        padding: const EdgeInsets.all(18),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Expanded(
                  child: Text(
                    title,
                    style: theme.textTheme.titleSmall?.copyWith(
                      fontWeight: FontWeight.w700,
                      color: colorScheme.onSurface,
                    ),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                IconButton(
                  onPressed: () => _copyCommand(context, command, copiedToast),
                  icon: const Icon(Icons.copy_outlined, size: 16),
                  tooltip: copyTooltip,
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(),
                ),
              ],
            ),
            const SizedBox(height: 10),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: isDark ? const Color(0xFF10110E) : const Color(0xFFF3F4F6),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: isDark ? const Color(0xFF262626) : const Color(0xFFE5E7EB),
                ),
              ),
              child: Text(
                command,
                style: theme.textTheme.bodySmall?.copyWith(
                  fontFamily: "monospace",
                  color: colorScheme.onSurface,
                  fontSize: 11,
                  height: 1.4,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class AboutSection extends StatelessWidget {
  const AboutSection({super.key, required this.l10n, required this.fallback});

  final AppLocalizations? l10n;
  final AboutContent fallback;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;

    final title = l10n?.aboutTitle ?? fallback.title;
    final desc = l10n?.aboutDescription ?? fallback.description;

    final details = [
      l10n?.aboutDetail1 ?? fallback.details[0],
      l10n?.aboutDetail2 ?? fallback.details[1],
      l10n?.aboutDetail3 ?? fallback.details[2],
      l10n?.aboutDetail4 ?? fallback.details[3],
      l10n?.aboutDetail5 ?? fallback.details[4],
    ];

    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(title: title, subtitle: desc),
          const SizedBox(height: 28),
          Wrap(
            spacing: 16,
            runSpacing: 16,
            children: details.map((detail) {
              return SizedBox(
                width: 320,
                child: GlowCard(
                  padding: const EdgeInsets.all(18),
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Icon(
                        Icons.check_circle_outline,
                        size: 20,
                        color: isDark ? const Color(0xFFD7FF3F) : const Color(0xFF181818),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Text(
                          detail,
                          style: theme.textTheme.bodyMedium?.copyWith(
                            color: colorScheme.onSurface,
                            height: 1.45,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              );
            }).toList(),
          ),
        ],
      ),
    );
  }
}

class ArchitectureSection extends StatelessWidget {
  const ArchitectureSection({super.key, required this.l10n, required this.fallback});

  final AppLocalizations? l10n;
  final ArchitectureContent fallback;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;

    final title = l10n?.architectureTitle ?? fallback.title;
    final desc = l10n?.architectureSubtitle ?? fallback.description;

    final hostNodes = [
      ArchitectureNode(
        title: l10n?.archHostTitle ?? fallback.hostNodes[0].title,
        subtitle: l10n?.archHostSubtitle ?? fallback.hostNodes[0].subtitle,
        tooltip: l10n?.archHostTooltip ?? fallback.hostNodes[0].tooltip,
      ),
      ArchitectureNode(
        title: l10n?.archX11Title ?? fallback.hostNodes[1].title,
        subtitle: l10n?.archX11Subtitle ?? fallback.hostNodes[1].subtitle,
        tooltip: l10n?.archX11Tooltip ?? fallback.hostNodes[1].tooltip,
      ),
      ArchitectureNode(
        title: l10n?.archEncodeTitle ?? fallback.hostNodes[2].title,
        subtitle: l10n?.archEncodeSubtitle ?? fallback.hostNodes[2].subtitle,
        tooltip: l10n?.archEncodeTooltip ?? fallback.hostNodes[2].tooltip,
      ),
      ArchitectureNode(
        title: l10n?.archUdpStreamTitle ?? fallback.hostNodes[3].title,
        subtitle: l10n?.archUdpStreamSubtitle ?? fallback.hostNodes[3].subtitle,
        tooltip: l10n?.archUdpStreamTooltip ?? fallback.hostNodes[3].tooltip,
      ),
    ];

    final clientNodes = [
      ArchitectureNode(
        title: l10n?.archAndroidTitle ?? fallback.clientNodes[0].title,
        subtitle: l10n?.archAndroidSubtitle ?? fallback.clientNodes[0].subtitle,
        tooltip: l10n?.archAndroidTooltip ?? fallback.clientNodes[0].tooltip,
      ),
      ArchitectureNode(
        title: l10n?.archQrTitle ?? fallback.clientNodes[1].title,
        subtitle: l10n?.archQrSubtitle ?? fallback.clientNodes[1].subtitle,
        tooltip: l10n?.archQrTooltip ?? fallback.clientNodes[1].tooltip,
      ),
      ArchitectureNode(
        title: l10n?.archTcpTitle ?? fallback.clientNodes[2].title,
        subtitle: l10n?.archTcpSubtitle ?? fallback.clientNodes[2].subtitle,
        tooltip: l10n?.archTcpTooltip ?? fallback.clientNodes[2].tooltip,
      ),
      ArchitectureNode(
        title: l10n?.archDecodeTitle ?? fallback.clientNodes[3].title,
        subtitle: l10n?.archDecodeSubtitle ?? fallback.clientNodes[3].subtitle,
        tooltip: l10n?.archDecodeTooltip ?? fallback.clientNodes[3].tooltip,
      ),
    ];

    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(title: title, subtitle: desc),
          const SizedBox(height: 28),
          LayoutBuilder(
            builder: (context, constraints) {
              final isNarrow = constraints.maxWidth < 720;
              return Flex(
                direction: isNarrow ? Axis.vertical : Axis.horizontal,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(
                    flex: isNarrow ? 0 : 1,
                    child: _ArchitectureColumn(
                      title: l10n?.archHostTitle ?? "Linux Host (Rust)",
                      nodes: hostNodes,
                      icon: Icons.computer_outlined,
                    ),
                  ),
                  SizedBox(width: isNarrow ? 0 : 24, height: isNarrow ? 24 : 0),
                  Expanded(
                    flex: isNarrow ? 0 : 1,
                    child: _ArchitectureColumn(
                      title: l10n?.archAndroidTitle ?? "Android Client",
                      nodes: clientNodes,
                      icon: Icons.phone_android_outlined,
                    ),
                  ),
                ],
              );
            },
          ),
          const SizedBox(height: 24),
          Wrap(
            spacing: 12,
            runSpacing: 8,
            children: [
              Chip(
                label: Text(l10n?.archFlowTcp ?? "TCP control (bi-directional)"),
                avatar: Icon(
                  Icons.sync_alt,
                  size: 16,
                  color: colorScheme.onSurface,
                ),
                backgroundColor: isDark ? const Color(0xFF191A16) : const Color(0xFFF3F4F6),
                side: BorderSide(
                  color: isDark ? const Color(0xFF262626) : const Color(0xFFE5E7EB),
                ),
              ),
              Chip(
                label: Text(l10n?.archFlowUdp ?? "UDP video stream (host → client)"),
                avatar: Icon(
                  Icons.arrow_forward,
                  size: 16,
                  color: colorScheme.onSurface,
                ),
                backgroundColor: isDark ? const Color(0xFF191A16) : const Color(0xFFF3F4F6),
                side: BorderSide(
                  color: isDark ? const Color(0xFF262626) : const Color(0xFFE5E7EB),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class FeaturesSection extends StatelessWidget {
  const FeaturesSection({
    super.key,
    required this.l10n,
    required this.fallbackItems,
  });

  final AppLocalizations? l10n;
  final List<FeatureItem> fallbackItems;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;

    final title = l10n?.featuresTitle ?? "Core features";
    final desc = l10n?.featuresSubtitle ?? "High-performance streaming primitives designed for VR latency budgets.";

    final items = [
      (
        icon: Icons.bolt_outlined,
        title: l10n?.featUdpTitle ?? fallbackItems[0].title,
        desc: l10n?.featUdpDesc ?? fallbackItems[0].description,
      ),
      (
        icon: Icons.lock_outlined,
        title: l10n?.featPairingTitle ?? fallbackItems[1].title,
        desc: l10n?.featPairingDesc ?? fallbackItems[1].description,
      ),
      (
        icon: Icons.extension_outlined,
        title: l10n?.featProtocolTitle ?? fallbackItems[2].title,
        desc: l10n?.featProtocolDesc ?? fallbackItems[2].description,
      ),
      (
        icon: Icons.desktop_windows_outlined,
        title: l10n?.featHostUiTitle ?? fallbackItems[3].title,
        desc: l10n?.featHostUiDesc ?? fallbackItems[3].description,
      ),
      (
        icon: Icons.phone_android_outlined,
        title: l10n?.featAndroidClientTitle ?? fallbackItems[4].title,
        desc: l10n?.featAndroidClientDesc ?? fallbackItems[4].description,
      ),
    ];

    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(title: title, subtitle: desc),
          const SizedBox(height: 28),
          LayoutBuilder(
            builder: (context, constraints) {
              final width = constraints.maxWidth;
              final crossAxisCount = width > 1000 ? 3 : (width > 680 ? 2 : 1);
              final itemWidth = (width - (crossAxisCount - 1) * 20) / crossAxisCount;

              return Wrap(
                spacing: 20,
                runSpacing: 20,
                children: items.map((item) {
                  return SizedBox(
                    width: itemWidth,
                    child: GlowCard(
                      onTap: () => _openGithub(context),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Container(
                            padding: const EdgeInsets.all(10),
                            decoration: BoxDecoration(
                              color: isDark ? const Color(0xFF10110E) : const Color(0xFFF3F4F6),
                              borderRadius: BorderRadius.circular(10),
                              border: Border.all(
                                color: isDark ? const Color(0xFF262626) : const Color(0xFFE5E7EB),
                              ),
                            ),
                            child: Icon(
                              item.icon,
                              size: 24,
                              color: isDark ? const Color(0xFFD7FF3F) : const Color(0xFF181818),
                            ),
                          ),
                          const SizedBox(height: 16),
                          Text(
                            item.title,
                            style: theme.textTheme.titleMedium?.copyWith(
                              fontWeight: FontWeight.w700,
                              color: colorScheme.onSurface,
                            ),
                          ),
                          const SizedBox(height: 8),
                          Text(
                            item.desc,
                            style: theme.textTheme.bodyMedium?.copyWith(
                              color: colorScheme.onSurfaceVariant,
                              height: 1.5,
                            ),
                          ),
                        ],
                      ),
                    ),
                  );
                }).toList(),
              );
            },
          ),
        ],
      ),
    );
  }
}

class HowItWorksSection extends StatefulWidget {
  const HowItWorksSection({
    super.key,
    required this.l10n,
    required this.fallbackSteps,
  });

  final AppLocalizations? l10n;
  final List<StepItem> fallbackSteps;

  @override
  State<HowItWorksSection> createState() => _HowItWorksSectionState();
}

class _HowItWorksSectionState extends State<HowItWorksSection> {
  int _selectedIndex = 0;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;
    final l10n = widget.l10n;

    final steps = [
      (
        title: l10n?.step1Title ?? widget.fallbackSteps[0].title,
        desc: l10n?.step1Desc ?? widget.fallbackSteps[0].description,
      ),
      (
        title: l10n?.step2Title ?? widget.fallbackSteps[1].title,
        desc: l10n?.step2Desc ?? widget.fallbackSteps[1].description,
      ),
      (
        title: l10n?.step3Title ?? widget.fallbackSteps[2].title,
        desc: l10n?.step3Desc ?? widget.fallbackSteps[2].description,
      ),
      (
        title: l10n?.step4Title ?? widget.fallbackSteps[3].title,
        desc: l10n?.step4Desc ?? widget.fallbackSteps[3].description,
      ),
      (
        title: l10n?.step5Title ?? widget.fallbackSteps[4].title,
        desc: l10n?.step5Desc ?? widget.fallbackSteps[4].description,
      ),
    ];

    final selected = steps[_selectedIndex];

    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(
            title: l10n?.howItWorksTitle ?? "How it works",
            subtitle: l10n?.howItWorksSubtitle ?? "From desktop display to VR headset in five synchronized pipeline steps.",
          ),
          const SizedBox(height: 24),
          Wrap(
            spacing: 10,
            runSpacing: 10,
            children: steps.asMap().entries.map((entry) {
              final idx = entry.key;
              final isSel = idx == _selectedIndex;
              return ChoiceChip(
                label: Text("0${idx + 1}"),
                selected: isSel,
                onSelected: (_) => setState(() => _selectedIndex = idx),
                selectedColor: isDark ? Colors.white : Colors.black,
                backgroundColor: isDark ? const Color(0xFF191A16) : const Color(0xFFF3F4F6),
                labelStyle: TextStyle(
                  color: isSel
                      ? (isDark ? Colors.black : Colors.white)
                      : colorScheme.onSurface,
                  fontWeight: FontWeight.w700,
                ),
                side: BorderSide(
                  color: isDark ? const Color(0xFF262626) : const Color(0xFFE5E7EB),
                ),
              );
            }).toList(),
          ),
          const SizedBox(height: 20),
          GlowCard(
            padding: const EdgeInsets.all(24),
            child: Row(
              children: [
                Container(
                  width: 48,
                  height: 48,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: isDark ? Colors.white : Colors.black,
                  ),
                  child: Center(
                    child: Text(
                      "${_selectedIndex + 1}",
                      style: TextStyle(
                        fontSize: 18,
                        fontWeight: FontWeight.w900,
                        color: isDark ? Colors.black : Colors.white,
                      ),
                    ),
                  ),
                ),
                const SizedBox(width: 20),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        selected.title,
                        style: theme.textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.w700,
                          color: colorScheme.onSurface,
                        ),
                      ),
                      const SizedBox(height: 6),
                      Text(
                        selected.desc,
                        style: theme.textTheme.bodyMedium?.copyWith(
                          color: colorScheme.onSurfaceVariant,
                          height: 1.5,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class GettingStartedSection extends StatelessWidget {
  const GettingStartedSection({
    super.key,
    required this.l10n,
    required this.fallbackSamples,
  });

  final AppLocalizations? l10n;
  final List<CodeSample> fallbackSamples;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;

    final samples = [
      (
        title: l10n?.sample1Title ?? fallbackSamples[0].title,
        command: fallbackSamples[0].command,
        caption: l10n?.sample1Caption ?? fallbackSamples[0].caption,
      ),
      (
        title: l10n?.sample2Title ?? fallbackSamples[1].title,
        command: fallbackSamples[1].command,
        caption: l10n?.sample2Caption ?? fallbackSamples[1].caption,
      ),
      (
        title: l10n?.sample3Title ?? fallbackSamples[2].title,
        command: fallbackSamples[2].command,
        caption: l10n?.sample3Caption ?? fallbackSamples[2].caption,
      ),
      (
        title: l10n?.sample4Title ?? fallbackSamples[3].title,
        command: fallbackSamples[3].command,
        caption: l10n?.sample4Caption ?? fallbackSamples[3].caption,
      ),
    ];

    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(
            title: l10n?.gettingStartedTitle ?? "Getting started",
            subtitle: l10n?.gettingStartedSubtitle ?? "Choose an installation method to deploy Parallax on your Linux host.",
          ),
          const SizedBox(height: 24),
          LayoutBuilder(
            builder: (context, constraints) {
              final cardWidth = constraints.maxWidth > 800
                  ? (constraints.maxWidth - 20) / 2
                  : constraints.maxWidth;

              return Wrap(
                spacing: 20,
                runSpacing: 20,
                children: samples.map((sample) {
                  return SizedBox(
                    width: cardWidth,
                    child: GlowCard(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            mainAxisAlignment: MainAxisAlignment.spaceBetween,
                            children: [
                              Expanded(
                                child: Text(
                                  sample.title,
                                  style: theme.textTheme.titleSmall?.copyWith(
                                    fontWeight: FontWeight.w700,
                                    color: colorScheme.onSurface,
                                  ),
                                ),
                              ),
                              IconButton(
                                onPressed: () => _copyCommand(
                                  context,
                                  sample.command,
                                  l10n?.commandCopied ?? "Copied to clipboard",
                                ),
                                icon: const Icon(Icons.copy_outlined, size: 16),
                                tooltip: l10n?.copyCommand ?? "Copy command",
                              ),
                            ],
                          ),
                          const SizedBox(height: 12),
                          Container(
                            width: double.infinity,
                            padding: const EdgeInsets.all(12),
                            decoration: BoxDecoration(
                              color: isDark ? const Color(0xFF10110E) : const Color(0xFFF3F4F6),
                              borderRadius: BorderRadius.circular(8),
                              border: Border.all(
                                color: isDark ? const Color(0xFF262626) : const Color(0xFFE5E7EB),
                              ),
                            ),
                            child: Text(
                              sample.command,
                              style: theme.textTheme.bodySmall?.copyWith(
                                fontFamily: "monospace",
                                color: colorScheme.onSurface,
                                fontSize: 12,
                                height: 1.45,
                              ),
                            ),
                          ),
                          const SizedBox(height: 10),
                          Text(
                            sample.caption,
                            style: theme.textTheme.bodySmall?.copyWith(
                              color: colorScheme.onSurfaceVariant,
                            ),
                          ),
                        ],
                      ),
                    ),
                  );
                }).toList(),
              );
            },
          ),
        ],
      ),
    );
  }
}

class ProtocolSection extends StatelessWidget {
  const ProtocolSection({super.key, required this.l10n, required this.fallback});

  final AppLocalizations? l10n;
  final ProtocolContent fallback;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;

    final highlights = [
      l10n?.protocolHighlight1 ?? fallback.highlights[0],
      l10n?.protocolHighlight2 ?? fallback.highlights[1],
      l10n?.protocolHighlight3 ?? fallback.highlights[2],
      l10n?.protocolHighlight4 ?? fallback.highlights[3],
      l10n?.protocolHighlight5 ?? fallback.highlights[4],
    ];

    final notes = [
      l10n?.protocolNote1 ?? fallback.payloadNotes[0],
      l10n?.protocolNote2 ?? fallback.payloadNotes[1],
    ];

    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(
            title: l10n?.protocolTitle ?? fallback.title,
            subtitle: l10n?.protocolSubtitle ?? fallback.summary,
          ),
          const SizedBox(height: 24),
          GlowCard(
            padding: const EdgeInsets.all(24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  "UDP Packet Highlights",
                  style: theme.textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.w700,
                    color: colorScheme.onSurface,
                  ),
                ),
                const SizedBox(height: 14),
                ...highlights.map((item) {
                  return Padding(
                    padding: const EdgeInsets.only(bottom: 8),
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Container(
                          margin: const EdgeInsets.only(top: 6),
                          width: 6,
                          height: 6,
                          decoration: BoxDecoration(
                            shape: BoxShape.circle,
                            color: isDark ? const Color(0xFFD7FF3F) : Colors.black,
                          ),
                        ),
                        const SizedBox(width: 12),
                        Expanded(
                          child: Text(
                            item,
                            style: theme.textTheme.bodyMedium?.copyWith(
                              color: colorScheme.onSurface,
                              height: 1.45,
                            ),
                          ),
                        ),
                      ],
                    ),
                  );
                }),
                const SizedBox(height: 16),
                Text(
                  l10n?.protocolPayloadNotesTitle ?? "Payload notes",
                  style: theme.textTheme.titleSmall?.copyWith(
                    fontWeight: FontWeight.w700,
                    color: colorScheme.onSurface,
                  ),
                ),
                const SizedBox(height: 8),
                ...notes.map((item) {
                  return Padding(
                    padding: const EdgeInsets.only(bottom: 6),
                    child: Text(
                      item,
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: colorScheme.onSurfaceVariant,
                        height: 1.4,
                      ),
                    ),
                  );
                }),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class RoadmapSection extends StatelessWidget {
  const RoadmapSection({super.key, required this.l10n, required this.fallback});

  final AppLocalizations? l10n;
  final RoadmapContent fallback;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;

    final items = [
      (
        title: l10n?.roadmapItem1Title ?? fallback.timeline[0].title,
        desc: l10n?.roadmapItem1Desc ?? fallback.timeline[0].description,
      ),
      (
        title: l10n?.roadmapItem2Title ?? fallback.timeline[1].title,
        desc: l10n?.roadmapItem2Desc ?? fallback.timeline[1].description,
      ),
      (
        title: l10n?.roadmapItem3Title ?? fallback.timeline[2].title,
        desc: l10n?.roadmapItem3Desc ?? fallback.timeline[2].description,
      ),
      (
        title: l10n?.roadmapItem4Title ?? fallback.timeline[3].title,
        desc: l10n?.roadmapItem4Desc ?? fallback.timeline[3].description,
      ),
    ];

    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(
            title: l10n?.roadmapTitle ?? fallback.title,
            subtitle: l10n?.roadmapSubtitle ?? fallback.statusLines.join(" "),
          ),
          const SizedBox(height: 24),
          Wrap(
            spacing: 20,
            runSpacing: 20,
            children: items.map((item) {
              return SizedBox(
                width: 300,
                child: GlowCard(
                  onTap: () => _openGithub(context),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Icon(
                            Icons.arrow_circle_right_outlined,
                            size: 18,
                            color: isDark ? const Color(0xFFD7FF3F) : const Color(0xFF181818),
                          ),
                          const SizedBox(width: 8),
                          Expanded(
                            child: Text(
                              item.title,
                              style: theme.textTheme.titleSmall?.copyWith(
                                fontWeight: FontWeight.w700,
                                color: colorScheme.onSurface,
                              ),
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 8),
                      Text(
                        item.desc,
                        style: theme.textTheme.bodyMedium?.copyWith(
                          color: colorScheme.onSurfaceVariant,
                          height: 1.45,
                        ),
                      ),
                    ],
                  ),
                ),
              );
            }).toList(),
          ),
        ],
      ),
    );
  }
}

class CommunitySection extends StatelessWidget {
  const CommunitySection({super.key, required this.l10n, required this.fallback});

  final AppLocalizations? l10n;
  final CommunityContent fallback;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;

    final items = [
      l10n?.communityItem1 ?? fallback.items[0],
      l10n?.communityItem2 ?? fallback.items[1],
      l10n?.communityItem3 ?? fallback.items[2],
    ];

    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(
            title: l10n?.communityTitle ?? fallback.title,
            subtitle: l10n?.communityDescription ?? fallback.description,
          ),
          const SizedBox(height: 20),
          Wrap(
            spacing: 12,
            runSpacing: 10,
            children: items.map((item) {
              return ActionChip(
                label: Text(item),
                onPressed: () => _openGithub(context),
                avatar: const Icon(Icons.open_in_new, size: 14),
                backgroundColor: isDark ? const Color(0xFF191A16) : const Color(0xFFF3F4F6),
                side: BorderSide(
                  color: isDark ? const Color(0xFF262626) : const Color(0xFFE5E7EB),
                ),
                labelStyle: TextStyle(
                  color: colorScheme.onSurface,
                  fontWeight: FontWeight.w500,
                ),
              );
            }).toList(),
          ),
        ],
      ),
    );
  }
}

class FinalCtaSection extends StatelessWidget {
  const FinalCtaSection({super.key, required this.l10n, required this.fallback});

  final AppLocalizations? l10n;
  final FinalCtaContent fallback;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    final headline = l10n?.finalCtaHeadline ?? fallback.headline;
    final subHeadline = l10n?.finalCtaSubHeadline ?? fallback.subHeadline;
    final primaryCta = l10n?.heroPrimaryCta ?? fallback.primaryCta;
    final secondaryCta = l10n?.heroSecondaryCta ?? fallback.secondaryCta;

    return _SectionContainer(
      child: GlowCard(
        padding: const EdgeInsets.symmetric(horizontal: 36, vertical: 48),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Text(
              headline,
              textAlign: TextAlign.center,
              style: theme.textTheme.headlineSmall?.copyWith(
                fontWeight: FontWeight.w700,
                color: colorScheme.onSurface,
              ),
            ),
            const SizedBox(height: 12),
            Text(
              subHeadline,
              textAlign: TextAlign.center,
              style: theme.textTheme.bodyLarge?.copyWith(
                color: colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 28),
            Wrap(
              spacing: 16,
              runSpacing: 12,
              alignment: WrapAlignment.center,
              children: [
                NeonButton(
                  label: primaryCta,
                  icon: const Icon(Icons.arrow_forward, size: 16),
                  onPressed: () => _openGithub(context),
                ),
                NeonButton(
                  label: secondaryCta,
                  isPrimary: false,
                  icon: const Icon(Icons.code, size: 16),
                  onPressed: () => _openGithub(context),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _ArchitectureColumn extends StatelessWidget {
  const _ArchitectureColumn({
    required this.title,
    required this.nodes,
    required this.icon,
  });

  final String title;
  final List<ArchitectureNode> nodes;
  final IconData icon;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;

    return GlowCard(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                icon,
                size: 20,
                color: isDark ? const Color(0xFFD7FF3F) : const Color(0xFF181818),
              ),
              const SizedBox(width: 10),
              Text(
                title,
                style: theme.textTheme.titleMedium?.copyWith(
                  fontWeight: FontWeight.w700,
                  color: colorScheme.onSurface,
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          ...nodes.map(
            (node) => Padding(
              padding: const EdgeInsets.symmetric(vertical: 8),
              child: Tooltip(
                message: node.tooltip,
                child: Row(
                  children: [
                    Container(
                      width: 8,
                      height: 8,
                      decoration: BoxDecoration(
                        shape: BoxShape.circle,
                        color: isDark ? const Color(0xFFD7FF3F) : const Color(0xFF181818),
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            node.title,
                            style: theme.textTheme.bodyMedium?.copyWith(
                              fontWeight: FontWeight.w600,
                              color: colorScheme.onSurface,
                            ),
                          ),
                          Text(
                            node.subtitle,
                            style: theme.textTheme.bodySmall?.copyWith(
                              color: colorScheme.onSurfaceVariant,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class SectionWrapper extends StatefulWidget {
  const SectionWrapper({
    super.key,
    required this.child,
    required this.scrollOffset,
  });

  final Widget child;
  final ValueListenable<double> scrollOffset;

  @override
  State<SectionWrapper> createState() => _SectionWrapperState();
}

class _SectionWrapperState extends State<SectionWrapper> {
  bool _visible = false;

  @override
  void initState() {
    super.initState();
    widget.scrollOffset.addListener(_handleScroll);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    WidgetsBinding.instance.addPostFrameCallback((_) => _evaluateVisibility());
  }

  @override
  void dispose() {
    widget.scrollOffset.removeListener(_handleScroll);
    super.dispose();
  }

  void _handleScroll() {
    if (!_visible) {
      _evaluateVisibility();
    }
  }

  void _evaluateVisibility() {
    if (!mounted || _visible) return;
    final box = context.findRenderObject() as RenderBox?;
    if (box == null) return;
    final offset = box.localToGlobal(Offset.zero).dy;
    final screenHeight = MediaQuery.of(context).size.height;
    if (offset < screenHeight * 0.95) {
      setState(() => _visible = true);
    }
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedOpacity(
      duration: const Duration(milliseconds: 500),
      opacity: _visible ? 1 : 0,
      child: widget.child,
    );
  }
}

class _SectionContainer extends StatelessWidget {
  const _SectionContainer({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Container(
      constraints: const BoxConstraints(maxWidth: 1040),
      padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 48),
      child: child,
    );
  }
}

void _copyCommand(BuildContext context, String value, String copiedToast) {
  Clipboard.setData(ClipboardData(text: value));
  _showToast(context, copiedToast);
}

const _githubUrl = "https://github.com/asodya/parallax";

Future<void> _openGithub(BuildContext context) async {
  final uri = Uri.parse(_githubUrl);
  final launched = await launchUrl(uri, mode: LaunchMode.externalApplication);
  if (!context.mounted) return;
  if (!launched) {
    _showToast(context, "Unable to open GitHub link.");
  }
}

void _showToast(BuildContext context, String message) {
  final isDark = Theme.of(context).brightness == Brightness.dark;
  ScaffoldMessenger.of(context).showSnackBar(
    SnackBar(
      content: Text(
        message,
        style: TextStyle(
          color: isDark ? Colors.black : Colors.white,
          fontWeight: FontWeight.w600,
        ),
      ),
      backgroundColor: isDark ? Colors.white : const Color(0xFF181818),
      behavior: SnackBarBehavior.floating,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
      duration: const Duration(seconds: 2),
    ),
  );
}
