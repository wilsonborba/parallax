// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Thai (`th`).
class AppLocalizationsTh extends AppLocalizations {
  AppLocalizationsTh([String locale = 'th']) : super(locale);

  @override
  String get navBrand => 'PARALLAX';

  @override
  String get navBadge => 'EXPERIMENTAL v0.1';

  @override
  String get navArchitecture => 'สถาปัตยกรรม';

  @override
  String get navFeatures => 'คุณสมบัติ';

  @override
  String get navHowItWorks => 'การทำงาน';

  @override
  String get navGetStarted => 'เริ่มต้นใช้งาน';

  @override
  String get navProtocol => 'โปรโตคอล';

  @override
  String get navRoadmap => 'แผนงาน';

  @override
  String get navThemeLight => 'โหมดสว่าง';

  @override
  String get navThemeDark => 'โหมดมืด';

  @override
  String get navLanguage => 'ภาษา';

  @override
  String get navGithub => 'GitHub';

  @override
  String get heroHeadline =>
      'สตรีมเดสก์ท็อป Linux สู่ VR ด้วยความหน่วงต่ำเป็นพิเศษ';

  @override
  String get heroSubHeadline =>
      'โอเพนซอร์ส เพื่อการทดลอง สร้างขึ้นเพื่ออนาคตของการประมวลผลเชิงพื้นที่';

  @override
  String get heroPrimaryCta => 'เริ่มต้นใช้งาน';

  @override
  String get heroSecondaryCta => 'ดูบน GitHub';

  @override
  String get heroMicroText => 'กำลังอยู่ในช่วงการพัฒนาขั้นต้น (v0.1)';

  @override
  String get aboutTitle => 'Parallax คืออะไร?';

  @override
  String get aboutDescription =>
      'Parallax คือไปป์ไลน์การเรนเดอร์ระยะไกลเชิงทดลองสำหรับการสตรีมเดสก์ท็อป Linux ไปยังอุปกรณ์ปลายทาง โดยมุ่งเน้นการใช้งานกับ VR/AR ทำการจับภาพหน้าจอ X11 เข้ารหัสเป็น H.264 ส่งผ่าน UDP และประสานเซสชันผ่านช่องทางควบคุม TCP น้ำหนักเบา';

  @override
  String get aboutDetail1 =>
      'โฮสต์ Linux จับภาพหน้าจอ X11 สำหรับการสตรีม VR แบบเรียลไทม์';

  @override
  String get aboutDetail2 =>
      'เฟรมจะถูกเข้ารหัสเป็น H.264 โดยใช้ฮาร์ดแวร์ (VAAPI) หรือซอฟต์แวร์สำรอง';

  @override
  String get aboutDetail3 =>
      'สตรีมวิดีโอผ่าน UDP เพื่อให้ได้ความหน่วงต่ำระดับมิลลิวินาที';

  @override
  String get aboutDetail4 =>
      'ช่องทางควบคุม TCP จัดการโทเค็นการจับคู่และการตั้งค่าเซสชัน';

  @override
  String get aboutDetail5 =>
      'อยู่ในช่วงเริ่มต้นและกำลังพัฒนา ยินดีต้อนรับการมีส่วนร่วมจากชุมชน';

  @override
  String get architectureTitle => 'ภาพรวมสถาปัตยกรรม';

  @override
  String get architectureSubtitle =>
      'ไปป์ไลน์ความหน่วงต่ำที่เรียบง่ายระหว่าง Linux host daemon และไคลเอนต์ Android พร้อมการควบคุมสองทิศทางและการสตรีมวิดีโอทิศทางเดียว';

  @override
  String get archHostTitle => 'โฮสต์ Linux (Rust)';

  @override
  String get archHostSubtitle => 'prlx-hostd';

  @override
  String get archHostTooltip =>
      'จับภาพเฟรม X11 เข้ารหัส H.264 และให้บริการเซสชันควบคุม';

  @override
  String get archX11Title => 'การจับภาพ X11';

  @override
  String get archX11Subtitle => 'ฟีดหน้าจอ';

  @override
  String get archX11Tooltip =>
      'ดึงเฟรมเดสก์ท็อป Linux โดยตรงจากเซิร์ฟเวอร์แสดงผล X11';

  @override
  String get archEncodeTitle => 'เข้ารหัส H.264';

  @override
  String get archEncodeSubtitle => 'VAAPI หรือซอฟต์แวร์';

