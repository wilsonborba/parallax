// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Portuguese (`pt`).
class AppLocalizationsPt extends AppLocalizations {
  AppLocalizationsPt([String locale = 'pt']) : super(locale);

  @override
  String get navBrand => 'PARALLAX';

  @override
  String get navBadge => 'EXPERIMENTAL v0.1';

  @override
  String get navArchitecture => 'Arquitetura';

  @override
  String get navFeatures => 'Recursos';

  @override
  String get navHowItWorks => 'Como Funciona';

  @override
  String get navGetStarted => 'Começar';

  @override
  String get navProtocol => 'Protocolo';

  @override
  String get navRoadmap => 'Roadmap';

  @override
  String get navThemeLight => 'Modo Claro';

  @override
  String get navThemeDark => 'Modo Escuro';

  @override
  String get navLanguage => 'Idioma';

  @override
  String get navGithub => 'GitHub';

  @override
  String get heroHeadline =>
      'Transmita seu desktop Linux para VR com latência ultrabaixa.';

  @override
  String get heroSubHeadline =>
      'Código aberto. Experimental. Criado para o futuro da computação espacial.';

  @override
  String get heroPrimaryCta => 'Começar';

  @override
  String get heroSecondaryCta => 'Ver no GitHub';

  @override
  String get heroMicroText =>
      'Atualmente em estágio inicial de desenvolvimento (v0.1).';

  @override
  String get aboutTitle => 'O que é o Parallax?';

  @override
  String get aboutDescription =>
      'O Parallax é um pipeline experimental de renderização remota para transmitir um desktop Linux para dispositivos clientes focado em realidade virtual/aumentada (VR/AR). Ele captura telas X11, codifica em H.264, transmite via UDP e coordena sessões por um canal de controle TCP leve.';

  @override
  String get aboutDetail1 =>
      'Host Linux captura display X11 para streaming em VR em tempo real.';

  @override
  String get aboutDetail2 =>
      'Quadros codificados em H.264 via aceleração de hardware (VAAPI) ou software.';

  @override
  String get aboutDetail3 =>
      'Streaming de vídeo via UDP para entrega com latência ultrabaixa.';

  @override
  String get aboutDetail4 =>
      'Canal de controle TCP gerencia tokens de pareamento e configuração de sessão.';

  @override
  String get aboutDetail5 =>
      'Em evolução contínua — contribuições da comunidade são bem-vindas.';

  @override
  String get architectureTitle => 'Visão Geral da Arquitetura';

  @override
  String get architectureSubtitle =>
      'Pipeline minimalista de baixa latência entre daemon Linux host e cliente Android com controle bidirecional e streaming de vídeo unidirecional.';

  @override
  String get archHostTitle => 'Host Linux (Rust)';

  @override
  String get archHostSubtitle => 'prlx-hostd';

  @override
  String get archHostTooltip =>
      'Captura quadros X11, codifica H.264 e gerencia sessões de controle.';

  @override
  String get archX11Title => 'Captura X11';

  @override
  String get archX11Subtitle => 'Feed de exibição';

  @override
  String get archX11Tooltip =>
      'Captura quadros do desktop Linux diretamente do servidor X11.';

  @override
  String get archEncodeTitle => 'Codificação H.264';

  @override
  String get archEncodeSubtitle => 'VAAPI ou software';

  @override
  String get archEncodeTooltip =>
      'Codifica quadros em fluxo de bytes H.264 Annex B.';

  @override
  String get archUdpStreamTitle => 'Stream de Vídeo UDP';

  @override
  String get archUdpStreamSubtitle => 'Enquadramento leve';

  @override
  String get archUdpStreamTooltip =>
      'Envia pacotes UDP com cabeçalho compacto de 24 bytes.';

  @override
  String get archAndroidTitle => 'Cliente Android';

  @override
  String get archAndroidSubtitle => 'Parallax Receiver';

  @override
  String get archAndroidTooltip =>
      'App Jetpack Compose para reprodução em VR/AR.';

  @override
  String get archQrTitle => 'Pareamento QR';

  @override
  String get archQrSubtitle => 'Fluxo por token';

  @override
  String get archQrTooltip =>
      'Escaneia código QR para conexão segura ao canal de controle.';

  @override
  String get archTcpTitle => 'Controle TCP';

  @override
  String get archTcpSubtitle => 'Broker de sessão';

