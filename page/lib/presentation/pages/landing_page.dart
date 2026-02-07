import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../domain/entities/landing_data.dart';
import '../viewmodels/landing_view_model.dart';
import '../widgets/animated_background.dart';
import '../widgets/glow_card.dart';
import '../widgets/gradient_text.dart';
import '../widgets/neon_button.dart';
import '../widgets/section_header.dart';

class LandingPage extends StatefulWidget {
  const LandingPage({super.key, required this.viewModel});

  final LandingViewModel viewModel;

  @override
  State<LandingPage> createState() => _LandingPageState();
}

class _LandingPageState extends State<LandingPage> {
  late final ScrollController _scrollController;
  final ValueNotifier<double> _scrollOffset = ValueNotifier<double>(0);

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

  @override
  Widget build(BuildContext context) {
    final data = widget.viewModel.landingData;

    return Scaffold(
      body: Stack(
        children: [
          Positioned.fill(
            child: AnimatedBackground(scrollOffset: _scrollOffset),
          ),
          Positioned.fill(
            child: SingleChildScrollView(
              controller: _scrollController,
              child: Column(
                children: [
                  HeroSection(data: data.hero),
                  SectionWrapper(
                    scrollOffset: _scrollOffset,
                    child: AboutSection(data: data.about),
                  ),
                  SectionWrapper(
                    scrollOffset: _scrollOffset,
                    child: ArchitectureSection(data: data.architecture),
                  ),
                  SectionWrapper(
                    scrollOffset: _scrollOffset,
                    child: FeaturesSection(items: data.features),
                  ),
                  SectionWrapper(
                    scrollOffset: _scrollOffset,
                    child: HowItWorksSection(steps: data.steps),
                  ),
                  SectionWrapper(
                    scrollOffset: _scrollOffset,
                    child: GettingStartedSection(samples: data.gettingStarted),
                  ),
                  SectionWrapper(
                    scrollOffset: _scrollOffset,
                    child: ProtocolSection(data: data.protocol),
                  ),
                  SectionWrapper(
                    scrollOffset: _scrollOffset,
                    child: RoadmapSection(data: data.roadmap),
                  ),
                  SectionWrapper(
                    scrollOffset: _scrollOffset,
                    child: CommunitySection(data: data.community),
                  ),
                  SectionWrapper(
                    scrollOffset: _scrollOffset,
                    child: FinalCtaSection(data: data.finalCta),
                  ),
                  const SizedBox(height: 80),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class HeroSection extends StatelessWidget {
  const HeroSection({super.key, required this.data});

  final HeroContent data;

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 48),
      child: Column(
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Row(
                children: [
                  const Icon(Icons.blur_on, color: Color(0xFFB86CFF)),
                  const SizedBox(width: 12),
                  Text(
                    'Parallax',
                    style: Theme.of(context).textTheme.titleLarge?.copyWith(
                          fontWeight: FontWeight.w700,
                        ),
                  ),
                ],
              ),
              Row(
                children: [
                  TextButton(
                    onPressed: () => _showToast(context, 'Docs coming soon.'),
                    child: const Text('Docs'),
                  ),
                  const SizedBox(width: 12),
                  TextButton(
                    onPressed: () => _showToast(context, 'GitHub link coming soon.'),
                    child: const Text('GitHub'),
                  ),
                ],
              ),
            ],
          ),
          const SizedBox(height: 80),
          Container(
            constraints: const BoxConstraints(maxWidth: 920),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                GradientText(
                  'Parallax',
                  gradient: LinearGradient(
                    colors: [
                      colorScheme.primary,
                      colorScheme.secondary,
                      Colors.white,
                    ],
                  ),
                  style: Theme.of(context).textTheme.displayLarge?.copyWith(
                        fontWeight: FontWeight.w800,
                        letterSpacing: -1.2,
                      ),
                ),
                const SizedBox(height: 24),
                Text(
                  data.headline,
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                        fontWeight: FontWeight.w700,
                        height: 1.2,
                      ),
                ),
                const SizedBox(height: 16),
                Text(
                  data.subHeadline,
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        color: Colors.white70,
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
                      label: data.primaryCta,
                      onPressed: () => _showToast(context, 'Getting started soon.'),
                    ),
                    NeonButton(
                      label: data.secondaryCta,
                      onPressed: () => _showToast(context, 'GitHub link coming soon.'),
                      isPrimary: false,
                    ),
                  ],
                ),
                const SizedBox(height: 18),
                Text(
                  data.microText,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Colors.white54,
                        letterSpacing: 0.4,
                      ),
                ),
                const SizedBox(height: 60),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 16),
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(24),
                    border: Border.all(color: Colors.white10),
                    color: Colors.white.withOpacity(0.04),
                  ),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.waves, color: colorScheme.primary),
                      const SizedBox(width: 12),
                      Text(
                        'Ultra-low latency signal path with adaptive UDP framing',
                        style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                              color: Colors.white70,
                            ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 80),
          SizedBox(
            height: 180,
            child: Stack(
              children: [
                Positioned.fill(
                  child: AnimatedGradientPanel(color: colorScheme.primary),
                ),
                Align(
                  alignment: Alignment.center,
                  child: Text(
                    'Latency · Fidelity · Immersion',
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          color: Colors.white70,
                          letterSpacing: 2,
                        ),
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

class AboutSection extends StatelessWidget {
  const AboutSection({super.key, required this.data});

  final AboutContent data;

  @override
  Widget build(BuildContext context) {
    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(title: data.title, subtitle: data.description),
          const SizedBox(height: 28),
          Wrap(
            spacing: 16,
            runSpacing: 16,
            children: data.details
                .map(
                  (detail) => GlowCard(
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        const Icon(Icons.blur_on, color: Color(0xFFB86CFF)),
                        const SizedBox(width: 12),
                        Flexible(
                          child: Text(
                            detail,
                            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                                  color: Colors.white70,
                                  height: 1.4,
                                ),
                          ),
                        ),
                      ],
                    ),
                  ),
                )
                .toList(),
          ),
          const SizedBox(height: 32),
          GlowCard(
            padding: const EdgeInsets.all(24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Experimental pipeline diagram',
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.w600,
                      ),
                ),
                const SizedBox(height: 16),
                SizedBox(
                  height: 160,
                  child: Stack(
                    children: [
                      Positioned.fill(
                        child: AnimatedGradientPanel(
                          color: Theme.of(context).colorScheme.secondary,
                        ),
                      ),
                      Align(
                        alignment: Alignment.center,
                        child: Row(
                          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                          children: const [
                            _DiagramNode(label: 'X11 Capture'),
                            _DiagramNode(label: 'H.264 Encode'),
                            _DiagramNode(label: 'UDP Stream'),
                            _DiagramNode(label: 'VR Client'),
                          ],
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

class ArchitectureSection extends StatelessWidget {
  const ArchitectureSection({super.key, required this.data});

  final ArchitectureContent data;

  @override
  Widget build(BuildContext context) {
    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(title: data.title, subtitle: data.description),
          const SizedBox(height: 28),
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(
                child: _ArchitectureColumn(title: 'Linux Host (Rust)', nodes: data.hostNodes),
              ),
              const SizedBox(width: 24),
              Expanded(
                child: _ArchitectureColumn(title: 'Android Client', nodes: data.clientNodes),
              ),
            ],
          ),
          const SizedBox(height: 24),
          Wrap(
            spacing: 12,
            children: data.flows
                .map(
                  (flow) => Chip(
                    label: Text(flow.label),
                    avatar: Icon(
                      flow.direction == FlowDirection.biDirectional
                          ? Icons.sync_alt
                          : Icons.arrow_forward,
                      size: 18,
                      color: Theme.of(context).colorScheme.primary,
                    ),
                    backgroundColor: Colors.white.withOpacity(0.08),
                    labelStyle: const TextStyle(color: Colors.white70),
                  ),
                )
                .toList(),
          ),
        ],
      ),
    );
  }
}

class FeaturesSection extends StatelessWidget {
  const FeaturesSection({super.key, required this.items});

  final List<FeatureItem> items;

  @override
  Widget build(BuildContext context) {
    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(
            title: 'Core features',
            subtitle: 'High-performance streaming primitives designed for VR latency budgets.',
          ),
          const SizedBox(height: 28),
          LayoutBuilder(
            builder: (context, constraints) {
              final width = constraints.maxWidth;
              final crossAxisCount = width > 1000 ? 3 : width > 700 ? 2 : 1;
              return Wrap(
                spacing: 20,
                runSpacing: 20,
                children: items
                    .map(
                      (item) => SizedBox(
                        width: width / crossAxisCount - 20,
                        child: GlowCard(
                          onTap: () => _showToast(context, '${item.title} selected.'),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(item.icon, style: const TextStyle(fontSize: 28)),
                              const SizedBox(height: 16),
                              Text(
                                item.title,
                                style: Theme.of(context).textTheme.titleMedium?.copyWith(
                                      fontWeight: FontWeight.w600,
                                    ),
                              ),
                              const SizedBox(height: 8),
                              Text(
                                item.description,
                                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                                      color: Colors.white70,
                                      height: 1.5,
                                    ),
                              ),
                            ],
                          ),
                        ),
                      ),
                    )
                    .toList(),
              );
            },
          ),
        ],
      ),
    );
  }
}

