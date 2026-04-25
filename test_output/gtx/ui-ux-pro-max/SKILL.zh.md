---
name: ui-ux-pro-max
description: "UI/UX design intelligence for web and mobile. Includes 50+ styles, 161 color palettes, 57 font pairings, 161 product types, 99 UX guidelines, and 25 chart types across 10 stacks (React, Next.js, Vue, Svelte, SwiftUI, React Native, Flutter, Tailwind, shadcn/ui, and HTML/CSS). Actions: plan, build, create, design, implement, review, fix, improve, optimize, enhance, refactor, and check UI/UX code. Projects: website, landing page, dashboard, admin panel, e-commerce, SaaS, portfolio, blog, and mobile app. Elements: button, modal, navbar, sidebar, card, table, form, and chart. Styles: glassmorphism, claymorphism, minimalism, brutalism, neumorphism, bento grid, dark mode, responsive, skeuomorphism, and flat design. Topics: color systems, accessibility, animation, layout, typography, font pairing, spacing, interaction states, shadow, and gradient. Integrations: shadcn/ui MCP for component search and examples."
---

# UI/UX Pro Max - 设计智能

Web 和移动应用程序的综合设计指南。包含 50 多种样式、161 种调色板、57 种字体配对、161 种带有推理规则的产品类型、99 条用户体验指南以及跨 10 个技术堆栈的 25 种图表类型。可搜索的数据库，具有基于优先级的推荐。

## 何时申请

当任务涉及 UI 结构、视觉设计决策、交互模式或用户体验质量控制时，应使用此技能。****

### 必须使用

在以下情况下必须调用此技能：

- 设计新页面（登陆页面、仪表板、管理、SaaS、移动应用程序）
- 创建或重构 UI 组件（按钮、模式、表单、表格、图表等）
- 选择配色方案、排版系统、间距标准或布局系统
- 审查 UI 代码的用户体验、可访问性或视觉一致性
- 实现导航结构、动画或响应行为
- 制定产品级设计决策（风格、信息层次、品牌表达）
- 提高界面的感知质量、清晰度或可用性

### 受到推崇的

在以下情况下推荐使用此技能：

- UI看起来“不够专业”但原因尚不清楚
- 接收有关可用性或体验的反馈
- 启动前 UI 质量优化
- 协调跨平台设计（Web / iOS / Android）
- 构建设计系统或可重用组件库

### 跳过

以下情况不需要此技能：

- 纯后端逻辑开发
- 只涉及API或数据库设计
- 与界面无关的性能优化
- 基础设施或 DevOps 工作
- 非可视化脚本或自动化任务

**决策标准：如果任务将改变某个功能的外观、感觉、移动或交互方式，则应使用此技能。******

## 按优先级划分的规则类别

*供人类/AI参考：按照优先级1→10来决定首先关注哪个规则类别；需要时用于查询详细信息。脚本不读取此表。`--domain <Domain>`*

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

## 快速参考

### 1\. 辅助功能（关键）

- `color-contrast`- 普通文本的最小比例为 4.5:1（大文本 3:1）；材料设计
- `focus-states`- 交互元素上的可见聚焦环（2–4 像素；Apple HIG，MD）
- `alt-text`- 有意义的图像的描述性替代文本
- `aria-labels`- 仅图标按钮的 aria-label；本机中的accessibilityLabel（Apple HIG）
- `keyboard-nav`- Tab 键顺序与视觉顺序一致；全键盘支持（Apple HIG）
- `form-labels`- 使用带有 for 属性的标签
- `skip-links`- 跳至键盘用户的主要内容
- `heading-hierarchy`- 顺序h1→h6，无级别跳跃
- `color-not-only`- 不要仅通过颜色传达信息（添加图标/文本）
- `dynamic-type`- 支持系统文字缩放；避免随着文本增长而截断（Apple Dynamic Type，MD）
- `reduced-motion`- 尊重优先于减少动作；根据请求减少/禁用动画（Apple 减少运动 API，MD）
- `voiceover-sr`- 有意义的可访问性标签/可访问性提示； VoiceOver/屏幕阅读器的逻辑阅读顺序（Apple HIG、MD）
- `escape-routes`- 在模式和多步骤流程中提供取消/返回（Apple HIG）
- `keyboard-shortcuts`- 保留系统和a11y快捷方式；提供拖放键盘替代方案（Apple HIG）

### 2\. 触摸和交互（关键）