  @override
  String get archTcpTooltip =>
      'Coordena criação de sessão, pareamento e configuração.';

  @override
  String get archDecodeTitle => 'Decodificação H.264';

  @override
  String get archDecodeSubtitle => 'UI de reprodução';

  @override
  String get archDecodeTooltip =>
      'Decodifica quadros H.264 para renderização imersiva.';

  @override
  String get archFlowTcp => 'Controle TCP (bidirecional)';

  @override
  String get archFlowUdp => 'Stream de vídeo UDP (host → cliente)';

  @override
  String get featuresTitle => 'Recursos principais';

  @override
  String get featuresSubtitle =>
      'Primitivas de streaming de alta performance desenhadas para os limites de latência em VR.';

  @override
  String get featUdpTitle => 'Streaming UDP de baixa latência';

  @override
  String get featUdpDesc =>
      'Protocolo de enquadramento minimalista para máxima velocidade e resiliência.';

  @override
  String get featPairingTitle => 'Pareamento seguro';

  @override
  String get featPairingDesc =>
      'Canal de controle protegido por tokens de pareamento e fluxo QR.';

  @override
  String get featProtocolTitle => 'Protocolo aberto';

  @override
  String get featProtocolDesc =>
      'Estrutura de pacotes UDP documentada e totalmente extensível.';

  @override
  String get featHostUiTitle => 'Interface do Host';

  @override
  String get featHostUiDesc =>
      'Interface desktop para pareamento e controle de sessões ativas.';

  @override
  String get featAndroidClientTitle => 'Cliente Android';

  @override
  String get featAndroidClientDesc =>
      'App Jetpack Compose que escaneia QR para conectar instantaneamente.';

  @override
  String get howItWorksTitle => 'Como funciona';

  @override
  String get howItWorksSubtitle =>
      'Da tela do desktop aos óculos VR em cinco etapas sincronizadas de pipeline.';

  @override
  String get step1Title => 'Captura da tela X11';

  @override
  String get step1Desc =>
      'O prlx-hostd captura os quadros da área de trabalho diretamente do X11.';

  @override
  String get step2Title => 'Codificação em H.264';

  @override
  String get step2Desc =>
      'Codificação por hardware VAAPI ou fallback por software para portabilidade.';

  @override
  String get step3Title => 'Empacotamento via Parallax';

  @override
  String get step3Desc =>
      'Quadros divididos em pacotes UDP com metadados de fluxo e sincronização.';

  @override
  String get step4Title => 'Coordenação via TCP';

  @override
  String get step4Desc =>
      'Token de pareamento e comandos de controle trafegam por TCP.';

  @override
  String get step5Title => 'Decodificação no Android';

  @override
  String get step5Desc =>
      'Cliente Jetpack Compose escaneia QR e renderiza o fluxo em tempo real.';

  @override
  String get gettingStartedTitle => 'Primeiros passos';

  @override
  String get gettingStartedSubtitle =>
      'Escolha um método de instalação para executar o Parallax no seu host Linux.';

  @override
  String get sample1Title => 'Instalação em comando único (Debian/Ubuntu)';

  @override
  String get sample1Caption =>
      'Instala dependências, binários, comando CLI e inicializador desktop.';

  @override
  String get sample2Title => 'URL do instalador no Cloudflare Pages';

  @override
  String get sample2Caption =>
      'Instalador publicado como asset estático em parallax.asodya.com.';

  @override
  String get sample3Title => 'Instalação via Cargo';

  @override
  String get sample3Caption =>
      'Alternativa para desenvolvedores Rust usando o Cargo.';

  @override
  String get sample4Title => 'Instalação manual pelo repositório';

  @override
  String get sample4Caption =>
      'Alternativa manual para quem deseja controle total passo a passo.';

  @override
  String get copyCommand => 'Copiar comando';

  @override
  String get commandCopied => 'Copiado para a área de transferência';

  @override
  String get protocolTitle => 'Protocolo: Enquadramento de pacotes UDP';

  @override
  String get protocolSubtitle =>
      'O Parallax adota um cabeçalho fixo de 24 bytes com magic value \"PRLX\", identificadores de stream/frame e flags de keyframe, config e fim de frame.';

  @override
  String get protocolHighlight1 =>
      'Cabeçalho fixo de 24 bytes em ordem de rede big-endian.';

  @override
  String get protocolHighlight2 =>
      'Magic value \"PRLX\" valida integridade do frame.';