class HowItWorksSection extends StatefulWidget {
  const HowItWorksSection({super.key, required this.steps});

  final List<StepItem> steps;

  @override
  State<HowItWorksSection> createState() => _HowItWorksSectionState();
}

class _HowItWorksSectionState extends State<HowItWorksSection> {
  int _selectedIndex = 0;

  @override
  Widget build(BuildContext context) {
    final selected = widget.steps[_selectedIndex];
    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(
            title: 'How it works',
            subtitle: 'Follow the end-to-end flow from Linux capture to immersive playback.',
          ),
          const SizedBox(height: 24),
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: widget.steps.asMap().entries.map((entry) {
              final index = entry.key;
              final step = entry.value;
              final isSelected = index == _selectedIndex;
              return ChoiceChip(
                label: Text('Step ${index + 1}'),
                selected: isSelected,
                onSelected: (_) => setState(() => _selectedIndex = index),
                selectedColor: Theme.of(context).colorScheme.primary.withOpacity(0.4),
                backgroundColor: Colors.white10,
                labelStyle: TextStyle(color: isSelected ? Colors.white : Colors.white70),
              );
            }).toList(),
          ),
          const SizedBox(height: 20),
          GlowCard(
            child: Row(
              children: [
                Container(
                  width: 52,
                  height: 52,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    gradient: LinearGradient(
                      colors: [
                        Theme.of(context).colorScheme.primary,
                        Theme.of(context).colorScheme.secondary,
                      ],
                    ),
                  ),
                  child: Center(
                    child: Text(
                      '${_selectedIndex + 1}',
                      style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
                    ),
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        selected.title,
                        style: Theme.of(context).textTheme.titleMedium?.copyWith(
                              fontWeight: FontWeight.w600,
                            ),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        selected.description,
                        style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                              color: Colors.white70,
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
  const GettingStartedSection({super.key, required this.samples});

  final List<CodeSample> samples;

  @override
  Widget build(BuildContext context) {
    return _SectionContainer(
      child: LayoutBuilder(
        builder: (context, constraints) {
          final cardWidth = min(420.0, constraints.maxWidth);
          return Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              SectionHeader(
                title: 'Getting started',
                subtitle: 'Developer-friendly setup commands with instant copy actions.',
              ),
              const SizedBox(height: 24),
              Wrap(
                spacing: 20,
                runSpacing: 20,
                children: samples
                    .map(
                      (sample) => SizedBox(
                        width: cardWidth,
                        child: GlowCard(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Row(
                                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                                children: [
                                  Text(
                                    sample.title,
                                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                                          fontWeight: FontWeight.w600,
                                        ),
                                  ),
                                  IconButton(
                                    onPressed: () => _copyCommand(context, sample.command),
                                    icon: const Icon(Icons.copy, size: 18),
                                    tooltip: 'Copy command',
                                  ),
                                ],
                              ),
                              const SizedBox(height: 12),
                              Container(
                                padding: const EdgeInsets.all(16),
                                decoration: BoxDecoration(
                                  color: Colors.black.withOpacity(0.3),
                                  borderRadius: BorderRadius.circular(16),
                                  border: Border.all(color: Colors.white10),
                                ),
                                child: Text(
                                  sample.command,
                                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                                        fontFamily: 'monospace',
                                        color: Colors.white70,
                                        height: 1.5,
                                      ),
                                ),
                              ),
                              const SizedBox(height: 12),
                              Text(
                                sample.caption,
                                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                                      color: Colors.white60,
                                    ),
                              ),
                            ],
                          ),
                        ),
                      ),
                    )
                    .toList(),
              ),
              const SizedBox(height: 12),
              Text(
                'Scan the QR code from the host UI and connect from your Android device.',
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(color: Colors.white70),
              ),
            ],
          );
        },
      ),
    );
  }
}