- `touch-target-size`- 最小 44×44pt（苹果）/48×48dp（材质）；如果需要，将命中区域延伸到视觉范围之外
- `touch-spacing`- 触摸目标之间的最小间隙为 8px/8dp（Apple HIG、MD）
- `hover-vs-tap`- 使用点击/点击进行主要交互；不要仅仅依赖悬停
- `loading-buttons`- 异步操作期间禁用按钮；显示微调器或进度
- `error-feedback`- 清除问题附近的错误消息
- `cursor-pointer`- 将光标指针添加到可点击元素（Web）
- `gesture-conflicts`- 避免在主要内容上水平滑动；更喜欢垂直滚动
- `tap-delay`- 使用触摸操作：减少 300 毫秒延迟的操作（Web）
- `standard-gestures`- 一致使用平台标准手势；不要重新定义（例如向后滑动、捏合缩放）（Apple HIG）
- `system-gestures`- 不要阻止系统手势（控制中心、向后滑动等）（Apple HIG）
- `press-feedback`- 按下时的视觉反馈（波纹/突出显示；MD 状态层）
- `haptic-feedback`- 使用触觉进行确认和重要行动；避免过度使用（Apple HIG）
- `gesture-alternative`- 不要依赖仅手势交互；始终为关键操作提供可见的控制
- `safe-area-awareness`- 使主要触摸目标远离凹口、动态岛、手势栏和屏幕边缘
- `no-precision-required`- 避免在小图标或细边缘上进行像素完美的点击
- `swipe-clarity`- 滑动操作必须显示清晰的可供性或提示（V 形、标签、教程）
- `drag-threshold`- 在开始拖动之前使用移动阈值以避免意外拖动

### 3\. 性能（高）

- `image-optimization`- 使用 WebP/AVIF、响应式图像（srcset/size）、延迟加载非关键资产
- `image-dimension`- 声明宽度/高度或使用宽高比来防止布局移位（核心 Web Vitals：CLS）
- `font-loading`- 使用字体显示：交换/可选以避免不可见文本（FOIT）；预留空间以减少布局偏移（MD）
- `font-preload`- 仅预加载关键字体；避免在每个变体上过度使用预载
- `critical-css`- 优先考虑首屏 CSS（内联关键 CSS 或早期加载的样式表）
- `lazy-loading`- 通过动态导入/路由级拆分延迟加载非英雄组件
- `bundle-splitting`- 按路线/功能拆分代码（React Suspense / Next.js 动态）以减少初始负载和 TTI
- `third-party-scripts`- 异步/延迟加载第三方脚本；审核并删除不必要的（MD）
- `reduce-reflows`- 避免频繁的布局读/写；批量 DOM 读取然后写入
- `content-jumping`- 为异步内容预留空间以避免布局跳转（核心 Web Vitals：CLS）
- `lazy-load-below-fold`- 对首屏图像和重媒体使用loading="lazy"
- `virtualize-lists`- 虚拟化包含 50 多个项目的列表，以提高内存效率和滚动性能
- `main-thread-budget`- 将 60fps 的每帧工作保持在约 16ms 以内；将繁重的任务移出主线程（HIG、MD）
- `progressive-loading`- 使用骨架屏幕/闪光代替长阻塞旋转器进行 \>1 秒的操作（Apple HIG）
- `input-latency`- 将点击/滚动的输入延迟保持在约 100 毫秒以下（材质响应标准）
- `tap-feedback-speed`- 在点击后 100 毫秒内提供视觉反馈（Apple HIG）
- `debounce-throttle`- 对高频事件（滚动、调整大小、输入）使用去抖/节流
- `offline-support`- 提供离线状态消息传递和基本回退（PWA /移动）
- `network-fallback`- 为慢速网络提供降级模式（较低分辨率的图像，较少的动画）

### 4\. 风格选择（HIGH）

- `style-match`- 将风格与产品类型相匹配（用于推荐）`--design-system`
- `consistency`- 在所有页面上使用相同的样式
- `no-emoji-icons`- 使用 SVG 图标（Heroicons、Lucide），而不是表情符号
- `color-palette-from-product`- 从产品/行业中选择调色板（搜索）`--domain color`
- `effects-match-style`- 阴影、模糊、半径与所选风格一致（玻璃/平面/粘土等）
- `platform-adaptive`- 尊重平台习惯用法（iOS HIG 与 Material）：导航、控件、排版、动作
- `state-clarity`- 使悬停/按下/禁用状态在视觉上清晰，同时保持风格（材质状态层）
- `elevation-consistent`- 对卡片、表格、模态使用一致的标高/阴影比例；避免随机阴影值
- `dark-mode-pairing`- 一起设计浅色/深色变体，以保持品牌、对比度和风格的一致
- `icon-style-consistent`- 在整个产品中使用一套图标集/视觉语言（描边宽度、圆角半径）
- `system-controls`- 与完全自定义的控件相比，更喜欢本机/系统控件；仅在品牌需要时进行定制 (Apple HIG)
- `blur-purpose`- 使用模糊来指示背景消除（模态、表格），而不是作为装饰（Apple HIG）
- `primary-action`- 每个屏幕应该只有一个主要 CTA；次要操作视觉上从属（Apple HIG）