  @override
  String get protocolHighlight3 =>
      'Stream ID + Frame ID para remontagem e sincronismo.';

  @override
  String get protocolHighlight4 =>
      'Flags de controle para keyframe, config e marcadores de término.';

  @override
  String get protocolHighlight5 =>
      'Tipos de carga útil para vídeo, áudio e dados de controle.';

  @override
  String get protocolPayloadNotesTitle => 'Notas sobre payload';

  @override
  String get protocolNote1 =>
      'Payloads H.264 operam em formato Annex B delimitados por start-code.';

  @override
  String get protocolNote2 =>
      'SPS/PPS podem ser enviados em pacotes de configuração ou com keyframes.';

  @override
  String get roadmapTitle => 'Status do projeto e roadmap';

  @override
  String get roadmapSubtitle =>
      'Versão experimental inicial (v0.1). Desenvolvimento ativo para otimização de latência.';

  @override
  String get roadmapStatus1 => 'Versão experimental inicial (v0.1).';

  @override
  String get roadmapStatus2 =>
      'Desenvolvimento ativo para otimização de estabilidade e latência.';

  @override
  String get roadmapStatus3 =>
      'Contribuições da comunidade são muito bem-vindas.';

  @override
  String get roadmapItem1Title => 'Otimização de latência';

  @override
  String get roadmapItem1Desc =>
      'Redução de tempos do ciclo captura → codificação → transporte.';

  @override
  String get roadmapItem2Title => 'Aprimoramento de aceleração de hardware';

  @override
  String get roadmapItem2Desc =>
      'Ampliação de suporte a encoders VAAPI e detecção inteligente de GPU.';

  @override
  String get roadmapItem3Title => 'Expansão de plataformas clientes';

  @override
  String get roadmapItem3Desc =>
      'Desenvolvimento de clientes para desktop e headsets VR adicionais.';

  @override
  String get roadmapItem4Title => 'Renderização nativa em VR';

  @override
  String get roadmapItem4Desc =>
      'Integração imersiva de controles e renderização espacial direta.';

  @override
  String get communityTitle => 'Código aberto e comunidade';

  @override
  String get communityDescription =>
      'O Parallax possui licença MIT com foco em pesquisa e inovação. Participe com sugestões, issues e pull requests no GitHub.';

  @override
  String get communityItem1 => 'Licença MIT permissiva';

  @override
  String get communityItem2 => 'Documentação de protocolo aberta';

  @override
  String get communityItem3 => 'Discussions e issues ativas no GitHub';

  @override
  String get finalCtaHeadline => 'Construa o futuro do streaming para VR.';

  @override
  String get finalCtaSubHeadline =>
      'O Parallax é código aberto e está em constante evolução — junte-se à comunidade.';

  @override
  String get footerCopyright => '© 2026 Asodya. Todos os direitos reservados.';

  @override
  String get questSectionTitle => 'Dispositivos Alvo: Ecossistema Meta Quest';

  @override
  String get questSectionSubtitle =>
      'Projetado para Meta Quest 3, Quest 3S e clientes Android XR com streaming UDP de baixa latência e alta taxa de quadros.';

  @override
  String get questOptimizedBadge => 'OTIMIZADO PARA META QUEST 3 & QUEST 3S';

  @override
  String get questCard1Title => 'Pipeline Sem Fio no Meta Quest 3S';

  @override
  String get questCard1Desc =>
      'Transmita a área de trabalho Linux em resolução máxima via Wi-Fi 6E com latência ultrabaixa.';

  @override
  String get questCard2Title => 'Espaço de Trabalho Espacial Imersivo';

  @override
  String get questCard2Desc =>
      'Transforme seu terminal, IDE e múltiplos monitores em displays espaciais flutuantes no seu ambiente físico.';

  @override
  String get questCard3Title => 'Decodificação Acelerada por Hardware';

  @override
  String get questCard3Desc =>
      'Pipeline nativo de decodificação H.264 no chip Snapdragon XR2 Gen 2 sem emulação de CPU.';

  @override
  String get questLinkSpecs => 'Especificações Meta Quest 3S';

  @override
  String get questLinkDev => 'Centro de Desenvolvedores Meta Quest';

  @override
  String get questLinkAdb => 'Guia de Sideload & ADB';

  @override
  String get questLinkSidequest => 'Comunidade SideQuest';
}

