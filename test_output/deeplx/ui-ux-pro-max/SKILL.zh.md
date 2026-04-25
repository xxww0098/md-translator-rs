---
name: ui-ux-pro-max
description: "UI/UX design intelligence for web and mobile. Includes 50+ styles, 161 color palettes, 57 font pairings, 161 product types, 99 UX guidelines, and 25 chart types across 10 stacks (React, Next.js, Vue, Svelte, SwiftUI, React Native, Flutter, Tailwind, shadcn/ui, and HTML/CSS). Actions: plan, build, create, design, implement, review, fix, improve, optimize, enhance, refactor, and check UI/UX code. Projects: website, landing page, dashboard, admin panel, e-commerce, SaaS, portfolio, blog, and mobile app. Elements: button, modal, navbar, sidebar, card, table, form, and chart. Styles: glassmorphism, claymorphism, minimalism, brutalism, neumorphism, bento grid, dark mode, responsive, skeuomorphism, and flat design. Topics: color systems, accessibility, animation, layout, typography, font pairing, spacing, interaction states, shadow, and gradient. Integrations: shadcn/ui MCP for component search and examples."
---

# UI/UX Pro Max - 设计智能

网络和移动应用程序的综合设计指南。包含 50 多种风格、161 种调色板、57 种字体搭配、161 种产品类型（含推理规则）、99 项用户体验指南以及 25 种图表类型，涵盖 10 种技术堆栈。可搜索数据库，提供基于优先级的建议。

## 何时申请

当任务涉及用户界面结构、视觉设计决策、交互模式或用户体验质量控制时，应使用此技能。****

### 必须使用

在以下情况下必须调用该技能：

- 设计新页面（登陆页面、仪表板、管理员、SaaS、移动应用程序）
- 创建或重构用户界面组件（按钮、模式、表单、表格、图表等）
- 选择配色方案、排版系统、间距标准或布局系统
- 审查用户界面代码，以确保用户体验、可访问性或视觉一致性
- 实施导航结构、动画或响应行为
- 做出产品层面的设计决策（风格、信息层级、品牌表达）
- 提高界面的感知质量、清晰度或可用性

### 推荐

建议在以下情况下使用此技能：

- 用户界面看起来 "不够专业"，但原因不明
- 接收有关可用性或体验的反馈
- 启动前优化用户界面质量
- 调整跨平台设计（网络/iOS/安卓）
- 调整跨平台设计（网络/iOS/安卓）

### 建立设计系统或可重复使用的组件库

建立设计系统或可重复使用的组件库

- 在以下情况下不需要此技能：
- 纯后台逻辑开发
- 仅涉及应用程序接口或数据库设计
- 与界面无关的性能优化
- 基础设施或 DevOps 工作

**非视觉脚本或自动化任务******

## 非视觉脚本或自动化任务

*判定标准：如果任务会改变功能的外观、感觉、移动或交互方式，则应使用此技能。`--domain <Domain>`*

| Priority | Category | Impact | Domain | Key Checks (Must Have) | Anti-Patterns (Avoid) |
| --- | --- | --- | --- | --- | --- |
| 1 | Accessibility | CRITICAL | `ux` | Contrast 4.5:1, Alt text, Keyboard nav, Aria-labels | Removing focus rings, Icon-only buttons without labels |
| 2 | Touch & Interaction | CRITICAL | `ux` | Min size 44×44px, 8px+ spacing, Loading feedback | Reliance on hover only, Instant state changes (0ms) |
| 3 | Performance | HIGH | `ux` | WebP/AVIF, Lazy loading, Reserve space (CLS \< 0.1) | Layout thrashing, Cumulative Layout Shift |
| 4 | Style Selection | HIGH | `style`, `product` | Match product type, Consistency, SVG icons (no emoji) | Mixing flat & skeuomorphic randomly, Emoji as icons |
| 5 | Layout & Responsive | HIGH | `ux` | Mobile-first breakpoints, Viewport meta, No horizontal scroll | Horizontal scroll, Fixed px container widths, Disable zoom |
| 6 | Typography & Color | MEDIUM | `typography`, `color` | Base 16px, Line-height 1.5, Semantic color tokens | Text \< 12px body, Gray-on-gray, Raw hex in components |
| 7 | Animation | MEDIUM | `ux` | Duration 150–300ms, Motion conveys meaning, Spatial continuity | Decorative-only animation, Animating width/height, No reduced-motion |
| 8 | Forms & Feedback | MEDIUM | `ux` | Visible labels, Error near field, Helper text, Progressive disclosure | Placeholder-only label, Errors only at top, Overwhelm upfront |
| 9 | Navigation Patterns | HIGH | `ux` | Predictable back, Bottom nav ≤5, Deep linking | Overloaded nav, Broken back behavior, No deep links |
| 10 | Charts & Data | LOW | `chart` | Legends, Tooltips, Accessible colors | Relying on color alone to convey meaning |

## 按优先级划分的规则类别

### 供人类/人工智能参考：按照优先级 1→10 决定首先关注哪个规则类别；需要时用于查询详细信息。脚本不读取此表。

- `color-contrast`1.无障碍（关键）
- `focus-states`- 普通文本的最小比例为 4.5:1（大文本为 3:1）；材料设计
- `alt-text`- 互动元素上的明显聚焦环（2-4px；Apple HIG、MD）
- `aria-labels`- 为有意义的图像提供描述性 alt 文本
- `keyboard-nav`- aria-label 用于只显示图标的按钮；accessibilityLabel 用于本地按钮（Apple HIG）
- `form-labels`- 制表符顺序与视觉顺序一致；全键盘支持（Apple HIG）
- `skip-links`- 使用带 for 属性的标签
- `heading-hierarchy`- 键盘用户可跳至主要内容
- `color-not-only`- 顺序 h1→h6，无跳级
- `dynamic-type`- 不要仅靠颜色传达信息（添加图标/文字）
- `reduced-motion`- 支持系统文本缩放；避免文本增长时出现截断（Apple 动态类型，MD）
- `voiceover-sr`- 尊重 prefers-reduced-motion；根据要求减少/禁用动画（Apple Reduced Motion API，MD）。
- `escape-routes`- 有意义的可访问性标签/可访问性提示；VoiceOver/屏幕阅读器的逻辑阅读顺序（Apple HIG、MD）
- `keyboard-shortcuts`- 在模式和多步骤流程中提供取消/返回功能（Apple HIG）