### 5.布局和响应（高）

- `viewport-meta`- width=device-width initial-scale=1（永远不要禁用缩放）
- `mobile-first`- 首先设计移动设备，然后扩展到平板电脑和台式机
- `breakpoint-consistency`- 使用系统断点（例如 375 / 768 / 1024 / 1440）
- `readable-font-size`- 移动设备上的正文文本最小为 16 像素（避免 iOS 自动缩放）
- `line-length-control`- 移动每行 35–60 个字符；桌面 60–75 个字符
- `horizontal-scroll`- 移动设备上没有水平滚动条；确保内容适合视口宽度
- `spacing-scale`- 使用 4pt/8dp 增量间距系统（材质设计）
- `touch-density`- 保持元件间距舒适，便于触摸：不拥挤，不会导致误点击
- `container-width`- 桌面上一致的最大宽度（max-w-6xl / 7xl）
- `z-index-management`- 定义分层 z 索引比例（例如 0 / 10 / 20 / 40 / 100 / 1000）
- `fixed-element-offset`- 修复了导航栏/底部栏必须为底层内容保留安全填充的问题
- `scroll-behavior`- 避免干扰主滚动体验的嵌套滚动区域
- `viewport-units`- 在移动设备上更喜欢 min-h-dvh 而不是 100vh
- `orientation-support`- 保持布局在横向模式下可读和可操作
- `content-priority`- 在移动端优先展示核心内容；折叠或隐藏次要内容
- `visual-hierarchy`- 通过大小、间距、对比度建立层次结构——而不仅仅是颜色

### 6\. 版式和颜色（中）

- `line-height`- 正文使用 1.5-1.75
- `line-length`- 每行限制 65-75 个字符
- `font-pairing`- 匹配标题/正文字体个性
- `font-scale`- 一致的字体比例（例如 12 14 16 18 24 32）
- `contrast-readability`- 浅色背景上的深色文本（例如白色上的 slate-900）
- `text-styles-system`- 使用平台类型系统：iOS 11 动态类型样式/材质 5 种类型角色（显示、标题、标题、正文、标签）（HIG、MD）
- `weight-hierarchy`- 使用字体粗细来强化层次结构：粗体标题 (600–700)、常规正文 (400)、中等标签 (500) (MD)
- `color-semantic`- 定义组件中的语义颜色标记（主要、次要、错误、表面、表面）而不是原始十六进制（材质颜色系统）
- `color-dark-mode`- 深色模式使用去饱和/较浅的色调变体，而不是反转颜色；单独测试对比度（HIG、MD）
- `color-accessible-pairs`- 前景/背景对必须满足 4.5:1 (AA) 或 7:1 (AAA)；使用工具验证（WCAG、MD）
- `color-not-decorative-only`- 功能颜色（错误红色，成功绿色）必须包含图标/文本；避免仅颜色含义（HIG、MD）
- `truncation-strategy`- 优先选择换行而不是截断；截断时使用省略号并通过工具提示/展开提供全文（Apple HIG）
- `letter-spacing`- 尊重每个平台的默认字母间距；避免对正文进行严格跟踪（HIG、MD）
- `number-tabular`- 对数据列、价格和计时器使用表格/等宽数字，以防止布局变化
- `whitespace-balance`- 有意使用空格对相关项目进行分组并分隔部分；避免视觉混乱（Apple HIG）

### 7.动画（中）