/// The translations for Portuguese, as used in Brazil (`pt_BR`).
class AppLocalizationsPtBr extends AppLocalizationsPt {
  AppLocalizationsPtBr() : super('pt_BR');

  @override
  String get navBrand => 'PARALLAX';

  @override
  String get navBadge => 'EXPERIMENTAL v0.1';

  @override
  String get navArchitecture => 'Arquitetura';

  @override
  String get navFeatures => 'Recursos';

  @override
  String get navHowItWorks => 'Como Funciona';

  @override
  String get navGetStarted => 'Começar';

  @override
  String get navProtocol => 'Protocolo';

  @override
  String get navRoadmap => 'Roadmap';

  @override
  String get navThemeLight => 'Modo Claro';

  @override
  String get navThemeDark => 'Modo Escuro';

  @override
  String get navLanguage => 'Idioma';

  @override
  String get navGithub => 'GitHub';

  @override
  String get heroHeadline =>
      'Transmita seu desktop Linux para VR com latência ultrabaixa.';

  @override
  String get heroSubHeadline =>
      'Código aberto. Experimental. Criado para o futuro da computação espacial.';

  @override
  String get heroPrimaryCta => 'Começar';

  @override
  String get heroSecondaryCta => 'Ver no GitHub';

  @override
  String get heroMicroText =>
      'Atualmente em estágio inicial de desenvolvimento (v0.1).';

  @override
  String get aboutTitle => 'O que é o Parallax?';

  @override
  String get aboutDescription =>
      'O Parallax é um pipeline experimental de renderização remota para transmitir um desktop Linux para dispositivos clientes focado em realidade virtual/aumentada (VR/AR). Ele captura telas X11, codifica em H.264, transmite via UDP e coordena sessões por um canal de controle TCP leve.';

  @override
  String get aboutDetail1 =>
      'Host Linux captura display X11 para streaming em VR em tempo real.';

  @override
  String get aboutDetail2 =>
      'Quadros codificados em H.264 via aceleração de hardware (VAAPI) ou software.';

  @override
  String get aboutDetail3 =>
      'Streaming de vídeo via UDP para entrega com latência ultrabaixa.';

  @override
  String get aboutDetail4 =>
      'Canal de controle TCP gerencia tokens de pareamento e configuração de sessão.';

  @override
  String get aboutDetail5 =>
      'Em evolução contínua — contribuições da comunidade são bem-vindas.';

  @override
  String get architectureTitle => 'Visão Geral da Arquitetura';

  @override
  String get architectureSubtitle =>
      'Pipeline minimalista de baixa latência entre daemon Linux host e cliente Android com controle bidirecional e streaming de vídeo unidirecional.';

  @override
  String get archHostTitle => 'Host Linux (Rust)';

  @override
  String get archHostSubtitle => 'prlx-hostd';

  @override
  String get archHostTooltip =>
      'Captura quadros X11, codifica H.264 e gerencia sessões de controle.';

  @override
  String get archX11Title => 'Captura X11';

  @override
  String get archX11Subtitle => 'Feed de exibição';

  @override
  String get archX11Tooltip =>
      'Captura quadros do desktop Linux diretamente do servidor X11.';

  @override
  String get archEncodeTitle => 'Codificação H.264';

  @override
  String get archEncodeSubtitle => 'VAAPI ou software';

  @override
  String get archEncodeTooltip =>
      'Codifica quadros em fluxo de bytes H.264 Annex B.';

  @override
  String get archUdpStreamTitle => 'Stream de Vídeo UDP';

  @override
  String get archUdpStreamSubtitle => 'Enquadramento leve';

  @override
  String get archUdpStreamTooltip =>
      'Envia pacotes UDP com cabeçalho compacto de 24 bytes.';

  @override
  String get archAndroidTitle => 'Cliente Android';

  @override
  String get archAndroidSubtitle => 'Parallax Receiver';

  @override
  String get archAndroidTooltip =>
      'App Jetpack Compose para reprodução em VR/AR.';

  @override
  String get archQrTitle => 'Pareamento QR';

  @override
  String get archQrSubtitle => 'Fluxo por token';

  @override
  String get archQrTooltip =>
      'Escaneia código QR para conexão segura ao canal de controle.';

  @override
  String get archTcpTitle => 'Controle TCP';

  @override
  String get archTcpSubtitle => 'Broker de sessão';

