class LandingData {
  const LandingData({
    required this.hero,
    required this.about,
    required this.architecture,
    required this.features,
    required this.steps,
    required this.gettingStarted,
    required this.protocol,
    required this.roadmap,
    required this.community,
    required this.finalCta,
  });

  final HeroContent hero;
  final AboutContent about;
  final ArchitectureContent architecture;
  final List<FeatureItem> features;
  final List<StepItem> steps;
  final List<CodeSample> gettingStarted;
  final ProtocolContent protocol;
  final RoadmapContent roadmap;
  final CommunityContent community;
  final FinalCtaContent finalCta;
}

class HeroContent {
  const HeroContent({
    required this.headline,
    required this.subHeadline,
    required this.primaryCta,
    required this.secondaryCta,
    required this.microText,
  });

  final String headline;
  final String subHeadline;
  final String primaryCta;
  final String secondaryCta;
  final String microText;
}

class AboutContent {
  const AboutContent({
    required this.title,
    required this.description,
    required this.details,
  });

  final String title;
  final String description;
  final List<String> details;
}

class ArchitectureContent {
  const ArchitectureContent({
    required this.title,
    required this.description,
    required this.hostNodes,
    required this.clientNodes,
    required this.flows,
  });

  final String title;
  final String description;
  final List<ArchitectureNode> hostNodes;
  final List<ArchitectureNode> clientNodes;
  final List<DataFlow> flows;
}

class ArchitectureNode {
  const ArchitectureNode({
    required this.title,
    required this.subtitle,
    required this.tooltip,
  });

  final String title;
  final String subtitle;
  final String tooltip;
}

class DataFlow {
  const DataFlow({
    required this.label,
    required this.direction,
  });

  final String label;
  final FlowDirection direction;
}

enum FlowDirection {
  hostToClient,
  biDirectional,
}

class FeatureItem {
  const FeatureItem({
    required this.title,
    required this.description,
    required this.icon,
  });

  final String title;
  final String description;
  final String icon;
}

class StepItem {
  const StepItem({
    required this.title,
    required this.description,
  });

  final String title;
  final String description;
}

class CodeSample {
  const CodeSample({
    required this.title,
    required this.command,
    required this.caption,
  });

  final String title;
  final String command;
  final String caption;
}

class ProtocolContent {
  const ProtocolContent({
    required this.title,
    required this.summary,
    required this.highlights,
    required this.payloadNotes,
  });

  final String title;
  final String summary;
  final List<String> highlights;
  final List<String> payloadNotes;
}

class RoadmapContent {
  const RoadmapContent({
    required this.title,
    required this.statusLines,
    required this.timeline,
  });

  final String title;
  final List<String> statusLines;
  final List<RoadmapItem> timeline;
}

class RoadmapItem {
  const RoadmapItem({
    required this.title,
    required this.description,
  });

  final String title;
  final String description;
}

class CommunityContent {
  const CommunityContent({
    required this.title,
    required this.description,
    required this.items,
  });

  final String title;
  final String description;
  final List<String> items;
}

class FinalCtaContent {
  const FinalCtaContent({
    required this.headline,
    required this.subHeadline,
    required this.primaryCta,
    required this.secondaryCta,
  });

  final String headline;
  final String subHeadline;
  final String primaryCta;
  final String secondaryCta;
}