- `duration-timing`- 使用 150–300ms 进行微交互；复杂转换≤400ms；避免 \>500ms (MD)
- `transform-performance`- 仅使用变换/不透明度；避免设置宽度/高度/顶部/左侧的动画
- `loading-states`- 加载超过 300 毫秒时显示框架或进度指示器
- `excessive-motion`- 每个视图最多动画 1-2 个关键元素
- `easing`- 使用ease-out进入，ease-in退出；避免 UI 过渡呈线性
- `motion-meaning`- 每个动画都必须表达因果关系，而不仅仅是装饰性（Apple HIG）
- `state-transition`- 状态变化（悬停/活动/展开/折叠/模态）应该平滑地动画化，而不是快速动画
- `continuity`- 页面/屏幕过渡应保持空间连续性（共享元素、定向滑动）（Apple HIG）
- `parallax-subtle`- 谨慎使用视差；必须遵循简化运动且不会导致迷失方向（Apple HIG）
- `spring-physics`- 更喜欢基于弹簧/物理的曲线而不是线性或三次贝塞尔曲线以获得自然的感觉（Apple HIG 流体动画）
- `exit-faster-than-enter`- 退出动画比输入短（输入持续时间的约 60–70%）以感觉响应（MD 运动）
- `stagger-sequence`- 每个项目错开列表/网格项目入口 30–50 毫秒；避免一次性全部或太慢的揭示 (MD)
- `shared-element-transition`- 使用共享元素/英雄过渡来实现屏幕之间的视觉连续性（MD、HIG）
- `interruptible`- 动画必须是可中断的；用户点击/手势立即取消正在进行的动画（Apple HIG）
- `no-blocking-animation`- 切勿在动画期间阻止用户输入； UI 必须保持交互（Apple HIG）
- `fade-crossfade`- 使用交叉淡入淡出来替换同一容器内的内容 (MD)
- `scale-feedback`- 按下可点击卡片/按钮时的细微刻度 (0.95–1.05)；释放时恢复（HIG、MD）
- `gesture-feedback`- 拖动、滑动和捏合必须提供跟踪手指的实时视觉响应（MD Motion）
- `hierarchy-motion`- 使用平移/缩放方向来表达层次结构：从下面进入=更深，向上退出=返回（MD）
- `motion-consistency`- 在全球范围内统一久期/宽松代币；所有动画都有相同的节奏和感觉
- `opacity-threshold`- 褪色元素的不透明度不应低于 0.2；完全褪色或保持可见
- `modal-motion`- 模态/表格应根据空间上下文（HIG、MD）的触发源（缩放+淡入或滑入）进行动画处理
- `navigation-direction`- 向前导航向左/向上动画；向后向右/向下动画 - 保持方向逻辑一致（HIG）
- `layout-shift-avoid`- 动画不得导致布局回流或 CLS；使用变换来改变位置

### 8\. 表格和反馈（中）

- `input-labels`- 每个输入的可见标签（不仅仅是占位符）
- `error-placement`- 在相关字段下方显示错误
- `submit-feedback`- 加载然后提交时的成功/错误状态
- `required-indicators`- 标记必填字段（例如星号）
- `empty-states`- 无内容时的有用消息和操作
- `toast-dismiss`- 3-5 秒内自动关闭 toast
- `confirmation-dialogs`- 破坏性行为前确认
- `input-helper-text`- 在复杂输入下方提供持久的帮助文本，而不仅仅是占位符（材料设计）
- `disabled-states`- 禁用元素使用降低的不透明度 (0.38–0.5) + 光标更改 + 语义属性 (MD)
- `progressive-disclosure`- 逐步揭示复杂的选项；不要预先让用户不知所措（Apple HIG）
- `inline-validation`- 验证模糊（不是按键）；仅在用户完成输入后显示错误（MD）
- `input-type-keyboard`- 使用语义输入类型（电子邮件、电话、号码）触发正确的移动键盘（HIG、MD）
- `password-toggle`- 提供密码字段的显示/隐藏切换（MD）
- `autofill-support`- 使用自动完成/textContentType 属性，以便系统可以自动填充（HIG、MD）
- `undo-support`- 允许撤消破坏性或批量操作（例如“撤消删除”吐司）（Apple HIG）
- `success-feedback`- 通过简短的视觉反馈（复选标记、吐司、颜色闪烁）确认已完成的操作 (MD)
- `error-recovery`- 错误消息必须包含明确的恢复路径（重试、编辑、帮助链接）（HIG、MD）
- `multi-step-progress`- 多步骤流程显示步骤指示器或进度条；允许后退导航 (MD)
- `form-autosave`- 长表单应自动保存草稿，以防止意外解雇时数据丢失（Apple HIG）
- `sheet-dismiss-confirm`- 在关闭具有未保存更改的工作表/模式之前确认（Apple HIG）
- `error-clarity`- 错误消息必须说明原因 + 如何修复（不仅仅是“无效输入”）（HIG、MD）
- `field-grouping`- 对相关字段进行逻辑分组（字段集/图例或视觉分组）（MD）
- `read-only-distinction`- 只读状态在视觉和语义上应与禁用（MD）不同
- `focus-management`- 提交错误后，自动聚焦第一个无效字段（WCAG、MD）
- `error-summary`- 对于多个错误，在顶部显示摘要，并带有每个字段的锚链接 (WCAG)
- `touch-friendly-input`- 移动输入高度≥44px以满足触摸目标要求（Apple HIG）
- `destructive-emphasis`- 破坏性行为使用语义危险颜色（红色），并在视觉上与主要行为（HIG、MD）分开
- `toast-accessibility`- 祝酒词不得抢走焦点；使用 aria-live="polite" 进行屏幕阅读器公告 (WCAG)
- `aria-live-errors`- 表单错误使用 aria-live 区域或 role="alert" 通知屏幕阅读器 (WCAG)
- `contrast-feedback`- 错误和成功状态颜色必须满足 4.5:1 对比度（WCAG、MD）
- `timeout-feedback`- 请求超时必须显示带有重试选项的清晰反馈（MD）