### \- 保留系统和 a11y 快捷键；为拖放操作提供键盘选择（Apple HIG）

- `touch-target-size`2.触摸与互动（关键）
- `touch-spacing`- 最小 44×44pt（苹果）/ 48×48dp（材料）；如有必要，可将打击区域扩展到视觉范围之外
- `hover-vs-tap`- 触摸目标之间的最小间隙为 8px/8dp（Apple HIG、MD）
- `loading-buttons`- 使用点击/轻触进行主要互动；不要仅仅依赖悬停
- `error-feedback`- 在异步操作期间禁用按钮；显示旋转器或进度条
- `cursor-pointer`- 清除问题附近的错误信息
- `gesture-conflicts`- 为可点击元素添加光标指针（网络）
- `tap-delay`- 避免在主要内容上横向滑动，而是选择垂直滚动
- `standard-gestures`- 使用 "触摸操作"：操作以减少 300 毫秒的延迟（网络）
- `system-gestures`- 统一使用平台标准手势；不要重新定义（如向后轻扫、捏住缩放）（Apple HIG）
- `press-feedback`- 不要阻止系统手势（控制中心、向后轻扫等）（Apple HIG）
- `haptic-feedback`- 按压时的视觉反馈（波纹/高亮；MD 状态层）
- `gesture-alternative`- 在确认和重要操作时使用触觉；避免过度使用（Apple HIG）
- `safe-area-awareness`- 不要只依赖手势进行交互；始终为关键操作提供可见控件
- `no-precision-required`- 让主要触摸目标远离凹槽、动态岛、手势栏和屏幕边缘
- `swipe-clarity`- 避免要求对小图标或薄边缘进行像素完美的点击
- `drag-threshold`- 轻扫操作必须显示明确的承受能力或提示（楔形、标签、教程）

### \- 开始拖动前使用移动阈值，以避免意外拖动

- `image-optimization`3.性能（高）
- `image-dimension`- 使用 WebP/AVIF、响应式图像（srcset/大小）、懒加载非关键资产
- `font-loading`- 声明宽度/高度或使用宽高比以防止布局偏移（Core Web Vitals: CLS）
- `font-preload`- 使用字体显示：交换/可选，以避免不可见文本 (FOIT)；预留空间，以减少布局偏移 (MD)
- `critical-css`- 只预载关键字体；避免在每个变体上过度使用预载功能
- `lazy-loading`- 优先处理折叠上方的 CSS（内联关键 CSS 或提前加载的样式表）
- `bundle-splitting`- 通过动态导入/路由级拆分，懒加载非英雄组件
- `third-party-scripts`- 按路径/功能（React Suspense / Next.js 动态）拆分代码，以减少初始负载和 TTI
- `reduce-reflows`- 同步/删除加载第三方脚本；审核并删除不必要的脚本 (MD)
- `content-jumping`- 避免频繁读取/写入布局；先批量读取 DOM，然后再写入
- `lazy-load-below-fold`- 为异步内容预留空间，避免布局跳转（Core Web Vitals: CLS）
- `virtualize-lists`- 对于折页下方的图片和重型媒体，使用 loading="lazy" 功能
- `main-thread-budget`- 虚拟化包含 50 多个项目的列表，提高内存效率和滚动性能
- `progressive-loading`- 将每帧工作时间控制在 ~16ms 以下（60fps）；将繁重任务移出主线程（HIG、MD
- `input-latency`- 在 \>1 秒的操作中使用镂空屏幕/微光，而不是长阻塞旋转器（Apple HIG）
- `tap-feedback-speed`- 将轻敲/滚动的输入延迟时间控制在 ~100ms 以下（材料响应速度标准）
- `debounce-throttle`- 轻点 100 毫秒内提供视觉反馈（Apple HIG）
- `offline-support`- 对高频事件（滚动、调整大小、输入）使用去抖动/节流功能
- `network-fallback`- 提供离线状态消息传递和基本回退功能（PWA/移动设备）

### \- 为速度较慢的网络提供降级模式（图像分辨率较低、动画较少）

- `style-match`4.风格选择（高）`--design-system`
- `consistency`- 根据产品类型匹配样式（用于推荐）
- `no-emoji-icons`- 所有页面使用相同的风格
- `color-palette-from-product`- 使用 SVG 图标（Heroicons、Lucide），而不是表情符号`--domain color`
- `effects-match-style`- 从产品/行业中选择调色板（搜索）
- `platform-adaptive`- 阴影、模糊、半径与所选风格（玻璃/平面/粘土等）一致
- `state-clarity`- 尊重平台习惯（iOS HIG 与 Material）：导航、控制、排版、动作
- `elevation-consistent`- 在保持风格的同时，使悬停/按下/禁用状态在视觉上截然不同（材料状态层）
- `dark-mode-pairing`- 在卡片、纸张、模版上使用一致的高程/阴影比例；避免随机阴影值
- `icon-style-consistent`- 同时设计明暗变体，以保持品牌、对比度和风格的一致性
- `system-controls`- 在整个产品中使用一个图标集/视觉语言（笔画宽度、角半径
- `blur-purpose`- 更喜欢本地/系统控件，而不是完全定制的控件；只有在品牌推广需要时才进行定制（Apple HIG）
- `primary-action`- 使用模糊来表示背景取消（模式、表单），而不是作为装饰（Apple HIG）。