  @override
  String get archEncodeTooltip => 'เข้ารหัสเฟรมเป็นไบต์สตรีม H.264 Annex B';

  @override
  String get archUdpStreamTitle => 'สตรีมวิดีโอ UDP';

  @override
  String get archUdpStreamSubtitle => 'ส่วนหัวขนาดกะทัดรัด';

  @override
  String get archUdpStreamTooltip =>
      'ส่งแพ็กเก็ต UDP ด้วยส่วนหัว 24 ไบต์น้ำหนักเบา';

  @override
  String get archAndroidTitle => 'ไคลเอนต์ Android';

  @override
  String get archAndroidSubtitle => 'Parallax Receiver';

  @override
  String get archAndroidTooltip => 'แอป Jetpack Compose สำหรับการสตรีม VR/AR';

  @override
  String get archQrTitle => 'การจับคู่ผ่าน QR';

  @override
  String get archQrSubtitle => 'เวิร์กโฟลว์โทเค็น';

  @override
  String get archQrTooltip => 'สแกน QR เพื่อเข้าร่วมช่องทางควบคุมอย่างปลอดภัย';

  @override
  String get archTcpTitle => 'การควบคุม TCP';

  @override
  String get archTcpSubtitle => 'ตัวจัดการเซสชัน';

  @override
  String get archTcpTooltip =>
      'ประสานงานการตั้งค่าเซสชัน การจับคู่ และการกำหนดค่า';

  @override
  String get archDecodeTitle => 'ถอดรหัส H.264';

  @override
  String get archDecodeSubtitle => 'UI สตรีมมิ่ง';

  @override
  String get archDecodeTooltip =>
      'ถอดรหัสเฟรม H.264 เพื่อการแสดงผลภาพเสมือนจริง';

  @override
  String get archFlowTcp => 'การควบคุม TCP (สองทิศทาง)';

  @override
  String get archFlowUdp => 'สตรีมวิดีโอ UDP (โฮสต์ → ไคลเอนต์)';

  @override
  String get featuresTitle => 'คุณสมบัติหลัก';

  @override
  String get featuresSubtitle =>
      'ระบบสตรีมมิ่งประสิทธิภาพสูง ออกแบบมาเพื่อควบคุมความหน่วงในระบบ VR';

  @override
  String get featUdpTitle => 'การสตรีม UDP ความหน่วงต่ำ';

  @override
  String get featUdpDesc =>
      'โปรโตคอลแพ็กเกจขั้นต่ำเพื่อความเร็วและความเสถียรสูงสุด';

  @override
  String get featPairingTitle => 'การจับคู่อย่างปลอดภัย';

  @override
  String get featPairingDesc =>
      'ช่องทางควบคุมใช้โทเค็นการจับคู่และกระบวนการสแกน QR';

  @override
  String get featProtocolTitle => 'โปรโตคอลแบบเปิด';

  @override
  String get featProtocolDesc =>
      'โครงสร้างแพ็กเก็ต UDP มีเอกสารกำกับและขยายขีดความสามารถได้';

  @override
  String get featHostUiTitle => 'อินเทอร์เฟซของโฮสต์';

  @override
  String get featHostUiDesc => 'เดสก์ท็อป UI สำหรับการจับคู่และการควบคุมเซสชัน';

  @override
  String get featAndroidClientTitle => 'ไคลเอนต์ Android';

  @override
  String get featAndroidClientDesc =>
      'แอป Jetpack Compose สแกน QR เพื่อเชื่อมต่อได้ทันที';

  @override
  String get howItWorksTitle => 'ขั้นตอนการทำงาน';

  @override
  String get howItWorksSubtitle =>
      'จากหน้าจอเดสก์ท็อปสู่แว่น VR ใน 5 ขั้นตอนของไปป์ไลน์ที่ทำงานสอดประสานกัน';

  @override
  String get step1Title => 'จับภาพหน้าจอ X11';

  @override
  String get step1Desc =>
      'prlx-hostd ดึงภาพหน้าจอเดสก์ท็อป Linux โดยตรงจาก X11';

  @override
  String get step2Title => 'เข้ารหัสด้วย H.264';

  @override
  String get step2Desc =>
      'เข้ารหัสด้วยฮาร์ดแวร์ VAAPI หรือซอฟต์แวร์สำรองเพื่อความยืดหยุ่น';

  @override
  String get step3Title => 'จัดแพ็กเก็ตผ่าน Parallax framing';