### 9\. 导航模式（高）

- `bottom-nav-limit`- 底部导航最多 5 项；使用带有图标的标签（材料设计）
- `drawer-usage`- 使用抽屉/侧边栏进行辅助导航，而不是主要操作（材质设计）
- `back-behavior`- 后退导航必须是可预测的且一致的；保留滚动/状态（Apple HIG、MD）
- `deep-linking`- 所有关键屏幕必须可通过深层链接/URL 访问以进行共享和通知（Apple HIG、MD）
- `tab-bar-ios`- iOS：使用底部选项卡栏进行顶级导航（Apple HIG）
- `top-app-bar-android`- Android：使用带有导航图标的顶部应用栏作为主要结构（材料设计）
- `nav-label-icon`- 导航项必须同时具有图标和文本标签；仅图标导航会损害可发现性 (MD)
- `nav-state-active`- 当前位置必须在导航中以视觉方式突出显示（颜色、重量、指示器）（HIG、MD）
- `nav-hierarchy`- 主要导航（选项卡/底栏）与辅助导航（抽屉/设置）必须清晰分开（MD）
- `modal-escape`- 模态框和表单必须提供明确的关闭/关闭功能；在移动设备上向下滑动即可关闭 (Apple HIG)
- `search-accessible`- 搜索必须易于访问（顶部栏或选项卡）；提供最近/建议的查询 (MD)
- `breadcrumb-web`- Web：使用面包屑进行 3 级以上的深层层次结构以帮助定位（MD）
- `state-preservation`- 向后导航必须恢复之前的滚动位置、过滤器状态和输入（HIG、MD）
- `gesture-nav-support`- 支持系统手势导航（iOS 滑动返回、Android 预测返回）无冲突（HIG、MD）
- `tab-badge`- 谨慎地在导航项目上使用徽章来指示未读/待处理；用户访问后清除（HIG、MD）
- `overflow-menu`- 当操作超出可用空间时，使用溢出/更多菜单而不是临时抱佛脚（MD）
- `bottom-nav-top-level`- 底部导航仅适用于顶级屏幕；永远不要在其中嵌套子导航（MD）
- `adaptive-navigation`- 大屏幕（≥1024px）更喜欢侧边栏；小屏幕使用底部/顶部导航（材质自适应）
- `back-stack-integrity`- 切勿默默重置导航堆栈或意外跳转到主页（HIG、MD）
- `navigation-consistency`- 所有页面的导航位置必须保持相同；不要按页面类型更改
- `avoid-mixed-patterns`- 不要在同一层次结构级别混合使用选项卡+侧边栏+底部导航
- `modal-vs-navigation`- 模态不得用于主要导航流程；他们破坏了用户的路径（HIG）
- `focus-on-route-change`- 页面转换后，将焦点移至屏幕阅读器用户的主要内容区域 (WCAG)
- `persistent-nav`- 核心导航必须保持可从深层页面访问；不要将其完全隐藏在子流程中（HIG、MD）
- `destructive-nav-separation`- 危险操作（删除帐户、注销）必须在视觉上和空间上与正常导航项目（HIG、MD）分开
- `empty-nav-state`- 当导航目的地不可用时，解释原因而不是默默地隐藏它（MD）

### 10.图表和数据（低）