class ProtocolSection extends StatelessWidget {
  const ProtocolSection({super.key, required this.data});

  final ProtocolContent data;

  @override
  Widget build(BuildContext context) {
    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(title: data.title, subtitle: data.summary),
          const SizedBox(height: 24),
          GlowCard(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Packet format highlights',
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.w600,
                      ),
                ),
                const SizedBox(height: 12),
                ...data.highlights.map(
                  (item) => Padding(
                    padding: const EdgeInsets.only(bottom: 8),
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Icon(Icons.fiber_manual_record, size: 10, color: Colors.white54),
                        const SizedBox(width: 12),
                        Expanded(
                          child: Text(
                            item,
                            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                                  color: Colors.white70,
                                  height: 1.4,
                                ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(height: 16),
                Text(
                  'Payload notes',
                  style: Theme.of(context).textTheme.titleSmall?.copyWith(
                        fontWeight: FontWeight.w600,
                      ),
                ),
                const SizedBox(height: 8),
                ...data.payloadNotes.map(
                  (item) => Padding(
                    padding: const EdgeInsets.only(bottom: 6),
                    child: Text(
                      item,
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: Colors.white60,
                          ),
                    ),
                  ),
                ),
                const SizedBox(height: 16),
                ExpansionTile(
                  title: const Text('Read full protocol'),
                  collapsedIconColor: Colors.white54,
                  iconColor: Theme.of(context).colorScheme.primary,
                  children: [
                    Padding(
                      padding: const EdgeInsets.only(bottom: 12),
                      child: Text(
                        'See proto/README.md for the full header layout, flag definitions, and MTU guidance.',
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                              color: Colors.white60,
                            ),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class RoadmapSection extends StatelessWidget {
  const RoadmapSection({super.key, required this.data});

  final RoadmapContent data;

  @override
  Widget build(BuildContext context) {
    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(title: data.title, subtitle: data.statusLines.join(' ')),
          const SizedBox(height: 24),
          Wrap(
            spacing: 20,
            runSpacing: 20,
            children: data.timeline
                .map(
                  (item) => SizedBox(
                    width: 320,
                    child: GlowCard(
                      onTap: () => _showToast(context, '${item.title} roadmap item.'),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            item.title,
                            style: Theme.of(context).textTheme.titleMedium?.copyWith(
                                  fontWeight: FontWeight.w600,
                                ),
                          ),
                          const SizedBox(height: 8),
                          Text(
                            item.description,
                            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                                  color: Colors.white70,
                                  height: 1.4,
                                ),
                          ),
                        ],
                      ),
                    ),
                  ),
                )
                .toList(),
          ),
        ],
      ),
    );
  }
}