### \- 每个屏幕应只有一个主要的 CTA；次要操作在视觉上处于从属地位（Apple HIG）

- `viewport-meta`5.布局与响应（高）
- `mobile-first`- width=device-width initial-scale=1 （切勿禁用缩放功能）
- `breakpoint-consistency`- 移动优先设计，然后扩展到平板电脑和台式电脑
- `readable-font-size`- 使用系统断点（如 375 / 768 / 1024 / 1440）
- `line-length-control`- 移动设备上的正文文字最小为 16px（避免 iOS 自动缩放功能）
- `horizontal-scroll`- 移动设备每行 35-60 个字符；台式机 60-75 个字符
- `spacing-scale`- 移动设备上无水平滚动；确保内容适合视口宽度
- `touch-density`- 使用 4pt/8dp 递增间距系统（材料设计）
- `container-width`- 保持元件间距便于触摸：不拥挤，不造成误触
- `z-index-management`- 一致的桌面最大宽度（最大-w-6xl / 7xl）
- `fixed-element-offset`- 定义分层 z 索引比例（例如 0 / 10 / 20 / 40 / 100 / 1000）
- `scroll-behavior`- 固定导航栏/底部栏必须为底层内容预留安全填充
- `viewport-units`- 避免嵌套滚动区域干扰主滚动体验
- `orientation-support`- 在移动设备上优先选择 min-h-dvh 而不是 100vh
- `content-priority`- 保持布局在横向模式下的可读性和可操作性
- `visual-hierarchy`- 在移动设备上首先显示核心内容；折叠或隐藏次要内容

### \- 通过尺寸、间距和对比度建立层次结构，而不仅仅是颜色

- `line-height`6.排版与色彩（中等）
- `line-length`- 正文文本使用 1.5-1.75
- `font-pairing`- 每行限 65-75 个字符
- `font-scale`- 匹配标题/正文字体个性
- `contrast-readability`- 一致的类型标度（如 12 14 16 18 24 32）
- `text-styles-system`- 浅色背景上的深色文本（如白色背景上的板岩-900）
- `weight-hierarchy`- 使用平台类型系统：iOS 11 动态类型样式/Material 5 类型角色（显示、标题、标题、正文、标签）（HIG、MD）
- `color-semantic`- 使用字号加强层次结构：粗体标题（600-700）、普通正文（400）、中等标签（500） (MD)
- `color-dark-mode`- 在组件中定义语义颜色标记（主要、次要、误差、表面、表面上），而不是原始十六进制（材料颜色系统）
- `color-accessible-pairs`- 深色模式使用去饱和/浅色调变体，而非倒置色彩；单独测试对比度（HIG、MD）。
- `color-not-decorative-only`- 前景/背景配对必须符合 4.5:1 (AA) 或 7:1 (AAA)；使用工具进行验证（WCAG、MD）。
- `truncation-strategy`- 功能颜色（错误红色，成功绿色）必须包括图标/文字；避免只用颜色表示（HIG，MD）。
- `letter-spacing`- 更倾向于包装而不是截断；截断时使用省略号，并通过工具提示/展开提供全文（Apple HIG）
- `number-tabular`- 尊重每个平台的默认字母间距；避免在正文文本中进行紧密跟踪（HIG、MD）
- `whitespace-balance`- 对数据列、价格和计时器使用表格/单倍行距数字，以防止布局偏移

### \- 有意使用空白来归类相关项目和分隔章节；避免视觉杂乱（Apple HIG）

- `duration-timing`7.动画（中）
- `transform-performance`- 微观相互作用使用 150-300ms；复杂过渡 ≤400ms；避免 \>500ms (MD)
- `loading-states`- 仅使用变换/不透明度；避免动画化宽度/高度/顶部/左侧
- `excessive-motion`- 加载时间超过 300ms 时显示骨架或进度指示器
- `easing`- 每个视图最多为 1-2 个关键元素制作动画
- `motion-meaning`- 进入时使用 "缓出"，退出时使用 "缓入"；避免使用线性方式进行用户界面过渡
- `state-transition`- 每个动画都必须表达因果关系，而不仅仅是装饰（Apple HIG）
- `continuity`- 状态变化（悬停/激活/展开/折叠/模态）应采用平滑的动画效果，而不是卡扣效果
- `parallax-subtle`- 页面/屏幕过渡应保持空间连续性（共享元素、定向滑动）（Apple HIG）
- `spring-physics`- 慎用视差；必须尊重缩减运动，不造成迷失方向（Apple HIG）
- `exit-faster-than-enter`- 首选基于弹簧/物理的曲线，而不是线性或立方体曲线，以获得自然的感觉（Apple HIG 流体动画）
- `stagger-sequence`- 退出动画比进入动画短（约为进入动画时长的 60-70%），以便感觉反应灵敏（MD 运动）
- `shared-element-transition`- 错开列表/网格项目入口，每个项目错开 30-50 毫秒；避免一次性或过慢显示 (MD)
- `interruptible`- 使用共享元素/英雄过渡，实现屏幕之间的视觉连续性（MD、HIG）
- `no-blocking-animation`- 动画必须是可中断的；用户点击/手势可立即取消正在进行的动画（Apple HIG）
- `fade-crossfade`- 切勿在动画期间阻止用户输入；用户界面必须保持交互性（Apple HIG）
- `scale-feedback`- 在同一容器内使用交叉渐变进行内容替换 (MD)
- `gesture-feedback`- 按下可轻触卡片/按钮时产生微妙的刻度（0.95-1.05）；松开时还原（HIG，MD）
- `hierarchy-motion`- 拖动、轻扫和捏合必须提供跟踪手指的实时视觉响应（MD Motion）。
- `motion-consistency`- 使用平移/缩放方向来表达层次：从下向上进入 = 更深，从上向下退出 = 更深 (MD)
- `opacity-threshold`- 在全局范围内统一持续时间/缓和标记；所有动画都具有相同的节奏和感觉
- `modal-motion`- 渐变元素的不透明度不应低于 0.2；要么完全渐变，要么保持可见
- `navigation-direction`- 模态/表单应根据其触发源（缩放+渐变或滑入）制作动画，以显示空间背景（HIG、MD）。
- `layout-shift-avoid`- 向前导航时，左/上为动画；向后导航时，右/下为动画 - 保持方向逻辑上的一致性 (HIG)