- `chart-type`- 将图表类型与数据类型相匹配（趋势 → 折线、比较 → 条形图、比例 → 饼图/圆环图）
- `color-guidance`- 使用可访问的调色板；避免色盲用户仅使用红色/绿色对（WCAG、MD）
- `data-table`- 提供表格替代方案以实现可访问性；图表本身并不适合屏幕阅读器 (WCAG)
- `pattern-texture`- 用图案、纹理或形状补充颜色，以便数据无需颜色即可区分（WCAG、MD）
- `legend-visible`- 始终展现传奇；靠近图表的位置，未在滚动折叠下方分离 (MD)
- `tooltip-on-interact`- 在悬停（Web）或点击（移动）时提供工具提示/数据标签，显示精确值（HIG、MD）
- `axis-labels`- 用单位和可读刻度标记轴；避免移动设备上的标签被截断或旋转
- `responsive-chart`- 图表必须在小屏幕上重排或简化（例如水平条而不是垂直条，更少的刻度）
- `empty-data-state`- 当不存在数据时显示有意义的空状态（“尚无数据”+指导），而不是空白图表（MD）
- `loading-chart`- 加载图表数据时使用骨架或微光占位符；不显示空轴框架
- `animation-optional`- 图表入口动画必须尊重首选减少运动；数据应立即可读（HIG）
- `large-dataset`- 对于 1000 多个数据点、汇总或样本；提供详细信息的向下钻取，而不是渲染全部 (MD)
- `number-formatting`- 对轴和标签上的数字、日期、货币使用区域设置感知格式（HIG、MD）
- `touch-target-chart`- 交互式图表元素（点、线段）必须具有 ≥44pt 的点击区域或在触摸时扩展（Apple HIG）
- `no-pie-overuse`- 避免超过 5 个类别的馅饼/甜甜圈；为了清晰起见，切换到条形图
- `contrast-data`- 数据线/条与背景≥3:1；数据文本标签≥4.5:1 (WCAG)
- `legend-interactive`- 图例应该可点击以切换系列可见性（MD）
- `direct-labeling`- 对于小型数据集，直接在图表上标记值以减少视线移动
- `tooltip-keyboard`- 工具提示内容必须是键盘可访问的，不能仅依赖悬停 (WCAG)
- `sortable-table`- 数据表必须支持使用 aria-sort 指示当前排序状态 (WCAG) 进行排序
- `axis-readability`- 轴刻度不得狭窄；保持可读间距，在小屏幕上自动跳过
- `data-density`- 限制每个图表的信息密度，以避免认知过载；如果需要的话分成多个图表
- `trend-emphasis`- 强调数据趋势而非装饰；避免使数据变得模糊的严重渐变/阴影
- `gridline-subtle`- 网格线应该是低对比度的（例如gray-200），这样它们就不会与数据竞争
- `focusable-elements`- 交互式图表元素（点、条形图、切片）必须可通过键盘导航 (WCAG)
- `screen-reader-summary`- 提供文本摘要或咏叹调标签，为屏幕阅读器 (WCAG) 描述图表的关键见解
- `error-state-chart`- 数据加载失败必须显示带有重试操作的错误消息，而不是损坏/空的图表
- `export-option`- 对于数据量大的产品，提供图表数据的 CSV/图像导出
- `drill-down-consistency`- 向下钻取交互必须保持清晰的反向路径和层次结构面包屑
- `time-scale-clarity`- 时间序列图表必须明确标注时间粒度（日/周/月）并允许切换

## 如何使用

使用下面的 CLI 工具搜索特定域。

-----

## 先决条件

检查Python是否已安装：

``` bash
python3 --version || python --version
```

如果未安装Python，请根据用户操作系统安装：

**苹果系统：**

``` bash
brew install python3
```

**Ubuntu/Debian:**

``` bash
sudo apt update && sudo apt install python3
```

**视窗：**

``` powershell
winget install Python.Python.3.12
```

-----

## 如何使用此技能

当用户请求以下任何一项时，请使用此技能：

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

请遵循以下工作流程：

### 步骤一：分析用户需求

从用户请求中提取关键信息：

- **产品类型：娱乐（社交、视频、音乐、游戏）、工具（扫描仪、编辑器、转换器）、生产力（任务管理器、笔记、日历）或混合**
- **目标受众：C端消费用户；考虑年龄组、使用环境（通勤、休闲、工作）**
- **风格关键词：俏皮、活力、极简、深色模式、内容优先、沉浸式等。**
- **堆栈：React Native（该项目唯一的技术堆栈）**

### 第 2 步：生成设计系统（必需）

**始终从以下方面开始获得全面的推理建议：`--design-system`**

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "<product_type> <industry> <keywords>" --design-system [-p "Project Name"]
```

这个命令：

1. 并行搜索域（产品、风格、颜色、着陆、版式）
2. 应用推理规则来选择最佳匹配`ui-reasoning.csv`
3. 返回完整的设计系统：图案、风格、颜色、排版、效果
4. 包括要避免的反模式

**例子：**

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "beauty spa wellness service" --design-system -p "Serenity Spa"
```

### 步骤 2b：坚持设计系统（主模式 + 覆盖模式）

要保存设计系统以便跨会话进行分层检索，请添加：****`--persist`

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "<query>" --design-system --persist -p "Project Name"
```

这将创建：

- `design-system/MASTER.md`— 具有所有设计规则的全球事实来源
- `design-system/pages/`— 页面特定覆盖的文件夹

**使用特定于页面的覆盖：**

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "<query>" --design-system --persist -p "Project Name" --page "dashboard"
```

