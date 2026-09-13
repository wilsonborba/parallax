import "package:flutter/material.dart";
import "package:url_launcher/url_launcher.dart";

import "../../l10n/app_localizations.dart";
import "glow_card.dart";
import "neon_button.dart";
import "section_header.dart";

class QuestShowcaseSection extends StatelessWidget {
  const QuestShowcaseSection({super.key, required this.l10n});

  final AppLocalizations? l10n;

  Future<void> _openUrl(BuildContext context, String url) async {
    final uri = Uri.parse(url);
    if (await canLaunchUrl(uri)) {
      await launchUrl(uri, mode: LaunchMode.externalApplication);
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;

    final title = l10n?.questSectionTitle ?? "Target Hardware: Meta Quest Ecosystem";
    final subtitle = l10n?.questSectionSubtitle ??
        "Engineered for Meta Quest 3, Quest 3S, and Android XR clients with low-latency UDP framing.";
    final badge = l10n?.questOptimizedBadge ?? "OPTIMIZED FOR META QUEST 3 & QUEST 3S";

    return Container(
      constraints: const BoxConstraints(maxWidth: 1040),
      padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 48),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionHeader(title: title, subtitle: subtitle),
          const SizedBox(height: 28),

          // Main Hero Showcase Banner
          GlowCard(
            padding: EdgeInsets.zero,
            child: ClipRRect(
              borderRadius: BorderRadius.circular(16),
              child: Stack(
                children: [
                  AspectRatio(
                    aspectRatio: 16 / 8,
                    child: Image.asset(
                      "assets/quest/quest_3s_feature.webp",
                      fit: BoxFit.cover,
                    ),
                  ),
                  Positioned.fill(
                    child: Container(
                      decoration: BoxDecoration(
                        gradient: LinearGradient(
                          begin: Alignment.topCenter,
                          end: Alignment.bottomCenter,
                          colors: [
                            Colors.transparent,
                            (isDark ? const Color(0xFF10110E) : Colors.black)
                                .withValues(alpha: 0.8),
                          ],
                          stops: const [0.4, 1.0],
                        ),
                      ),
                    ),
                  ),
                  Positioned(
                    bottom: 24,
                    left: 24,
                    right: 24,
                    child: LayoutBuilder(
                      builder: (context, bannerConstraints) {
                        final isNarrow = bannerConstraints.maxWidth < 420;

                        final badgeAndTitle = Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Container(
                              padding: const EdgeInsets.symmetric(
                                horizontal: 10,
                                vertical: 4,
                              ),
                              decoration: BoxDecoration(
                                color: isDark
                                    ? const Color(0xFFD7FF3F)
                                    : Colors.white,
                                borderRadius: BorderRadius.circular(6),
                              ),
                              child: Text(
                                badge,
                                style: theme.textTheme.labelSmall?.copyWith(
                                  color: Colors.black,
                                  fontWeight: FontWeight.w900,
                                  letterSpacing: 1.1,
                                  fontSize: 10,
                                ),
                              ),
                            ),
                            const SizedBox(height: 8),
                            Text(
                              "Meta Quest 3S & Quest 3",
                              style: theme.textTheme.titleLarge?.copyWith(
                                fontWeight: FontWeight.w800,
                                color: Colors.white,
                                letterSpacing: -0.5,
                              ),
                            ),
                          ],
                        );

                        final specsButton = NeonButton(
                          label: l10n?.questLinkSpecs ?? "Meta Quest 3S",
                          icon: const Icon(Icons.open_in_new, size: 14),
                          onPressed: () => _openUrl(
                            context,
                            "https://www.meta.com/quest/quest-3s/",
                          ),
                        );

                        if (isNarrow) {
                          return Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              badgeAndTitle,
                              const SizedBox(height: 12),
                              specsButton,
                            ],
                          );
                        }

                        return Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          crossAxisAlignment: CrossAxisAlignment.end,
                          children: [
                            Flexible(child: badgeAndTitle),
                            specsButton,
                          ],
                        );
                      },
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 24),

          // 3 Column Hardware Highlights
          LayoutBuilder(
            builder: (context, constraints) {
              final isNarrow = constraints.maxWidth < 840;
              final cardWidth = isNarrow
                  ? constraints.maxWidth
                  : (constraints.maxWidth - 32) / 3;

              final cards = [
                (
                  image: "assets/quest/quest_3s_headset.webp",
                  title: l10n?.questCard1Title ?? "Meta Quest 3S Wireless Pipeline",
                  desc: l10n?.questCard1Desc ??
                      "Stream full-resolution Linux X11 desktops over Wi-Fi 6E with sub-20ms latency.",
                  url: "https://www.meta.com/quest/quest-3s/",
                ),
                (
                  image: "assets/quest/quest_spatial_view.webp",
                  title: l10n?.questCard2Title ?? "Spatial Room-Scale Workspace",
                  desc: l10n?.questCard2Desc ??
                      "Transform your terminal and IDE into floating spatial displays anywhere.",
                  url: "https://developer.oculus.com/documentation/native/android/",
                ),
                (
                  image: "assets/quest/quest_immersion.webp",
                  title: l10n?.questCard3Title ?? "Hardware-Accelerated Decoding",
                  desc: l10n?.questCard3Desc ??
                      "Snapdragon XR2 Gen 2 native H.264 decode with zero battery-draining emulation.",
                  url: "https://developer.oculus.com/",
                ),
              ];

              return Wrap(
                spacing: 16,
                runSpacing: 16,
                children: cards.map((c) {
                  return SizedBox(
                    width: cardWidth,
                    child: GlowCard(
                      padding: EdgeInsets.zero,
                      onTap: () => _openUrl(context, c.url),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          ClipRRect(
                            borderRadius: const BorderRadius.vertical(
                              top: Radius.circular(16),
                            ),
                            child: AspectRatio(
                              aspectRatio: 16 / 9,
                              child: Image.asset(c.image, fit: BoxFit.cover),
                            ),
                          ),
                          Padding(
                            padding: const EdgeInsets.all(18),
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  c.title,
                                  style: theme.textTheme.titleSmall?.copyWith(
                                    fontWeight: FontWeight.w700,
                                    color: colorScheme.onSurface,
                                  ),
                                ),
                                const SizedBox(height: 6),
                                Text(
                                  c.desc,
                                  style: theme.textTheme.bodySmall?.copyWith(
                                    color: colorScheme.onSurfaceVariant,
                                    height: 1.45,
                                  ),
                                ),
                              ],
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
          const SizedBox(height: 24),

          // Helpful Developer & Ecosystem Links
          Wrap(
            spacing: 12,
            runSpacing: 10,
            children: [
              _LinkChip(
                label: l10n?.questLinkSpecs ?? "Meta Quest 3S Official",
                url: "https://www.meta.com/quest/quest-3s/",
                icon: Icons.launch,
              ),
              _LinkChip(
                label: l10n?.questLinkDev ?? "Meta Quest Developer Center",
                url: "https://developer.oculus.com/",
                icon: Icons.developer_board,
              ),
              _LinkChip(
                label: l10n?.questLinkAdb ?? "Sideloading & ADB Setup",
                url:
                    "https://developer.oculus.com/documentation/native/android/mobile-device-setup/",
                icon: Icons.install_desktop,
              ),
              _LinkChip(
                label: l10n?.questLinkSidequest ?? "SideQuest Community",
                url: "https://sidequestvr.com/",
                icon: Icons.view_in_ar,
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _LinkChip extends StatelessWidget {
  const _LinkChip({
    required this.label,
    required this.url,
    required this.icon,
  });

  final String label;
  final String url;
  final IconData icon;

  Future<void> _launch() async {
    final uri = Uri.parse(url);
    if (await canLaunchUrl(uri)) {
      await launchUrl(uri, mode: LaunchMode.externalApplication);
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;

    return ActionChip(
      onPressed: _launch,
      avatar: Icon(icon, size: 14, color: colorScheme.onSurface),
      label: Text(label),
      backgroundColor: isDark ? const Color(0xFF191A16) : const Color(0xFFF3F4F6),
      side: BorderSide(
        color: isDark ? const Color(0xFF262626) : const Color(0xFFE5E7EB),
      ),
      labelStyle: theme.textTheme.bodySmall?.copyWith(
        fontWeight: FontWeight.w600,
        color: colorScheme.onSurface,
      ),
    );
  }
}