  @override
  String get step3Desc =>
      'แบ่งเฟรมเป็นแพ็กเก็ต UDP พร้อมข้อมูลเมตาของสตรีมและเฟรม';

  @override
  String get step4Title => 'ประสานงานผ่าน TCP';

  @override
  String get step4Desc => 'โทเค็นการจับคู่และคำสั่งควบคุมส่งผ่านช่องทาง TCP';

  @override
  String get step5Title => 'ไคลเอนต์ Android ถอดรหัส';

  @override
  String get step5Desc =>
      'ไคลเอนต์ Jetpack Compose สแกน QR และเรนเดอร์สตรีมแบบเรียลไทม์';

  @override
  String get gettingStartedTitle => 'เริ่มต้นใช้งาน';

  @override
  String get gettingStartedSubtitle =>
      'เลือกวิธีการติดตั้งเพื่อติดตั้ง Parallax บนเครื่อง Linux โฮสต์ของคุณ';

  @override
  String get sample1Title => 'ติดตั้งด้วยคำสั่งเดียว (Debian/Ubuntu)';

  @override
  String get sample1Caption =>
      'ติดตั้งการอ้างอิง ไบนารี คำสั่ง CLI และตัวเปิดใช้งานเดสก์ท็อป';

  @override
  String get sample2Title => 'URL ตัวติดตั้ง Cloudflare Pages';

  @override
  String get sample2Caption =>
      'ตัวติดตั้งเผยแพร่เป็นไฟล์สแตติกที่ parallax.asodya.com';

  @override
  String get sample3Title => 'การติดตั้งผ่าน Cargo';

  @override
  String get sample3Caption =>
      'ทางเลือกสำหรับผู้ใช้ที่ต้องการสร้างด้วย Cargo (Rust)';

  @override
  String get sample4Title => 'ติดตั้งจากที่เก็บโค้ด (Git Clone)';

  @override
  String get sample4Caption =>
      'ทางเลือกสำหรับผู้ที่ต้องการควบคุมทีละขั้นตอนด้วยตนเอง';

  @override
  String get copyCommand => 'คัดลอกคำสั่ง';

  @override
  String get commandCopied => 'คัดลอกลงในคลิปบอร์ดแล้ว';

  @override
  String get protocolTitle => 'โปรโตคอล: การจัดเฟรมแพ็กเก็ต UDP';

  @override
  String get protocolSubtitle =>
      'Parallax ใช้ส่วนหัวคงที่ขนาด 24 ไบต์พร้อมค่า Magic \"PRLX\" ตัวระบุสตรีม/เฟรม และแฟล็กสำหรับคีย์เฟรมและการกำหนดค่า';

  @override
  String get protocolHighlight1 =>
      'ส่วนหัวคงที่ 24 ไบต์ในรูปแบบ Big-endian network order';

  @override
  String get protocolHighlight2 =>
      'ค่า Magic \"PRLX\" ยืนยันความถูกต้องของเฟรม';

  @override
  String get protocolHighlight3 =>
      'Stream ID + Frame ID สำหรับการประกอบเฟรมใหม่และการกำหนดเวลา';

  @override
  String get protocolHighlight4 =>
      'แฟล็กสำหรับคีย์เฟรม การกำหนดค่า และเครื่องหมายสิ้นสุดเฟรม';

  @override
  String get protocolHighlight5 =>
      'ประเภท Payload สำหรับวิดีโอ เสียง และข้อมูลควบคุม';

  @override
  String get protocolPayloadNotesTitle => 'หมายเหตุเกี่ยวกับ Payload';

  @override
  String get protocolNote1 => 'H.264 Payload เป็นไบต์สตรีมแบบ Annex B';

  @override
  String get protocolNote2 =>
      'SPS/PPS สามารถส่งมาพร้อมกับคีย์เฟรมหรือแพ็กเก็ตการกำหนดค่า';

  @override
  String get roadmapTitle => 'สถานะโครงการและแผนงาน';

  @override
  String get roadmapSubtitle =>
      'เวอร์ชันทดลองเริ่มต้น (v0.1) กำลังพัฒนาอย่างต่อเนื่องเพื่อลดความหน่วง';

  @override
  String get roadmapStatus1 => 'เวอร์ชันทดลองเริ่มต้น (v0.1)';

  @override
  String get roadmapStatus2 =>
      'กำลังพัฒนาอย่างต่อเนื่องเพื่อความเสถียรและความหน่วงที่ต่ำลง';