### \- 动画不得导致布局回流或 CLS；使用变换来改变位置

- `input-labels`8.表格与反馈 (中)
- `error-placement`- 每个输入的可见标签（非仅占位符）
- `submit-feedback`- 在相关字段下方显示错误
- `required-indicators`- 提交时加载成功/错误状态
- `empty-states`- 标记必填字段（如星号）
- `toast-dismiss`- 无内容时的有用信息和操作
- `confirmation-dialogs`- 3-5 秒内自动解除祝酒词
- `input-helper-text`- 在采取破坏性行动前进行确认
- `disabled-states`- 在复杂输入的下方提供持久的辅助文本，而不仅仅是占位符（材料设计）
- `progressive-disclosure`- 禁用元素使用降低的不透明度 (0.38-0.5) + 光标变化 + 语义属性 (MD)
- `inline-validation`- 逐步揭示复杂的选项，不要让用户一开始就不知所措（Apple HIG）
- `input-type-keyboard`- 在模糊（而非按键）时验证；仅在用户完成输入后才显示错误 (MD)
- `password-toggle`- 使用语义输入类型（电子邮件、电话、号码）触发正确的移动键盘（HIG、MD）
- `autofill-support`- 为密码字段提供显示/隐藏切换功能 (MD)
- `undo-support`- 使用自动完成/textContentType 属性，以便系统自动填写（HIG、MD）
- `success-feedback`- 允许撤销破坏性或批量操作（如 "撤销删除 "吐司）（Apple HIG）
- `error-recovery`- 通过简短的视觉反馈（复选标记、吐司、彩色闪光）确认已完成的操作（MD）
- `multi-step-progress`- 错误信息必须包括明确的恢复路径（重试、编辑、帮助链接）（HIG，MD）
- `form-autosave`- 多步骤流程显示步骤指示器或进度条；允许返回导航 (MD)
- `sheet-dismiss-confirm`- 长表格应自动保存草稿，以防意外删除时丢失数据（Apple HIG）
- `error-clarity`- 在取消有未保存更改的工作表/模式之前进行确认（Apple HIG）
- `field-grouping`- 错误信息必须说明原因和解决方法（而不仅仅是 "输入无效"）（HIG，MD）
- `read-only-distinction`- 对相关字段进行逻辑分组（字段集/图例或可视化分组）（MD）
- `focus-management`- 只读状态应与禁用（MD）状态在视觉和语义上有所区别
- `error-summary`- 提交错误后，自动聚焦第一个无效字段（WCAG，MD）
- `touch-friendly-input`- 对于多个错误，在顶部显示摘要，并带有指向每个字段的锚链接（WCAG）
- `destructive-emphasis`- 移动输入高度 ≥44px 以满足触摸目标要求（Apple HIG）
- `toast-accessibility`- 破坏性操作使用语义危险色（红色），并在视觉上与主要操作（HIG、MD）区分开来
- `aria-live-errors`- 祝酒词不得抢夺焦点；使用 aria-live="礼貌 "进行屏幕阅读器公告（WCAG）
- `contrast-feedback`- 表单错误使用 aria-live 区域或 role="alert" 通知屏幕阅读器（WCAG）
- `timeout-feedback`- 错误和成功状态的颜色必须符合 4.5:1 的对比度（WCAG，MD）

### \- 请求超时必须显示清晰的反馈，并带有重试选项 (MD)

- `bottom-nav-limit`9.导航模式（高）
- `drawer-usage`- 底部导航最多 5 个项目；使用带图标的标签（材料设计）
- `back-behavior`- 将抽屉/侧边栏用于二级导航，而非主要操作（材料设计）
- `deep-linking`- 后退导航必须具有可预测性和一致性；保留滚动/状态（苹果公司高级用户组，马里兰州）
- `tab-bar-ios`- 所有关键屏幕都必须可以通过深层链接/URL 进行共享和通知（Apple HIG、MD）
- `top-app-bar-android`- iOS：使用底部标签栏进行顶级导航（Apple HIG）
- `nav-label-icon`- 安卓：使用带导航图标的顶部应用栏作为主要结构（材料设计）
- `nav-state-active`- 导航项必须有图标和文本标签；仅有图标的导航会降低可发现性（MD）
- `nav-hierarchy`- 当前位置必须在导航中直观突出显示（颜色、重量、指示器）（HIG、MD）。
- `modal-escape`- 主导航（选项卡/底部栏）与次导航（抽屉/设置）必须明确分开 (MD)
- `search-accessible`- 模态和表单必须提供明确的关闭/取消功能；在移动设备上向下滑动即可取消（Apple HIG）
- `breadcrumb-web`- 搜索必须易于访问（顶部栏或选项卡）；提供最近/建议的查询 (MD)
- `state-preservation`- 网络：在 3 层以上的深层结构中使用面包屑，以帮助定位（MD）
- `gesture-nav-support`- 回退时必须恢复先前的滚动位置、滤波器状态和输入（高频、中频）。
- `tab-badge`- 支持系统手势导航（iOS 向后轻扫、Android 向后预测），不会发生冲突（HIG、MD）
- `overflow-menu`- 在导航项上少用徽章，以表示未读/待读；用户访问后清除（HIG、MD）。
- `bottom-nav-top-level`- 当操作超出可用空间时，使用溢出/更多菜单，而不是填满 (MD)
- `adaptive-navigation`- 底部导航仅供顶层屏幕使用；切勿在其内部嵌套子导航（MD）
- `back-stack-integrity`- 大屏幕（≥1024px）更喜欢侧边栏；小屏幕使用底部/顶部导航（自适应材质）
- `navigation-consistency`- 切勿静默重置导航堆栈或意外跳至原点（HIG，MD）
- `avoid-mixed-patterns`- 所有页面的导航位置必须保持一致，不得因页面类型而改变
- `modal-vs-navigation`- 不要将 Tab + 侧边栏 + 底部导航混合在同一层次结构中
- `focus-on-route-change`- 主导航流不得使用模态；它们会破坏用户的路径（HIG）
- `persistent-nav`- 页面过渡后，将焦点移至主要内容区域，以便屏幕阅读器用户使用（WCAG）
- `destructive-nav-separation`- 核心导航必须能够从深层页面到达；不要将其完全隐藏在子流程中（HIG，MD）
- `empty-nav-state`- 危险操作（删除账户、注销）必须与正常导航项目在视觉上和空间上分开（HIG、MD）