  @override
  String get archTcpTooltip =>
      'Coordena criação de sessão, pareamento e configuração.';

  @override
  String get archDecodeTitle => 'Decodificação H.264';

  @override
  String get archDecodeSubtitle => 'UI de reprodução';

  @override
  String get archDecodeTooltip =>
      'Decodifica quadros H.264 para renderização imersiva.';

  @override
  String get archFlowTcp => 'Controle TCP (bidirecional)';

  @override
  String get archFlowUdp => 'Stream de vídeo UDP (host → cliente)';

  @override
  String get featuresTitle => 'Recursos principais';

  @override
  String get featuresSubtitle =>
      'Primitivas de streaming de alta performance desenhadas para os limites de latência em VR.';

  @override
  String get featUdpTitle => 'Streaming UDP de baixa latência';

  @override
  String get featUdpDesc =>
      'Protocolo de enquadramento minimalista para máxima velocidade e resiliência.';

  @override
  String get featPairingTitle => 'Pareamento seguro';

  @override
  String get featPairingDesc =>
      'Canal de controle protegido por tokens de pareamento e fluxo QR.';

  @override
  String get featProtocolTitle => 'Protocolo aberto';

  @override
  String get featProtocolDesc =>
      'Estrutura de pacotes UDP documentada e totalmente extensível.';

  @override
  String get featHostUiTitle => 'Interface do Host';

  @override
  String get featHostUiDesc =>
      'Interface desktop para pareamento e controle de sessões ativas.';

  @override
  String get featAndroidClientTitle => 'Cliente Android';

  @override
  String get featAndroidClientDesc =>
      'App Jetpack Compose que escaneia QR para conectar instantaneamente.';

  @override
  String get howItWorksTitle => 'Como funciona';

  @override
  String get howItWorksSubtitle =>
      'Da tela do desktop aos óculos VR em cinco etapas sincronizadas de pipeline.';

  @override
  String get step1Title => 'Captura da tela X11';

  @override
  String get step1Desc =>
      'O prlx-hostd captura os quadros da área de trabalho diretamente do X11.';

  @override
  String get step2Title => 'Codificação em H.264';

  @override
  String get step2Desc =>
      'Codificação por hardware VAAPI ou fallback por software para portabilidade.';

  @override
  String get step3Title => 'Empacotamento via Parallax';

  @override
  String get step3Desc =>
      'Quadros divididos em pacotes UDP com metadados de fluxo e sincronização.';

  @override
  String get step4Title => 'Coordenação via TCP';

  @override
  String get step4Desc =>
      'Token de pareamento e comandos de controle trafegam por TCP.';

  @override
  String get step5Title => 'Decodificação no Android';

  @override
  String get step5Desc =>
      'Cliente Jetpack Compose escaneia QR e renderiza o fluxo em tempo real.';

  @override
  String get gettingStartedTitle => 'Primeiros passos';

  @override
  String get gettingStartedSubtitle =>
      'Escolha um método de instalação para executar o Parallax no seu host Linux.';

  @override
  String get sample1Title => 'Instalação em comando único (Debian/Ubuntu)';

  @override
  String get sample1Caption =>
      'Instala dependências, binários, comando CLI e inicializador desktop.';

  @override
  String get sample2Title => 'URL do instalador no Cloudflare Pages';

  @override
  String get sample2Caption =>
      'Instalador publicado como asset estático em parallax.asodya.com.';

  @override
  String get sample3Title => 'Instalação via Cargo';

  @override
  String get sample3Caption =>
      'Alternativa para desenvolvedores Rust usando o Cargo.';

  @override
  String get sample4Title => 'Instalação manual pelo repositório';

  @override
  String get sample4Caption =>
      'Alternativa manual para quem deseja controle total passo a passo.';

  @override
  String get copyCommand => 'Copiar comando';

  @override
  String get commandCopied => 'Copiado para a área de transferência';

  @override
  String get protocolTitle => 'Protocolo: Enquadramento de pacotes UDP';

  @override
  String get protocolSubtitle =>
      'O Parallax adota um cabeçalho fixo de 24 bytes com magic value \"PRLX\", identificadores de stream/frame e flags de keyframe, config e fim de frame.';

  @override
  String get protocolHighlight1 =>
      'Cabeçalho fixo de 24 bytes em ordem de rede big-endian.';

  @override
  String get protocolHighlight2 =>
      'Magic value \"PRLX\" valida integridade do frame.';