class CommunitySection extends StatelessWidget {
  const CommunitySection({super.key, required this.data});

  final CommunityContent data;

  @override
  Widget build(BuildContext context) {
    return _SectionContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(title: data.title, subtitle: data.description),
          const SizedBox(height: 20),
          Wrap(
            spacing: 12,
            children: data.items
                .map(
                  (item) => ActionChip(
                    label: Text(item),
                    onPressed: () => _showToast(context, '$item selected.'),
                    backgroundColor: Colors.white10,
                    labelStyle: const TextStyle(color: Colors.white70),
                  ),
                )
                .toList(),
          ),
        ],
      ),
    );
  }
}

class FinalCtaSection extends StatelessWidget {
  const FinalCtaSection({super.key, required this.data});

  final FinalCtaContent data;

  @override
  Widget build(BuildContext context) {
    return _SectionContainer(
      child: GlowCard(
        padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 36),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Text(
              data.headline,
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                    fontWeight: FontWeight.w700,
                  ),
            ),
            const SizedBox(height: 12),
            Text(
              data.subHeadline,
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                    color: Colors.white70,
                  ),
            ),
            const SizedBox(height: 24),
            Wrap(
              spacing: 16,
              children: [
                NeonButton(
                  label: data.primaryCta,
                  onPressed: () => _showToast(context, 'Get started soon.'),
                ),
                NeonButton(
                  label: data.secondaryCta,
                  isPrimary: false,
                  onPressed: () => _showToast(context, 'GitHub link coming soon.'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class AnimatedGradientPanel extends StatefulWidget {
  const AnimatedGradientPanel({super.key, required this.color});

  final Color color;

  @override
  State<AnimatedGradientPanel> createState() => _AnimatedGradientPanelState();
}

class _AnimatedGradientPanelState extends State<AnimatedGradientPanel>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 6),
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        return Container(
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(32),
            gradient: LinearGradient(
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
              colors: [
                widget.color.withOpacity(0.7 + 0.3 * _controller.value),
                const Color(0xFF0F0A1A),
                widget.color.withOpacity(0.2),
              ],
            ),
          ),
          child: child,
        );
      },
    );
  }
}