### \- 当导航目的地不可用时，请解释原因，而不是默默地将其隐藏（MD）

- `chart-type`10.图表和数据（低）
- `color-guidance`- 根据数据类型匹配图表类型（趋势图 → 直线图、比较图 → 柱状图、比例图 → 饼图/圆锥图）
- `data-table`- 使用无障碍调色板；避免只为色盲用户提供红/绿配色（《通用行为规范》，MD）。
- `pattern-texture`- 提供表格替代品，以实现无障碍访问；图表本身对屏幕阅读器不友好（WCAG）
- `legend-visible`- 用图案、纹理或形状来补充颜色，使数据在没有颜色的情况下也能辨别（《世界数据采集格式》，MD）
- `tooltip-on-interact`- 始终显示图例；位置靠近图表，不脱离滚动折页 (MD)
- `axis-labels`- 在悬停（网页）或点击（手机）时提供工具提示/数据标签，显示精确值（HIG、MD）
- `responsive-chart`- 在坐标轴上标注单位和可读刻度；避免在移动设备上截断或旋转标签
- `empty-data-state`- 在小屏幕上，图表必须回流或简化（例如，用横条代替竖条，减少刻度）。
- `loading-chart`- 无数据时显示有意义的空白状态（"尚无数据 "+引导），而不是空白图表 (MD)
- `animation-optional`- 加载图表数据时使用骨架或闪烁占位符；不显示空轴框架
- `large-dataset`- 图表入口动画必须尊重 "优先缩减动作"；数据应能立即读取（高分辨率）。
- `number-formatting`- 对于 1000 个以上的数据点，进行汇总或抽样；提供详细信息的下钻功能，而不是显示所有数据 (MD)
- `touch-target-chart`- 对坐标轴和标签上的数字、日期和货币使用本地识别格式（HIG、MD）
- `no-pie-overuse`- 交互式图表元素（点、线段）必须有 ≥44pt 的点击区域，或在触摸时展开（Apple HIG）
- `contrast-data`- 大于 5 个类别时，避免使用饼图/圆锥图；为清晰起见，改用条形图
- `legend-interactive`- 数据线/条与背景对比 ≥3:1；数据文本标签 ≥4.5:1 (WCAG)
- `direct-labeling`- 图例应可点击以切换系列可见性（MD）
- `tooltip-keyboard`- 对于小数据集，可直接在图表上标注数值，以减少视线的移动
- `sortable-table`- 工具提示内容必须是键盘可触及的，而不能仅仅依靠悬停（WCAG）
- `axis-readability`- 数据表必须支持使用 aria-sort 显示当前排序状态的排序方式（WCAG）
- `data-density`- 坐标轴刻度不要太紧凑；保持可读的间距，在小屏幕上自动跳过
- `trend-emphasis`- 限制每张图表的信息密度，避免认知负担过重；如有必要，可分成多张图表
- `gridline-subtle`- 强调数据趋势而非装饰；避免遮盖数据的厚重梯度/阴影
- `focusable-elements`- 网格线的对比度应低（如灰色-200），以免与数据产生冲突
- `screen-reader-summary`- 交互式图表元素（点、条、片）必须可通过键盘导航（WCAG）
- `error-state-chart`- 为屏幕阅读器（WCAG）提供文本摘要或 aria 标签，描述图表的主要内容
- `export-option`- 数据加载失败必须显示错误信息和重试操作，而不是图表破损/为空
- `drill-down-consistency`- 对于数据量大的产品，提供 CSV/图像图表数据导出功能
- `time-scale-clarity`- 向下钻取交互必须保持清晰的回溯路径和层次面包屑

## \- 时间序列图表必须明确标注时间粒度（日/周/月）并允许切换

\- 时间序列图表必须明确标注时间粒度（日/周/月）并允许切换

-----

## 如何使用

使用下面的 CLI 工具搜索特定域。

``` bash
python3 --version || python --version
```

先决条件

**检查是否安装了 Python：**

``` bash
brew install python3
```

**Ubuntu/Debian:**

``` bash
sudo apt update && sudo apt install python3
```

**如果未安装 Python，请根据用户的操作系统安装：**

``` powershell
winget install Python.Python.3.12
```

-----

## macOS

视窗