  @override
  String get protocolHighlight3 =>
      'Stream ID + Frame ID para remontagem e sincronismo.';

  @override
  String get protocolHighlight4 =>
      'Flags de controle para keyframe, config e marcadores de término.';

  @override
  String get protocolHighlight5 =>
      'Tipos de carga útil para vídeo, áudio e dados de controle.';

  @override
  String get protocolPayloadNotesTitle => 'Notas sobre payload';

  @override
  String get protocolNote1 =>
      'Payloads H.264 operam em formato Annex B delimitados por start-code.';

  @override
  String get protocolNote2 =>
      'SPS/PPS podem ser enviados em pacotes de configuração ou com keyframes.';

  @override
  String get roadmapTitle => 'Status do projeto e roadmap';

  @override
  String get roadmapSubtitle =>
      'Versão experimental inicial (v0.1). Desenvolvimento ativo para otimização de latência.';

  @override
  String get roadmapStatus1 => 'Versão experimental inicial (v0.1).';

  @override
  String get roadmapStatus2 =>
      'Desenvolvimento ativo para otimização de estabilidade e latência.';

  @override
  String get roadmapStatus3 =>
      'Contribuições da comunidade são muito bem-vindas.';

  @override
  String get roadmapItem1Title => 'Otimização de latência';

  @override
  String get roadmapItem1Desc =>
      'Redução de tempos do ciclo captura → codificação → transporte.';

  @override
  String get roadmapItem2Title => 'Aprimoramento de aceleração de hardware';

  @override
  String get roadmapItem2Desc =>
      'Ampliação de suporte a encoders VAAPI e detecção inteligente de GPU.';

  @override
  String get roadmapItem3Title => 'Expansão de plataformas clientes';

  @override
  String get roadmapItem3Desc =>
      'Desenvolvimento de clientes para desktop e headsets VR adicionais.';

  @override
  String get roadmapItem4Title => 'Renderização nativa em VR';

  @override
  String get roadmapItem4Desc =>
      'Integração imersiva de controles e renderização espacial direta.';

  @override
  String get communityTitle => 'Código aberto e comunidade';

  @override
  String get communityDescription =>
      'O Parallax possui licença MIT com foco em pesquisa e inovação. Participe com sugestões, issues e pull requests no GitHub.';

  @override
  String get communityItem1 => 'Licença MIT permissiva';

  @override
  String get communityItem2 => 'Documentação de protocolo aberta';

  @override
  String get communityItem3 => 'Discussions e issues ativas no GitHub';

  @override
  String get finalCtaHeadline => 'Construa o futuro do streaming para VR.';

  @override
  String get finalCtaSubHeadline =>
      'O Parallax é código aberto e está em constante evolução — junte-se à comunidade.';

  @override
  String get footerCopyright => '© 2026 Asodya. Todos os direitos reservados.';

  @override
  String get questSectionTitle => 'Dispositivos Alvo: Ecossistema Meta Quest';

  @override
  String get questSectionSubtitle =>
      'Projetado para Meta Quest 3, Quest 3S e clientes Android XR com streaming UDP de baixa latência e alta taxa de quadros.';

  @override
  String get questOptimizedBadge => 'OTIMIZADO PARA META QUEST 3 & QUEST 3S';

  @override
  String get questCard1Title => 'Pipeline Sem Fio no Meta Quest 3S';

  @override
  String get questCard1Desc =>
      'Transmita a área de trabalho Linux em resolução máxima via Wi-Fi 6E com latência ultrabaixa.';

  @override
  String get questCard2Title => 'Espaço de Trabalho Espacial Imersivo';

  @override
  String get questCard2Desc =>
      'Transforme seu terminal, IDE e múltiplos monitores em displays espaciais flutuantes no seu ambiente físico.';

  @override
  String get questCard3Title => 'Decodificação Acelerada por Hardware';

  @override
  String get questCard3Desc =>
      'Pipeline nativo de decodificação H.264 no chip Snapdragon XR2 Gen 2 sem emulação de CPU.';

  @override
  String get questLinkSpecs => 'Especificações Meta Quest 3S';

  @override
  String get questLinkDev => 'Centro de Desenvolvedores Meta Quest';

  @override
  String get questLinkAdb => 'Guia de Sideload & ADB';

  @override
  String get questLinkSidequest => 'Comunidade SideQuest';
}