这也创建了：

- `design-system/pages/dashboard.md`— 与主版的页面特定偏差

**分层检索的工作原理：**

1. 在构建特定页面（例如“结账”）时，首先检查`design-system/pages/checkout.md`
2. 如果页面文件存在，则其规则覆盖主文件****
3. 如果没有，请专门使用`design-system/MASTER.md`

**上下文感知检索提示：**

    I am building the [Page Name] page. Please read design-system/MASTER.md.
    Also check if design-system/pages/[page-name].md exists.
    If the page file exists, prioritize its rules.
    If not, use the Master rules exclusively.
    Now, generate the code...

### 步骤 3：补充详细搜索（根据需要）

获得设计系统后，使用域搜索来获取更多详细信息：

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "<keyword>" --domain <domain> [-n <max_results>]
```

**何时使用详细搜索：**

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

### 第 4 步：堆栈指南（React Native）

获取 React Native 实现特定的最佳实践：

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "<keyword>" --stack react-native
```

-----

## 搜索参考

### 可用域

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

### 可用堆栈

| Stack | Focus |
| --- | --- |
| `react-native` | Components, Navigation, Lists |

-----

## 示例工作流程

**用户请求：“制作一个人工智能搜索主页。”**

### 第 1 步：分析需求

- 产品类型：工具（AI搜索引擎）
- 目标受众：寻求快速、智能搜索的C端用户
- 风格关键词：现代、简约、内容优先、深色模式
- 堆栈：React Native

### 第 2 步：生成设计系统（必需）

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "AI search tool modern minimal" --design-system -p "AI Search"
```

**输出：完整的设计系统，包括图案、风格、颜色、排版、效果和反模式。**

### 步骤 3：补充详细搜索（根据需要）

``` bash
# Get style options for a modern tool product
python3 skills/ui-ux-pro-max/scripts/search.py "minimalism dark mode" --domain style

# Get UX best practices for search interaction and loading
python3 skills/ui-ux-pro-max/scripts/search.py "search loading animation" --domain ux
```

### 第 4 步：堆栈指南

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "list performance navigation" --stack react-native
```

**然后：综合设计体系+详细搜索并实现设计。**

-----

## 输出格式

该标志支持两种输出格式：`--design-system`

``` bash
# ASCII box (default) - best for terminal display
python3 skills/ui-ux-pro-max/scripts/search.py "fintech crypto" --design-system

# Markdown - best for documentation
python3 skills/ui-ux-pro-max/scripts/search.py "fintech crypto" --design-system -f markdown
```

-----

## 获得更好结果的技巧

### 查询策略

- 使用多维度关键词——结合产品+行业+基调+密度：不仅仅是****`"entertainment social vibrant content-dense"``"app"`
- 为相同的需求尝试不同的关键字：→ →`"playful neon"``"vibrant dark"``"content-first minimal"`
- 首先使用完整的建议，然后深入研究您不确定的任何维度`--design-system``--domain`
- 始终添加特定于实施的指导`--stack react-native`

### 常见的症结点

| Problem | What to Do |
| --- | --- |
| Can't decide on style/color | Re-run `--design-system` with different keywords |
| Dark mode contrast issues | Quick Reference §6: `color-dark-mode` + `color-accessible-pairs` |
| Animations feel unnatural | Quick Reference §7: `spring-physics` + `easing` + `exit-faster-than-enter` |
| Form UX is poor | Quick Reference §8: `inline-validation` + `error-clarity` + `focus-management` |
| Navigation feels confusing | Quick Reference §9: `nav-hierarchy` + `bottom-nav-limit` + `back-behavior` |
| Layout breaks on small screens | Quick Reference §5: `mobile-first` + `breakpoint-consistency` |
| Performance / jank | Quick Reference §3: `virtualize-lists` + `main-thread-budget` + `debounce-throttle` |

### 交货前检查清单

- 在实施之前作为 UX 验证通过运行`--domain ux "animation accessibility z-index loading"`
- 完成快速参考§1–§3（关键+高）作为最终审查****
- 在 375px（小手机）和横向上进行测试
- 验证启用减少运动和最大尺寸的动态类型的行为********
- 独立检查暗模式对比度（不要假设亮模式值有效）
- 确认所有触摸目标≥44pt并且没有内容隐藏在安全区域后面

-----

## 专业 UI 的通用规则

这些是经常被忽视的问题，使 UI 看起来不专业： 范围通知：以下规则适用于 App UI（iOS/Android/React Native/Flutter），而不是桌面 Web 交互模式。

### 图标和视觉元素

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

### 互动（应用程序）