| Scenario | Trigger Examples | Start From |
| --- | --- | --- |
| **New project / page** | "Build a landing page", "Build a dashboard" | Step 1 → Step 2 (design system) |
| **New component** | "Create a pricing card", "Add a modal" | Step 3 (domain search: style, ux) |
| **Choose style / color / font** | "What style fits a fintech app?", "Recommend a color palette" | Step 2 (design system) |
| **Review existing UI** | "Review this page for UX issues", "Check accessibility" | Quick Reference checklist above |
| **Fix a UI bug** | "Button hover is broken", "Layout shifts on load" | Quick Reference → relevant section |
| **Improve / optimize** | "Make this faster", "Improve mobile experience" | Step 3 (domain search: ux, react) |
| **Implement dark mode** | "Add dark mode support" | Step 3 (domain: style "dark mode") |
| **Add charts / data viz** | "Add an analytics dashboard chart" | Step 3 (domain: chart) |
| **Stack best practices** | "React performance tips"、"SwiftUI navigation" | Step 4 (stack search) |

如何使用这项技能

### 当用户要求使用以下技能时，请使用此技能：

请按照此工作流程操作：

- **从用户请求中提取关键信息：**
- **产品类型娱乐（社交、视频、音乐、游戏）、工具（扫描仪、编辑器、转换器）、生产力（任务管理器、便笺、日历）或混合型**
- **目标受众：C 端消费者用户；考虑年龄组、使用环境（通勤、休闲、工作）**
- **风格关键词：俏皮、活力、极简、暗模式、内容优先、身临其境等。**

### 堆栈：React Native（本项目唯一的技术栈）

**堆栈：React Native（本项目唯一的技术栈）`--design-system`**

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "<product_type> <industry> <keywords>" --design-system [-p "Project Name"]
```

步骤 2：生成设计系统（必填）

1. 该命令
2. 并行搜索域（产品、风格、颜色、着陆、排版）`ui-reasoning.csv`
3. 应用推理规则从中选择最佳匹配项
4. 返回完整的设计系统：图案、风格、色彩、排版、效果

**包括应避免的反模式**

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "beauty spa wellness service" --design-system -p "Serenity Spa"
```

### 包括应避免的反模式

例如****`--persist`

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "<query>" --design-system --persist -p "Project Name"
```

步骤 2b：坚持设计系统（主模式 + 重写模式）

- `design-system/MASTER.md`这就产生了
- `design-system/pages/`- 包含所有设计规则的全球真理之源

**- 用于特定页面重载的文件夹**

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "<query>" --design-system --persist -p "Project Name" --page "dashboard"
```

\- 用于特定页面重载的文件夹

- `design-system/pages/dashboard.md`这也产生了

**- 特定页面与母版的偏差**

1. 分层检索的工作原理`design-system/pages/checkout.md`
2. 在创建特定页面（如 "结账"）时，首先检查****
3. 如果页面文件存在，其规则优先于主文件`design-system/MASTER.md`

**如果没有，则专门使用**

    I am building the [Page Name] page. Please read design-system/MASTER.md.
    Also check if design-system/pages/[page-name].md exists.
    If the page file exists, prioritize its rules.
    If not, use the Master rules exclusively.
    Now, generate the code...

### 如果没有，则专门使用

情境感知检索提示：

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "<keyword>" --domain <domain> [-n <max_results>]
```

**第 3 步：补充详细搜索（根据需要）**

| Need | Domain | Example |
| --- | --- | --- |
| Product type patterns | `product` | `--domain product "entertainment social"` |
| More style options | `style` | `--domain style "glassmorphism dark"` |
| Color palettes | `color` | `--domain color "entertainment vibrant"` |
| Font pairings | `typography` | `--domain typography "playful modern"` |
| Chart recommendations | `chart` | `--domain chart "real-time dashboard"` |
| UX best practices | `ux` | `--domain ux "animation accessibility"` |
| Alternative fonts | `typography` | `--domain typography "elegant luxury"` |
| Individual Google Fonts | `google-fonts` | `--domain google-fonts "sans serif popular variable"` |
| Landing structure | `landing` | `--domain landing "hero social-proof"` |
| React Native perf | `react` | `--domain react "rerender memo list"` |
| App interface a11y | `web` | `--domain web "accessibilityLabel touch safe-areas"` |
| AI prompt / CSS keywords | `prompt` | `--domain prompt "minimalism"` |

### 获得设计系统后，使用域搜索来获取更多详细信息：

何时使用详细搜索：

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "<keyword>" --stack react-native
```

-----

## 步骤 4：堆栈指南（React Native）

### 获取针对 React Native 实施的最佳实践：

| Domain | Use For | Example Keywords |
| --- | --- | --- |
| `product` | Product type recommendations | SaaS, e-commerce, portfolio, healthcare, beauty, service |
| `style` | UI styles, colors, effects | glassmorphism, minimalism, dark mode, brutalism |
| `typography` | Font pairings, Google Fonts | elegant, playful, professional, modern |
| `color` | Color palettes by product type | saas, ecommerce, healthcare, beauty, fintech, service |
| `landing` | Page structure, CTA strategies | hero, hero-centric, testimonial, pricing, social-proof |
| `chart` | Chart types, library recommendations | trend, comparison, timeline, funnel, pie |
| `ux` | Best practices, anti-patterns | animation, accessibility, z-index, loading |
| `google-fonts` | Individual Google Fonts lookup | sans serif, monospace, japanese, variable font, popular |
| `react` | React/Next.js performance | waterfall, bundle, suspense, memo, rerender, cache |
| `web` | App interface guidelines (iOS/Android/React Native) | accessibilityLabel, touch targets, safe areas, Dynamic Type |
| `prompt` | AI prompts, CSS keywords | (style name) |

### 搜索参考

| Stack | Focus |
| --- | --- |
| `react-native` | Components, Navigation, Lists |

-----

## 可用域名

**可用堆栈**

### 工作流程示例