  @override
  String get roadmapStatus3 => 'ยินดีต้อนรับการมีส่วนร่วมจากชุมชนนักพัฒนา';

  @override
  String get roadmapItem1Title => 'เพิ่มประสิทธิภาพความหน่วง';

  @override
  String get roadmapItem1Desc =>
      'ปรับจูนระยะเวลาในไปป์ไลน์ จับภาพ → เข้ารหัส → การส่งข้อมูล';

  @override
  String get roadmapItem2Title => 'เพิ่มการรองรับตัวเข้ารหัสฮาร์ดแวร์';

  @override
  String get roadmapItem2Desc =>
      'ขยายเส้นทาง VAAPI/codec และการสลับโหมดอัตโนมัติ';

  @override
  String get roadmapItem3Title => 'ขยายแพลตฟอร์มไคลเอนต์';

  @override
  String get roadmapItem3Desc =>
      'พัฒนาไคลเอนต์สำหรับเดสก์ท็อปและอุปกรณ์ VR เพิ่มเติม';

  @override
  String get roadmapItem4Title => 'การเรนเดอร์ VR แบบเนทีฟ';

  @override
  String get roadmapItem4Desc =>
      'ผสานรวมการควบคุมแบบ VR และมุมมองเชิงพื้นที่โดยตรง';

  @override
  String get communityTitle => 'โอเพนซอร์สและชุมชน';

  @override
  String get communityDescription =>
      'Parallax เผยแพร่ภายใต้ใบอนุญาต MIT มุ่งเน้นการวิจัยและการทดลอง ร่วมส่งความคิดเห็นและ Pull Request บน GitHub';

  @override
  String get communityItem1 => 'ใบอนุญาต MIT แบบเปิด';

  @override
  String get communityItem2 => 'เอกสารโปรโตคอลแบบเปิดเผย';

  @override
  String get communityItem3 => 'Issues และ Discussions ที่เปิดรับบน GitHub';

  @override
  String get finalCtaHeadline => 'ร่วมสร้างอนาคตของการสตรีม VR';

  @override
  String get finalCtaSubHeadline =>
      'Parallax เป็นโอเพนซอร์สและกำลังเติบโต — มาร่วมเป็นส่วนหนึ่งของชุมชน';

  @override
  String get footerCopyright => '© 2026 Asodya สงวนลิขสิทธิ์';

  @override
  String get questSectionTitle => 'อุปกรณ์เป้าหมาย: ระบบนิเวศ Meta Quest';

  @override
  String get questSectionSubtitle =>
      'ออกแบบมาสำหรับ Meta Quest 3, Quest 3S และไคลเอนต์ Android XR พร้อมการสตรีม UDP ความหน่วงต่ำและการแสดงผลภาพเฟรมเรตสูง';

  @override
  String get questOptimizedBadge =>
      'ปรับแต่งมาสำหรับ META QUEST 3 และ QUEST 3S';

  @override
  String get questCard1Title => 'ระบบสตรีมมิ่งไร้สายบน Meta Quest 3S';

  @override
  String get questCard1Desc =>
      'สตรีมเดสก์ท็อป Linux แบบเต็มความละเอียดผ่าน Wi-Fi 6E ด้วยความหน่วงต่ำกว่า 20ms';

  @override
  String get questCard2Title => 'พื้นที่ทำงานเชิงพื้นที่ลอยในห้อง';

  @override
  String get questCard2Desc =>
      'เปลี่ยนเทอร์มินัล, IDE และจอแสดงผลหลายจอของคุณให้กลายเป็นหน้าต่างแสดงผลเชิงพื้นที่ลอยได้ในห้องจริง';

  @override
  String get questCard3Title => 'การถอดรหัสเร่งความเร็วด้วยฮาร์ดแวร์';

  @override
  String get questCard3Desc =>
      'ไปป์ไลน์การถอดรหัส H.264 เนทีฟบนชิป Snapdragon XR2 Gen 2 เพื่อประสิทธิภาพสูงสุดและประหยัดพลังงาน';

  @override
  String get questLinkSpecs => 'ข้อมูลจำเพาะ Meta Quest 3S';

  @override
  String get questLinkDev => 'ศูนย์นักพัฒนา Meta Quest';

  @override
  String get questLinkAdb => 'คู่มือการติดตั้งแอป (Sideload / ADB)';

  @override
  String get questLinkSidequest => 'ชุมชน SideQuest';
}