| Rule | Do | Don't |
| --- | --- | --- |
| **Tap feedback** | Provide clear pressed feedback (ripple/opacity/elevation) within 80-150ms | No visual response on tap |
| **Animation timing** | Keep micro-interactions around 150-300ms with platform-native easing | Instant transitions or slow animations (\>500ms) |
| **Accessibility focus** | Ensure screen reader focus order matches visual order and labels are descriptive | Unlabeled controls or confusing focus traversal |
| **Disabled state clarity** | Use disabled semantics (`disabled`/native disabled props), reduced emphasis, and no tap action | Controls that look tappable but do nothing |
| **Touch target minimum** | Keep tap areas \>=44x44pt (iOS) or \>=48x48dp (Android), expand hit area when icon is smaller | Tiny tap targets or icon-only hit areas without padding |
| **Gesture conflict prevention** | Keep one primary gesture per region and avoid nested tap/drag conflicts | Overlapping gestures causing accidental actions |
| **Semantic native controls** | Prefer native interactive primitives (`Button`, `Pressable`, platform equivalents) with proper accessibility roles | Generic containers used as primary controls without semantics |

### 明/暗模式对比

| Rule | Do | Don't |
| --- | --- | --- |
| **Surface readability (light)** | Keep cards/surfaces clearly separated from background with sufficient opacity/elevation | Overly transparent surfaces that blur hierarchy |
| **Text contrast (light)** | Maintain body text contrast \>=4.5:1 against light surfaces | Low-contrast gray body text |
| **Text contrast (dark)** | Maintain primary text contrast \>=4.5:1 and secondary text \>=3:1 on dark surfaces | Dark mode text that blends into background |
| **Border and divider visibility** | Ensure separators are visible in both themes (not just light mode) | Theme-specific borders disappearing in one mode |
| **State contrast parity** | Keep pressed/focused/disabled states equally distinguishable in light and dark themes | Defining interaction states for one theme only |
| **Token-driven theming** | Use semantic color tokens mapped per theme across app surfaces/text/icons | Hardcoded per-screen hex values |
| **Scrim and modal legibility** | Use a modal scrim strong enough to isolate foreground content (typically 40-60% black) | Weak scrim that leaves background visually competing |

### 布局和间距

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

## 交货前检查清单

在交付 UI 代码之前，请验证以下项目： 范围通知：此清单适用于应用程序 UI (iOS/Android/React Native/Flutter)。

### 视觉质量

- [ ] 不使用表情符号作为图标（使用 SVG 代替）
- [ ] 所有图标都来自一致的图标系列和风格
- [ ] 官方品牌资产使用比例正确、空间清晰
- [ ] 按下状态的视觉效果不会改变布局边界或导致抖动
- [ ] 一致地使用语义主题标记（没有特定的每屏幕硬编码颜色）

### 相互作用

- [ ] 所有可点击的元素都提供清晰的按压反馈（波纹/不透明度/高度）
- [ ] 触摸目标满足最小尺寸（\>=44x44pt iOS，\>=48x48dp Android）
- [ ] 微交互时序保持在 150-300ms 范围内，具有原生的缓和感
- [ ] 禁用状态视觉上清晰且非交互
- [ ] 屏幕阅读器焦点顺序与视觉顺序相匹配，交互式标签具有描述性
- [ ] 手势区域避免嵌套/冲突的交互（点击/拖动/向后滑动冲突）

### 明/暗模式

- [ ] 浅色和深色模式下主要文本对比度 \>=4.5:1
- [ ] 浅色和深色模式下辅助文本对比度 \>=3:1
- [ ] 分隔线/边界和交互状态在两种模式下都是可区分的
- [ ] 莫代尔/抽屉稀松布的不透明度足以保持前景的清晰度（通常为 40-60% 黑色）
- [ ] 两个主题在交付前都经过测试（不是从单个主题推断）

### 布局

- [ ] 标题、选项卡栏和底部 CTA 栏均遵循安全区域
- [ ] 滚动内容不会隐藏在固定/粘性栏后面
- [ ] 在小手机、大手机和平板电脑上经过验证（纵向+横向）
- [ ] 水平插图/装订线根据设备尺寸和方向正确调整
- [ ] 跨组件、部分和页面级别保持 4/8dp 间距节奏
- [ ] 长文本度量在较大的设备上仍然可读（无边到边的段落）

### 无障碍

- [ ] 所有有意义的图像/图标都有辅助功能标签
- [ ] 表单字段有标签、提示和清晰的错误消息
- [ ] 颜色不是唯一指标
- [ ] 支持减少运动和动态文本大小，而不会破坏布局
- [ ] 正确宣布辅助功能特征/角色/状态（选择、禁用、扩展）