- 步骤 1：分析需求
- 产品类型：工具（人工智能搜索引擎）
- 目标受众寻求快速、智能搜索的 C 端用户
- 风格关键词：现代、简约、内容优先、暗模式

### 堆栈反应原生

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "AI search tool modern minimal" --design-system -p "AI Search"
```

**堆栈反应原生**

### 步骤 2：生成设计系统（必填）

``` bash
# Get style options for a modern tool product
python3 skills/ui-ux-pro-max/scripts/search.py "minimalism dark mode" --domain style

# Get UX best practices for search interaction and loading
python3 skills/ui-ux-pro-max/scripts/search.py "search loading animation" --domain ux
```

### 输出：完整的设计系统，包括模式、风格、色彩、排版、效果和反模式。

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "list performance navigation" --stack react-native
```

**第 3 步：补充详细搜索（根据需要）**

-----

## 步骤 4：堆垛指南

然后：综合设计系统 + 详细搜索并实施设计。`--design-system`

``` bash
# ASCII box (default) - best for terminal display
python3 skills/ui-ux-pro-max/scripts/search.py "fintech crypto" --design-system

# Markdown - best for documentation
python3 skills/ui-ux-pro-max/scripts/search.py "fintech crypto" --design-system -f markdown
```

-----

## 输出格式

### 该标志支持两种输出格式：

- 查询策略****`"entertainment social vibrant content-dense"``"app"`
- 使用多维关键词 - 结合产品 + 行业 + 语调 + 密度：而不仅仅是`"playful neon"``"vibrant dark"``"content-first minimal"`
- 针对同一需求尝试不同的关键词：  → →`--design-system``--domain`
- 首先用于全面推荐，然后深入研究您不确定的任何维度`--stack react-native`

### 始终添加针对具体实施的指导

| Problem | What to Do |
| --- | --- |
| Can't decide on style/color | Re-run `--design-system` with different keywords |
| Dark mode contrast issues | Quick Reference §6: `color-dark-mode` + `color-accessible-pairs` |
| Animations feel unnatural | Quick Reference §7: `spring-physics` + `easing` + `exit-faster-than-enter` |
| Form UX is poor | Quick Reference §8: `inline-validation` + `error-clarity` + `focus-management` |
| Navigation feels confusing | Quick Reference §9: `nav-hierarchy` + `bottom-nav-limit` + `back-behavior` |
| Layout breaks on small screens | Quick Reference §5: `mobile-first` + `breakpoint-consistency` |
| Performance / jank | Quick Reference §3: `virtualize-lists` + `main-thread-budget` + `debounce-throttle` |

### 始终添加针对具体实施的指导

- 交货前核对表`--domain ux "animation accessibility z-index loading"`
- 在实施前作为用户体验验证程序运行****
- 对《快速参考》§1-§3（CRITICAL + HIGH）进行最后审查
- 在 375px（小型手机）和横向上进行测试********
- 验证启用减小运动和最大尺寸动态类型后的行为
- 独立检查暗模式对比度（不要假设亮模式值有效）

-----

## 确认所有触摸目标 ≥44pt，且安全区域后无隐藏内容

确认所有触摸目标 ≥44pt，且安全区域后无隐藏内容

### 专业用户界面的通用规则

| Rule | Standard | Avoid | Why It Matters |
| --- | --- | --- | --- |
| **No Emoji as Structural Icons** | Use vector-based icons (e.g., Lucide, react-native-vector-icons, @expo/vector-icons). | Using emojis (🎨 🚀 ⚙️) for navigation, settings, or system controls. | Emojis are font-dependent, inconsistent across platforms, and cannot be controlled via design tokens. |
| **Vector-Only Assets** | Use SVG or platform vector icons that scale cleanly and support theming. | Raster PNG icons that blur or pixelate. | Ensures scalability, crisp rendering, and dark/light mode adaptability. |
| **Stable Interaction States** | Use color, opacity, or elevation transitions for press states without changing layout bounds. | Layout-shifting transforms that move surrounding content or trigger visual jitter. | Prevents unstable interactions and preserves smooth motion/perceived quality on mobile. |
| **Correct Brand Logos** | Use official brand assets and follow their usage guidelines (spacing, color, clear space). | Guessing logo paths, recoloring unofficially, or modifying proportions. | Prevents brand misuse and ensures legal/platform compliance. |
| **Consistent Icon Sizing** | Define icon sizes as design tokens (e.g., icon-sm, icon-md = 24pt, icon-lg). | Mixing arbitrary values like 20pt / 24pt / 28pt randomly. | Maintains rhythm and visual hierarchy across the interface. |
| **Stroke Consistency** | Use a consistent stroke width within the same visual layer (e.g., 1.5px or 2px). | Mixing thick and thin stroke styles arbitrarily. | Inconsistent strokes reduce perceived polish and cohesion. |
| **Filled vs Outline Discipline** | Use one icon style per hierarchy level. | Mixing filled and outline icons at the same hierarchy level. | Maintains semantic clarity and stylistic coherence. |
| **Touch Target Minimum** | Minimum 44×44pt interactive area (use hitSlop if icon is smaller). | Small icons without expanded tap area. | Meets accessibility and platform usability standards. |
| **Icon Alignment** | Align icons to text baseline and maintain consistent padding. | Misaligned icons or inconsistent spacing around them. | Prevents subtle visual imbalance that reduces perceived quality. |
| **Icon Contrast** | Follow WCAG contrast standards: 4.5:1 for small elements, 3:1 minimum for larger UI glyphs. | Low-contrast icons that blend into the background. | Ensures accessibility in both light and dark modes. |

### 这些经常被忽视的问题会让用户界面显得不专业：注意范围：以下规则适用于应用程序用户界面（iOS/Android/React Native/Flutter），而非桌面网页交互模式。