class _DiagramNode extends StatelessWidget {
  const _DiagramNode({required this.label});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 36,
          height: 36,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: Theme.of(context).colorScheme.primary.withOpacity(0.8),
            boxShadow: [
              BoxShadow(
                color: Theme.of(context).colorScheme.primary.withOpacity(0.5),
                blurRadius: 16,
              ),
            ],
          ),
        ),
        const SizedBox(height: 8),
        Text(
          label,
          style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.white70),
        ),
      ],
    );
  }
}

class _ArchitectureColumn extends StatelessWidget {
  const _ArchitectureColumn({required this.title, required this.nodes});

  final String title;
  final List<ArchitectureNode> nodes;

  @override
  Widget build(BuildContext context) {
    return GlowCard(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: Theme.of(context).textTheme.titleMedium?.copyWith(
                  fontWeight: FontWeight.w600,
                ),
          ),
          const SizedBox(height: 12),
          ...nodes.map(
            (node) => Padding(
              padding: const EdgeInsets.symmetric(vertical: 8),
              child: Tooltip(
                message: node.tooltip,
                child: Row(
                  children: [
                    const Icon(Icons.hexagon, size: 18, color: Color(0xFFB86CFF)),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            node.title,
                            style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                                  fontWeight: FontWeight.w600,
                                ),
                          ),
                          Text(
                            node.subtitle,
                            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                                  color: Colors.white60,
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
  const SectionWrapper({super.key, required this.child, required this.scrollOffset});

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
    if (offset < screenHeight * 0.9) {
      setState(() => _visible = true);
    }
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedOpacity(
      duration: const Duration(milliseconds: 700),
      opacity: _visible ? 1 : 0,
      child: AnimatedSlide(
        duration: const Duration(milliseconds: 700),
        offset: _visible ? Offset.zero : const Offset(0, 0.06),
        child: widget.child,
      ),
    );
  }
}

class _SectionContainer extends StatelessWidget {
  const _SectionContainer({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 48, vertical: 56),
      child: child,
    );
  }
}

void _copyCommand(BuildContext context, String value) {
  Clipboard.setData(ClipboardData(text: value));
  _showToast(context, 'Command copied to clipboard.');
}

void _showToast(BuildContext context, String message) {
  ScaffoldMessenger.of(context).showSnackBar(
    SnackBar(
      content: Text(message),
      backgroundColor: const Color(0xFF1A1425),
      behavior: SnackBarBehavior.floating,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
    ),
  );
}