| Rule | Do | Don't |
| --- | --- | --- |
| **Tap feedback** | Provide clear pressed feedback (ripple/opacity/elevation) within 80-150ms | No visual response on tap |
| **Animation timing** | Keep micro-interactions around 150-300ms with platform-native easing | Instant transitions or slow animations (\>500ms) |
| **Accessibility focus** | Ensure screen reader focus order matches visual order and labels are descriptive | Unlabeled controls or confusing focus traversal |
| **Disabled state clarity** | Use disabled semantics (`disabled`/native disabled props), reduced emphasis, and no tap action | Controls that look tappable but do nothing |
| **Touch target minimum** | Keep tap areas \>=44x44pt (iOS) or \>=48x48dp (Android), expand hit area when icon is smaller | Tiny tap targets or icon-only hit areas without padding |
| **Gesture conflict prevention** | Keep one primary gesture per region and avoid nested tap/drag conflicts | Overlapping gestures causing accidental actions |
| **Semantic native controls** | Prefer native interactive primitives (`Button`, `Pressable`, platform equivalents) with proper accessibility roles | Generic containers used as primary controls without semantics |

### 图标和视觉元素

| Rule | Do | Don't |
| --- | --- | --- |
| **Surface readability (light)** | Keep cards/surfaces clearly separated from background with sufficient opacity/elevation | Overly transparent surfaces that blur hierarchy |
| **Text contrast (light)** | Maintain body text contrast \>=4.5:1 against light surfaces | Low-contrast gray body text |
| **Text contrast (dark)** | Maintain primary text contrast \>=4.5:1 and secondary text \>=3:1 on dark surfaces | Dark mode text that blends into background |
| **Border and divider visibility** | Ensure separators are visible in both themes (not just light mode) | Theme-specific borders disappearing in one mode |
| **State contrast parity** | Keep pressed/focused/disabled states equally distinguishable in light and dark themes | Defining interaction states for one theme only |
| **Token-driven theming** | Use semantic color tokens mapped per theme across app surfaces/text/icons | Hardcoded per-screen hex values |
| **Scrim and modal legibility** | Use a modal scrim strong enough to isolate foreground content (typically 40-60% black) | Weak scrim that leaves background visually competing |

### 互动（应用程序）

| Rule | Do | Don't |
| --- | --- | --- |
| **Safe-area compliance** | Respect top/bottom safe areas for all fixed headers, tab bars, and CTA bars | Placing fixed UI under notch, status bar, or gesture area |
| **System bar clearance** | Add spacing for status/navigation bars and gesture home indicator | Let tappable content collide with OS chrome |
| **Consistent content width** | Keep predictable content width per device class (phone/tablet) | Mixing arbitrary widths between screens |
| **8dp spacing rhythm** | Use a consistent 4/8dp spacing system for padding/gaps/section spacing | Random spacing increments with no rhythm |
| **Readable text measure** | Keep long-form text readable on large devices (avoid edge-to-edge paragraphs on tablets) | Full-width long text that hurts readability |
| **Section spacing hierarchy** | Define clear vertical rhythm tiers (e.g., 16/24/32/48) by hierarchy | Similar UI levels with inconsistent spacing |
| **Adaptive gutters by breakpoint** | Increase horizontal insets on larger widths and in landscape | Same narrow gutter on all device sizes/orientations |
| **Scroll and fixed element coexistence** | Add bottom/top content insets so lists are not hidden behind fixed bars | Scroll content obscured by sticky headers/footers |

-----

## 亮/暗模式对比度

布局和间距

### 交货前核对表

- [ ] 在交付 UI 代码之前，请验证这些项目：范围说明：本清单适用于应用程序用户界面（iOS/Android/React Native/Flutter）。
- [ ] 视觉质量
- [ ] 不使用表情符号作为图标（使用 SVG 代替）
- [ ] 所有图标均采用统一的图标系列和风格
- [ ] 官方品牌资产的使用比例正确、空间清晰

### 按压状态视觉效果不会移动布局边界或导致抖动

- [ ] 统一使用语义主题标记（没有按屏幕临时硬编码的颜色）
- [ ] 互动
- [ ] 所有可拍打元素都能提供清晰的按压反馈（波纹/透明度/高度）
- [ ] 触摸目标符合最小尺寸（\>=44x44pt iOS，\>=48x48dp Android）
- [ ] 微交互时间保持在 150-300ms 范围内，具有原生感觉的缓和效果
- [ ] 禁用状态在视觉上清晰可见，且不交互

### 屏幕阅读器的重点顺序与视觉顺序一致，交互式标签具有描述性

- [ ] 手势区域可避免嵌套/冲突交互（点击/拖动/后滑冲突）
- [ ] 亮/暗模式
- [ ] 在明暗模式下，主要文字对比度 \>=4.5:1
- [ ] 在明暗模式下，二级文字对比度 \>=3:1
- [ ] 两种模式下的分隔线/边界和交互状态均可区分

### 模态/抽屉边框的不透明度要足够高，以保持前景的可读性（通常为 40-60% 黑色）

- [ ] 两个主题在交付前均经过测试（而不是从单一主题推断出）
- [ ] 布局
- [ ] 尊重标题、标签栏和底部 CTA 栏的安全区域
- [ ] 滚动内容不会隐藏在固定/粘贴条后面
- [ ] 已在小型手机、大型手机和平板电脑上验证（纵向 + 横向）
- [ ] 根据设备尺寸和方向正确调整水平嵌入/凹槽

### 4/8dp 间距的节奏在组件、章节和页面各级均得到保持

- [ ] 在较大的设备上仍可阅读长篇文本措施（无边对边段落）
- [ ] 无障碍环境
- [ ] 所有有意义的图像/图标都有无障碍标签
- [ ] 表单字段有标签、提示和清晰的错误信息
- [ ] 颜色不是唯一的指标
