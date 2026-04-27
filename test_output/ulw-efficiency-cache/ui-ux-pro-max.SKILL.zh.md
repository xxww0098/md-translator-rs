---
name: ui-ux-pro-max
description: "UI/UX design intelligence for web and mobile. Includes 50+ styles, 161 color palettes, 57 font pairings, 161 product types, 99 UX guidelines, and 25 chart types across 10 stacks (React, Next.js, Vue, Svelte, SwiftUI, React Native, Flutter, Tailwind, shadcn/ui, and HTML/CSS). Actions: plan, build, create, design, implement, review, fix, improve, optimize, enhance, refactor, and check UI/UX code. Projects: website, landing page, dashboard, admin panel, e-commerce, SaaS, portfolio, blog, and mobile app. Elements: button, modal, navbar, sidebar, card, table, form, and chart. Styles: glassmorphism, claymorphism, minimalism, brutalism, neumorphism, bento grid, dark mode, responsive, skeuomorphism, and flat design. Topics: color systems, accessibility, animation, layout, typography, font pairing, spacing, interaction states, shadow, and gradient. Integrations: shadcn/ui MCP for component search and examples."
---

# UI/UX Pro Max - 设计智能

面向网页和移动应用的综合设计指南。包含50+种风格、161套配色方案、57组字体搭配、161种带推理规则的产品类型、99条UX指南，以及覆盖10种技术栈的25种图表类型。可搜索数据库，支持基于优先级的推荐。

## 何时应用

当任务涉及 UI 结构、视觉设计决策、交互模式或用户体验质量控制时，应使用此技能。****

### 必须使用

在以下情况下必须调用此技能：

- 设计新页面（落地页、仪表盘、管理后台、SaaS、移动应用）
- 创建或重构 UI 组件（按钮、弹窗、表单、表格、图表等）
- 选择配色方案、排版系统、间距标准或布局系统
- 审查 UI 代码中的用户体验、可访问性或视觉一致性
- 实现导航结构、动画或响应式行为
- 制定产品层面的设计决策（风格、信息层级、品牌表达）
- 提升界面的感知质量、清晰度或可用性

### 推荐

在以下情况下推荐使用此技能：

- UI 看起来“不够专业”，但原因不明确
- 正在收集可用性或体验反馈
- 上线前的 UI 质量优化
- 统一跨平台设计（Web / iOS / Android）
- 构建设计系统或可复用组件库

### 跳过

在以下情况下不需要此技能：

- 纯后端逻辑开发
- 仅涉及 API 或数据库设计
- 与界面无关的性能优化
- 基础设施或 DevOps 工作
- 非可视化脚本或自动化任务

**决策标准：如果该任务会改变某个功能的外观、感受、运动方式或交互方式，则应使用此技能。******

## 按优先级划分的规则类别

*供人工/AI 参考：按照优先级 1→10 决定首先关注哪个规则类别；需要时使用  查询详情。脚本不会读取此表。`--domain <Domain>`*

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

### 1\. 无障碍性（关键）

- `color-contrast`- 正常文本最小对比度 4.5:1（大文本 3:1）；Material Design
- `focus-states`- 交互元素上应显示可见焦点环（2–4px；Apple HIG，MD）
- `alt-text`- 为有意义的图片提供描述性替代文本
- `aria-labels`- 图标按钮的 aria-label；原生中的 accessibilityLabel（Apple HIG）
- `keyboard-nav`- Tab 顺序与视觉顺序一致；完全支持键盘操作（Apple HIG）
- `form-labels`- 使用带 for 属性的 label
- `skip-links`- 为键盘用户提供跳到主要内容
- `heading-hierarchy`- 按 h1→h6 顺序，不要跳级
- `color-not-only`- 不要仅靠颜色传达信息（添加图标/文本）
- `dynamic-type`- 支持系统文本缩放；随着文本增大避免截断（Apple Dynamic Type，MD）
- `reduced-motion`- 尊重 prefers-reduced-motion；在需要时减少/禁用动画（Apple Reduced Motion API，MD）
- `voiceover-sr`- 提供有意义的 accessibilityLabel/accessibilityHint；为 VoiceOver/屏幕阅读器保持逻辑阅读顺序（Apple HIG，MD）
- `escape-routes`- 在模态窗口和多步骤流程中提供取消/返回（Apple HIG）
- `keyboard-shortcuts`- 保留系统和辅助功能快捷键；为拖放操作提供键盘替代方案（Apple HIG）

### 2\. 触控 & 交互（关键）

- `touch-target-size`- 最小 44×44pt（Apple）/ 48×48dp（Material）；如有需要，将可点击区域扩展到视觉边界之外
- `touch-spacing`- 触控目标之间最小间距为 8px/8dp（Apple HIG，MD）
- `hover-vs-tap`- 主要交互使用点击/轻触；不要仅依赖悬停
- `loading-buttons`- 在异步操作期间禁用按钮；显示加载指示器或进度
- `error-feedback`- 在问题附近显示清晰的错误消息
- `cursor-pointer`- 为可点击元素添加 cursor-pointer（Web）
- `gesture-conflicts`- 避免在主内容上使用水平滑动；优先使用垂直滚动
- `tap-delay`- 使用 touch-action: manipulation 以减少 300ms 延迟（Web）
- `standard-gestures`- 始终一致地使用平台标准手势；不要重新定义（例如：返回滑动、捏合缩放）（Apple HIG）
- `system-gestures`- 不要阻止系统手势（控制中心、返回滑动等）（Apple HIG）
- `press-feedback`- 按下时提供视觉反馈（涟漪/高亮；MD 状态层）
- `haptic-feedback`- 在确认和重要操作时使用触觉反馈；避免过度使用（Apple HIG）
- `gesture-alternative`- 不要只依赖手势交互；对关键操作始终提供可见控件
- `safe-area-awareness`- 将主要触控目标避开刘海、灵动岛、手势条和屏幕边缘
- `no-precision-required`- 避免要求在小图标或细边缘上进行像素级精确点击
- `swipe-clarity`- 滑动操作必须显示明确的可操作提示或引导（箭头、标签、教程）
- `drag-threshold`- 在开始拖动前使用位移阈值，以避免误触拖动

### 3\. 性能（高）

- `image-optimization`- 使用 WebP/AVIF、响应式图片（srcset/sizes），并对非关键资源进行懒加载
- `image-dimension`- 声明宽度/高度或使用 aspect-ratio 以防止布局偏移（Core Web Vitals：CLS）
- `font-loading`- 使用 font-display: swap/optional 以避免不可见文本（FOIT）；预留空间以减少布局偏移（MD）
- `font-preload`- 仅预加载关键字体；避免对每个变体都过度使用 preload
- `critical-css`- 优先加载首屏 CSS（内联关键 CSS 或尽早加载样式表）
- `lazy-loading`- 通过动态导入 / 路由级拆分延迟加载非首屏组件
- `bundle-splitting`- 按路由/功能拆分代码（React Suspense / Next.js dynamic），以减少初始加载和 TTI
- `third-party-scripts`- 以 async/defer 方式加载第三方脚本；审计并移除不必要的脚本（MD）
- `reduce-reflows`- 避免频繁进行布局读/写；先批量读取 DOM，再批量写入
- `content-jumping`- 为异步内容预留空间，避免布局跳动（Core Web Vitals: CLS）
- `lazy-load-below-fold`- 对首屏下方的图片和大型媒体使用 loading="lazy"
- `virtualize-lists`- 对包含 50 个以上项目的列表进行虚拟化，以提高内存效率和滚动性能
- `main-thread-budget`- 将每帧工作控制在约 16ms 以内以实现 60fps；将重任务移出主线程（HIG，MD）
- `progressive-loading`- 对于超过 1 秒的操作，使用骨架屏 / 闪动占位，而不是长时间阻塞的加载转圈（Apple HIG）
- `input-latency`- 将点击/滚动的输入延迟控制在约 100ms 以内（Material 响应性标准）
- `tap-feedback-speed`- 在点击后 100ms 内提供视觉反馈（Apple HIG）
- `debounce-throttle`- 对高频事件（滚动、调整大小、输入）使用防抖/节流
- `offline-support`- 提供离线状态提示和基本降级方案（PWA / mobile）
- `network-fallback`- 为慢速网络提供降级模式（低分辨率图片、更少动画）

### 4\. 风格选择（高）

- `style-match`- 使风格与产品类型匹配（将  用于推荐）`--design-system`
- `consistency`- 整个页面使用相同的风格
- `no-emoji-icons`- 使用 SVG 图标（Heroicons、Lucide），不要使用表情符号
- `color-palette-from-product`- 从产品/行业中选择配色方案（搜索 ）`--domain color`
- `effects-match-style`- 阴影、模糊和圆角半径与所选风格保持一致（玻璃 / 扁平 / 黏土等）
- `platform-adaptive`- 尊重平台惯用规范（iOS HIG 与 Material）：导航、控件、排版、动效
- `state-clarity`- 使悬停/按下/禁用状态在视觉上可区分，同时保持风格一致（Material 状态层）
- `elevation-consistent`- 为卡片、面板、模态框使用一致的层级/阴影尺度；避免随机阴影值
- `dark-mode-pairing`- 将浅色/深色变体一并设计，以保持品牌、对比度和风格的一致性
- `icon-style-consistent`- 在整个产品中使用统一的图标集/视觉语言（描边粗细、圆角半径）
- `system-controls`- 优先使用原生/系统控件，而非完全自定义控件；仅在品牌要求时进行定制（Apple HIG）
- `blur-purpose`- 使用模糊效果表示背景被关闭（模态框、抽屉），而不是作为装饰（Apple HIG）
- `primary-action`- 每个屏幕应只有一个主要 CTA；次要操作在视觉上应从属显示（Apple HIG）

### 5\. 布局与响应式（高）

- `viewport-meta`- width=device-width initial-scale=1（不要禁用缩放）
- `mobile-first`- 先以移动端优先设计，再扩展到平板和桌面端
- `breakpoint-consistency`- 使用系统化断点（例如 375 / 768 / 1024 / 1440）
- `readable-font-size`- 移动端正文最小 16px（避免 iOS 自动缩放）
- `line-length-control`- 移动端每行 35–60 个字符；桌面端 60–75 个字符
- `horizontal-scroll`- 移动端不允许水平滚动；确保内容适应视口宽度
- `spacing-scale`- 使用 4pt/8dp 递增间距系统（Material Design）
- `touch-density`- 保持组件间距对触控友好：不拥挤，也不会导致误触
- `container-width`- 桌面端一致的最大宽度（max-w-6xl / 7xl）
- `z-index-management`- 定义分层的 z-index 层级（例如 0 / 10 / 20 / 40 / 100 / 1000）
- `fixed-element-offset`- 固定顶部导航栏/底部栏必须为其下方内容预留安全内边距
- `scroll-behavior`- 避免嵌套滚动区域干扰主滚动体验
- `viewport-units`- 在移动端优先使用 min-h-dvh 而不是 100vh
- `orientation-support`- 在横屏模式下保持布局可读且可操作
- `content-priority`- 在移动端优先展示核心内容；将次要内容折叠或隐藏
- `visual-hierarchy`- 通过大小、间距、对比度来建立层级——不要只依赖颜色

### 6\. 排版与颜色（中等）

- `line-height`- 正文使用 1.5-1.75 的行高
- `line-length`- 每行限制 65-75 个字符
- `font-pairing`- 标题/正文字体风格一致
- `font-scale`- 统一的字号比例（例如 12 14 16 18 24 32）
- `contrast-readability`- 在浅色背景上使用更深的文本（例如白色背景上的 slate-900）
- `text-styles-system`- 使用平台类型系统：iOS 11 Dynamic Type 样式 / Material 5 字体角色（display、headline、title、body、label）（HIG，MD）
- `weight-hierarchy`- 使用 font-weight 强化层级：标题加粗（600–700）、正文常规（400）、标签中等（500）（MD）
- `color-semantic`- 在组件中定义语义化颜色令牌（primary、secondary、error、surface、on-surface），不要直接使用原始十六进制值（Material 颜色系统）
- `color-dark-mode`- 深色模式使用低饱和度/更浅的色调变体，而不是反相颜色；需单独测试对比度（HIG，MD）
- `color-accessible-pairs`- 前景/背景配对必须达到 4.5:1（AA）或 7:1（AAA）；请使用工具验证（WCAG，MD）
- `color-not-decorative-only`- 功能性色彩（错误红、成功绿）必须配合图标/文字使用；避免仅靠颜色传达含义（HIG，MD）
- `truncation-strategy`- 优先换行而不是截断；如需截断，使用省略号，并通过工具提示/展开提供完整文本（Apple HIG）
- `letter-spacing`- 尊重各平台的默认字间距；避免正文使用过紧的字距（HIG, MD）
- `number-tabular`- 对数据列、价格和计时器使用表格数字/等宽数字，以防止布局偏移
- `whitespace-balance`- 有意识地使用空白来分组相关项目并分隔各部分；避免视觉杂乱（Apple HIG）

### 7\. 动画（中等）

- `duration-timing`- 微交互使用 150–300ms；复杂过渡 ≤400ms；避免 \>500ms（MD）
- `transform-performance`- 仅使用 transform/opacity；避免对 width/height/top/left 做动画
- `loading-states`- 当加载时间超过 300ms 时，显示骨架屏或进度指示器
- `excessive-motion`- 每个视图最多只为 1-2 个关键元素添加动画
- `easing`- 进入时使用 ease-out，退出时使用 ease-in；避免 UI 过渡使用 linear
- `motion-meaning`- 每个动画都必须表达因果关系，而不仅仅是装饰效果（Apple HIG）
- `state-transition`- 状态变化（悬停 / 激活 / 展开 / 折叠 / 模态）应平滑过渡，而不是突然跳变
- `continuity`- 页面/屏幕切换应保持空间连续性（共享元素、方向性滑动）（Apple HIG）
- `parallax-subtle`- 谨慎使用视差；必须尊重减少动态效果设置，且不得造成眩晕（Apple HIG）
- `spring-physics`- 优先使用基于弹簧/物理的曲线，而不是线性或 cubic-bezier，以获得更自然的感觉（Apple HIG 流畅动画）
- `exit-faster-than-enter`- 退出动画应比进入动画更短（约为进入时长的 60–70%），以显得更灵敏（MD 动效）
- `stagger-sequence`- 列表/网格项的进入应逐个错开 30–50 毫秒；避免一次全部出现或显示过慢（MD）
- `shared-element-transition`- 使用共享元素 / hero 过渡来保持不同屏幕之间的视觉连续性（MD，HIG）
- `interruptible`- 动画必须可中断；用户点击/手势应立即取消正在进行的动画（Apple HIG）
- `no-blocking-animation`- 动画过程中绝不能阻塞用户输入；界面必须保持可交互（Apple HIG）
- `fade-crossfade`- 在同一容器内替换内容时使用交叉淡化（MD）
- `scale-feedback`- 轻微缩放（0.95–1.05）用于按下可轻触的卡片/按钮时；松开时恢复（HIG，MD）
- `gesture-feedback`- 拖拽、滑动和捏合必须提供实时视觉反馈，跟随手指移动（MD Motion）
- `hierarchy-motion`- 使用 translate/scale 方向来表达层级：从下方进入 = 更深层，向上退出 = 返回（MD）
- `motion-consistency`- 全局统一 duration/easing 令牌；所有动画共享相同的节奏和感觉
- `opacity-threshold`- 淡出元素不应在低于 0.2 的不透明度下停留；要么完全淡出，要么保持可见
- `modal-motion`- 模态框/抽屉应从其触发源进行动画（缩放+淡入或滑入），以提供空间上下文（HIG，MD）
- `navigation-direction`- 前进导航向左/向上动画；后退导航向右/向下动画——保持方向逻辑一致（HIG）
- `layout-shift-avoid`- 动画不得引发布局重排或 CLS；位置变化应使用 transform

### 8\. 表单与反馈（中等）

- `input-labels`- 每个输入项都应有可见标签（而非仅使用占位符）
- `error-placement`- 在相关字段下方显示错误
- `submit-feedback`- 提交时先显示加载状态，然后显示成功/错误状态
- `required-indicators`- 标记必填字段（例如星号）
- `empty-states`- 无内容时提供有帮助的提示和操作
- `toast-dismiss`- 3-5秒后自动消失的提示
- `confirmation-dialogs`- 在执行破坏性操作前确认
- `input-helper-text`- 在复杂输入框下方提供持久的辅助文本，而不仅仅是占位符（Material Design）
- `disabled-states`- 禁用元素使用较低的不透明度（0.38–0.5）+ 光标变化 + 语义属性（MD）
- `progressive-disclosure`- 逐步展示复杂选项；不要一开始就让用户不知所措（Apple HIG）
- `inline-validation`- 在失焦时验证（而不是在每次按键时）；仅在用户完成输入后显示错误（MD）
- `input-type-keyboard`- 使用语义输入类型（email、tel、number）来调出正确的移动键盘（HIG，MD）
- `password-toggle`- 为密码字段提供显示/隐藏切换（MD）
- `autofill-support`- 使用 autocomplete / textContentType 属性，以便系统可以自动填充（HIG，MD）
- `undo-support`- 允许对破坏性或批量操作进行撤销（例如“撤销删除”提示条）（Apple HIG）
- `success-feedback`- 使用简短的视觉反馈确认已完成的操作（对勾、提示条、颜色闪烁）（MD）
- `error-recovery`- 错误消息必须包含清晰的恢复路径（重试、编辑、帮助链接）（HIG, MD）
- `multi-step-progress`- 多步骤流程显示步骤指示器或进度条；允许返回导航（MD）
- `form-autosave`- 长表单应自动保存草稿，以防因误关闭而丢失数据（Apple HIG）
- `sheet-dismiss-confirm`- 在关闭包含未保存更改的 sheet/modal 之前先确认（Apple HIG）
- `error-clarity`- 错误消息必须说明原因 + 如何修复（而不只是“输入无效”）（HIG, MD）
- `field-grouping`- 逻辑上将相关字段分组（fieldset/legend 或视觉分组）（MD）
- `read-only-distinction`- 只读状态在视觉和语义上应与禁用状态不同（MD）
- `focus-management`- 提交出错后，自动聚焦第一个无效字段（WCAG，MD）
- `error-summary`- 对于多个错误，在顶部显示摘要，并为每个字段提供锚点链接（WCAG）
- `touch-friendly-input`- 移动端输入框高度应≥44px，以满足触控目标要求（Apple HIG）
- `destructive-emphasis`- 破坏性操作使用语义化危险颜色（红色），并在视觉上与主要操作分隔（HIG, MD）
- `toast-accessibility`- Toast 不得抢占焦点；使用 aria-live="polite" 让屏幕阅读器播报（WCAG）
- `aria-live-errors`- 表单错误使用 aria-live 区域或 role="alert" 通知屏幕阅读器（WCAG）
- `contrast-feedback`- 错误和成功状态颜色必须满足 4.5:1 对比度（WCAG, MD）
- `timeout-feedback`- 请求超时必须显示清晰反馈并提供重试选项（MD）

### 9\. 导航模式（高）

- `bottom-nav-limit`- 底部导航最多 5 个项目；使用带图标的标签（Material Design）
- `drawer-usage`- 使用抽屉/侧边栏进行次级导航，不用于主要操作（Material Design）
- `back-behavior`- 返回导航必须可预测且一致；保留滚动位置/状态（Apple HIG, MD）
- `deep-linking`- 所有关键页面都必须可通过深度链接 / URL 访问，以便分享和通知（Apple HIG, MD）
- `tab-bar-ios`- iOS：使用底部标签栏作为顶层导航（Apple HIG）
- `top-app-bar-android`- Android：使用带导航图标的顶部应用栏作为主要结构（Material Design）
- `nav-label-icon`- 导航项必须同时具备图标和文本标签；仅图标导航会损害可发现性（MD）
- `nav-state-active`- 在导航中，当前位置必须在视觉上突出显示（颜色、字重、指示器）（HIG, MD）
- `nav-hierarchy`- 主导航（标签页/底部栏）与次导航（抽屉/设置）必须清晰区分（MD）
- `modal-escape`- 模态框和底部弹出页必须提供清晰的关闭/关闭提示；在移动端可通过向下滑动关闭（Apple HIG）
- `search-accessible`- 搜索必须易于访问（顶部栏或标签页）；提供最近/建议的查询（MD）
- `breadcrumb-web`- Web：对于 3 层及以上的深层层级结构，使用面包屑导航以帮助定位（MD）
- `state-preservation`- 返回时必须恢复之前的滚动位置、筛选状态和输入内容（HIG，MD）
- `gesture-nav-support`- 支持系统手势导航（iOS 侧滑返回、Android 预测性返回），且不与之冲突（HIG，MD）
- `tab-badge`- 谨慎在导航项上使用徽标，以指示未读/待处理；用户访问后清除（HIG，MD）
- `overflow-menu`- 当操作数量超过可用空间时，使用溢出/更多菜单，而不是硬塞进去（MD）
- `bottom-nav-top-level`- 底部导航仅用于顶级页面；切勿在其中嵌套子导航（MD）
- `adaptive-navigation`- 大屏幕（≥1024px）优先使用侧边栏；小屏幕使用底部/顶部导航（Material Adaptive）
- `back-stack-integrity`- 绝不要悄悄重置导航栈或意外跳转到主页（HIG，MD）
- `navigation-consistency`- 导航位置在所有页面中必须保持一致；不要按页面类型更改
- `avoid-mixed-patterns`- 不要在同一层级混用标签页 + 侧边栏 + 底部导航
- `modal-vs-navigation`- 不应将模态框用于主要导航流程；它们会打断用户的路径（HIG）
- `focus-on-route-change`- 页面切换后，将焦点移动到主内容区域，供屏幕阅读器用户使用（WCAG）
- `persistent-nav`- 核心导航必须在深层页面中仍可访问；不要在子流程中将其完全隐藏（HIG，MD）
- `destructive-nav-separation`- 危险操作（删除账户、登出）必须在视觉上和空间上与普通导航项分开（HIG，MD）
- `empty-nav-state`- 当导航目标不可用时，应说明原因，而不是静默隐藏它（MD）

### 10\. 图表与数据（低）

- `chart-type`- 将图表类型与数据类型相匹配（趋势 → 折线图，比较 → 柱状图，占比 → 饼图/圆环图）
- `color-guidance`- 使用无障碍配色方案；避免仅使用红/绿组合，以照顾色盲用户（WCAG，MD）
- `data-table`- 为无障碍性提供表格替代方案；单靠图表并不适合屏幕阅读器（WCAG）
- `pattern-texture`- 用图案、纹理或形状补充颜色，以便数据即使不依赖颜色也能区分（WCAG，MD）
- `legend-visible`- 始终显示图例；将其放在靠近图表的位置，不要把它放在下方的滚动折叠区域之外（MD）
- `tooltip-on-interact`- 在悬停时（Web）或轻点时（移动端）提供工具提示/数据标签，显示精确数值（HIG，MD）
- `axis-labels`- 为坐标轴标注单位和清晰可读的刻度；在移动端避免标签被截断或旋转
- `responsive-chart`- 图表在小屏幕上必须重新排版或简化（例如用横向条形图代替纵向图，减少刻度）
- `empty-data-state`- 当没有数据时，显示有意义的空状态（“暂无数据” + 引导说明），而不是空白图表（MD）
- `loading-chart`- 图表数据加载时使用骨架屏或闪烁占位；不要显示空的坐标轴框架
- `animation-optional`- 图表入场动画必须尊重“减少动态效果”偏好；数据应立即可读（HIG）
- `large-dataset`- 对于 1000+ 个数据点，应进行聚合或抽样；应提供下钻查看详情，而不是全部渲染（MD）
- `number-formatting`- 坐标轴和标签中的数字、日期、货币应使用符合本地化的格式（HIG，MD）
- `touch-target-chart`- 交互式图表元素（点、线段）必须具有≥44pt的点击区域，或在触摸时扩展（Apple HIG）
- `no-pie-overuse`- 当类别数\>5时，避免使用饼图/圆环图；改用柱状图以提高清晰度
- `contrast-data`- 数据线/柱状图与背景的对比度≥3:1；数据文本标签≥4.5:1（WCAG）
- `legend-interactive`- 图例应可点击，以切换系列可见性（MD）
- `direct-labeling`- 对于小型数据集，直接在图表上标注数值，以减少视线移动
- `tooltip-keyboard`- 提示内容必须可通过键盘访问，且不能只依赖悬停（WCAG）
- `sortable-table`- 数据表必须支持排序，并使用 aria-sort 指示当前排序状态（WCAG）
- `axis-readability`- 轴刻度不应过于拥挤；保持可读的间距，在小屏幕上自动跳过
- `data-density`- 限制每张图表的信息密度，以避免认知过载；必要时拆分为多张图表
- `trend-emphasis`- 强调数据趋势而非装饰；避免使用会遮挡数据的浓重渐变/阴影
- `gridline-subtle`- 网格线应使用低对比度（例如 gray-200），以免与数据抢眼
- `focusable-elements`- 交互式图表元素（点、条形、扇区）必须可通过键盘导航（WCAG）
- `screen-reader-summary`- 为屏幕阅读器提供文本摘要或 aria-label，描述图表的关键见解（WCAG）
- `error-state-chart`- 数据加载失败时必须显示带重试操作的错误消息，而不是损坏/空白的图表
- `export-option`- 对于数据量大的产品，提供图表数据的 CSV/图片导出
- `drill-down-consistency`- 下钻交互必须保持清晰的返回路径和层级面包屑
- `time-scale-clarity`- 时间序列图表必须清晰标注时间粒度（天/周/月）并允许切换

## 如何使用

使用下面的 CLI 工具搜索特定域名。

-----

## 前提条件

检查是否已安装 Python：

``` bash
python3 --version || python --version
```

如果未安装 Python，请根据用户的操作系统进行安装：

**macOS：**

``` bash
brew install python3
```

**Ubuntu/Debian:**

``` bash
sudo apt update && sudo apt install python3
```

**Windows：**

``` powershell
winget install Python.Python.3.12
```

-----

## 如何使用此技能

当用户请求以下任一内容时使用此技能：

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

遵循以下工作流程：

### 步骤 1：分析用户需求

从用户请求中提取关键信息：

- **产品类型：娱乐（社交、视频、音乐、游戏）、工具（扫描、编辑、转换）、生产力（任务管理、笔记、日历）或混合型**
- **目标受众：C端消费者用户；考虑年龄段、使用场景（通勤、休闲、工作）**
- **风格关键词：俏皮、鲜艳、极简、深色模式、内容优先、沉浸式等。**
- **技术栈：React Native（本项目唯一技术栈）**

### 步骤 2：生成设计系统（必需）

**始终先使用  获取带有推理的全面建议：`--design-system`**

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "<product_type> <industry> <keywords>" --design-system [-p "Project Name"]
```

该命令：

1. 并行搜索多个领域（产品、风格、颜色、落地页、排版）
2. 应用来自  的推理规则以选择最佳匹配项`ui-reasoning.csv`
3. 返回完整设计系统：模式、样式、颜色、排版、效果
4. 包含应避免的反模式

**示例：**

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "beauty spa wellness service" --design-system -p "Serenity Spa"
```

### 步骤 2b：持久化设计系统（主方案 + 覆盖模式）

为了在会话间实现分层检索并保存设计系统，请添加：****`--persist`

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "<query>" --design-system --persist -p "Project Name"
```

这会创建：

- `design-system/MASTER.md`— 包含所有设计规则的全局事实来源
- `design-system/pages/`— 用于页面特定覆盖的文件夹

**对于页面特定覆盖：**

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "<query>" --design-system --persist -p "Project Name" --page "dashboard"
```

这也会创建：

- `design-system/pages/dashboard.md`— 相对于主文件的页面特定偏差

**分层检索的工作方式：**

1. 在构建特定页面（例如 "Checkout"）时，首先检查`design-system/pages/checkout.md`
2. 如果页面文件存在，其规则将覆盖主文件****
3. 如果不是，则仅使用`design-system/MASTER.md`

**上下文感知检索提示：**

    I am building the [Page Name] page. Please read design-system/MASTER.md.
    Also check if design-system/pages/[page-name].md exists.
    If the page file exists, prioritize its rules.
    If not, use the Master rules exclusively.
    Now, generate the code...

### 步骤3：根据需要补充详细搜索

获取设计系统后，使用领域搜索获取更多细节：

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

### 第4步：技术栈指南（React Native）

获取 React Native 实现相关的最佳实践：

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

### 可用技术栈

| Stack | Focus |
| --- | --- |
| `react-native` | Components, Navigation, Lists |

-----

## 示例工作流

**用户请求：“制作一个 AI 搜索首页。”**

### 第1步：分析需求

- 产品类型：工具（AI 搜索引擎）
- 目标受众：寻求快速、智能搜索的 C 端用户
- 风格关键词：现代、简约、内容优先、深色模式
- 栈：React Native

### 第 2 步：生成设计系统（必需）

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "AI search tool modern minimal" --design-system -p "AI Search"
```

**输出：完整的设计系统，包括模式、样式、颜色、排版、效果和反模式。**

### 第 3 步：根据需要补充详细搜索

``` bash
# Get style options for a modern tool product
python3 skills/ui-ux-pro-max/scripts/search.py "minimalism dark mode" --domain style

# Get UX best practices for search interaction and loading
python3 skills/ui-ux-pro-max/scripts/search.py "search loading animation" --domain ux
```

### 第4步：技术栈指南

``` bash
python3 skills/ui-ux-pro-max/scripts/search.py "list performance navigation" --stack react-native
```

**然后：综合设计系统 + 详细搜索并实施设计。**

-----

## 输出格式

该  标志支持两种输出格式：`--design-system`

``` bash
# ASCII box (default) - best for terminal display
python3 skills/ui-ux-pro-max/scripts/search.py "fintech crypto" --design-system

# Markdown - best for documentation
python3 skills/ui-ux-pro-max/scripts/search.py "fintech crypto" --design-system -f markdown
```

-----

## 获得更佳结果的提示

### 查询策略

- 使用多维关键词——结合产品 + 行业 + 语气 + 密度：而不只是****`"entertainment social vibrant content-dense"``"app"`
- 为同一需求尝试不同关键词：  →  →`"playful neon"``"vibrant dark"``"content-first minimal"`
- 先使用  获取完整建议，然后用  深入查看你不确定的任何维度`--design-system``--domain`
- 始终添加  以获取针对具体实现的指导`--stack react-native`

### 常见卡点

| Problem | What to Do |
| --- | --- |
| Can't decide on style/color | Re-run `--design-system` with different keywords |
| Dark mode contrast issues | Quick Reference §6: `color-dark-mode` + `color-accessible-pairs` |
| Animations feel unnatural | Quick Reference §7: `spring-physics` + `easing` + `exit-faster-than-enter` |
| Form UX is poor | Quick Reference §8: `inline-validation` + `error-clarity` + `focus-management` |
| Navigation feels confusing | Quick Reference §9: `nav-hierarchy` + `bottom-nav-limit` + `back-behavior` |
| Layout breaks on small screens | Quick Reference §5: `mobile-first` + `breakpoint-consistency` |
| Performance / jank | Quick Reference §3: `virtualize-lists` + `main-thread-budget` + `debounce-throttle` |

### 交付前检查清单

- 在实施前作为 UX 验证步骤执行`--domain ux "animation accessibility z-index loading"`
- 在最终审查时通读快速参考§1–§3（严重 + 高）****
- 在 375px（小屏手机）和横屏方向下测试
- 在启用减少动态效果和最大字号的动态字体时验证行为********
- 单独检查深色模式对比度（不要假设浅色模式的数值也适用）
- 确认所有触控目标 ≥44pt，且没有内容被安全区域遮挡

-----

## 专业 UI 的通用规则

这些是经常被忽视、会让 UI 显得不专业的问题：适用范围说明：以下规则适用于 App UI（iOS/Android/React Native/Flutter），不适用于桌面网页交互模式。

### 图标与视觉元素

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

### 交互（App）

| Rule | Do | Don't |
| --- | --- | --- |
| **Tap feedback** | Provide clear pressed feedback (ripple/opacity/elevation) within 80-150ms | No visual response on tap |
| **Animation timing** | Keep micro-interactions around 150-300ms with platform-native easing | Instant transitions or slow animations (\>500ms) |
| **Accessibility focus** | Ensure screen reader focus order matches visual order and labels are descriptive | Unlabeled controls or confusing focus traversal |
| **Disabled state clarity** | Use disabled semantics (`disabled`/native disabled props), reduced emphasis, and no tap action | Controls that look tappable but do nothing |
| **Touch target minimum** | Keep tap areas \>=44x44pt (iOS) or \>=48x48dp (Android), expand hit area when icon is smaller | Tiny tap targets or icon-only hit areas without padding |
| **Gesture conflict prevention** | Keep one primary gesture per region and avoid nested tap/drag conflicts | Overlapping gestures causing accidental actions |
| **Semantic native controls** | Prefer native interactive primitives (`Button`, `Pressable`, platform equivalents) with proper accessibility roles | Generic containers used as primary controls without semantics |

### 浅色/深色模式对比度

| Rule | Do | Don't |
| --- | --- | --- |
| **Surface readability (light)** | Keep cards/surfaces clearly separated from background with sufficient opacity/elevation | Overly transparent surfaces that blur hierarchy |
| **Text contrast (light)** | Maintain body text contrast \>=4.5:1 against light surfaces | Low-contrast gray body text |
| **Text contrast (dark)** | Maintain primary text contrast \>=4.5:1 and secondary text \>=3:1 on dark surfaces | Dark mode text that blends into background |
| **Border and divider visibility** | Ensure separators are visible in both themes (not just light mode) | Theme-specific borders disappearing in one mode |
| **State contrast parity** | Keep pressed/focused/disabled states equally distinguishable in light and dark themes | Defining interaction states for one theme only |
| **Token-driven theming** | Use semantic color tokens mapped per theme across app surfaces/text/icons | Hardcoded per-screen hex values |
| **Scrim and modal legibility** | Use a modal scrim strong enough to isolate foreground content (typically 40-60% black) | Weak scrim that leaves background visually competing |

### 布局与间距

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

## 交付前检查清单

在交付 UI 代码之前，请验证以下项目：范围说明：此清单适用于 App UI（iOS/Android/React Native/Flutter）。

### 视觉质量

- [ ] 不使用表情符号作为图标（改用 SVG）
- [ ] 所有图标均来自一致的图标系列和风格
- [ ] 使用官方品牌素材，并保持正确的比例和留白
- [ ] 按下状态的视觉效果不会改变布局边界或引起抖动
- [ ] 语义化主题令牌使用一致（不为每个页面临时硬编码颜色）

### 交互

- [ ] 所有可点击元素都提供清晰的按下反馈（涟漪/不透明度/层级提升）
- [ ] 触控目标达到最小尺寸（iOS \>=44x44pt，Android \>=48x48dp）
- [ ] 微交互时长保持在150-300ms范围内，并使用符合原生手感的缓动
- [ ] 禁用状态在视觉上清晰可辨且不可交互
- [ ] 屏幕阅读器的焦点顺序与视觉顺序一致，且交互标签具有描述性
- [ ] 手势区域避免嵌套/冲突交互（点击/拖动/返回滑动冲突）

### 浅色/深色模式

- [ ] 主文本在浅色和深色模式下的对比度均 \>=4.5:1
- [ ] 次要文本在浅色和深色模式下的对比度均 \>=3:1
- [ ] 分隔线/边框和交互状态在两种模式下都能区分
- [ ] 模态框/抽屉遮罩层的不透明度足够高，以保持前景内容可读性（通常为40-60%的黑色）
- [ ] 在交付前会测试两种主题（而不是仅根据单一主题推断）

### 布局

- [ ] 标题栏、标签栏和底部 CTA 栏已正确避开安全区域
- [ ] 可滚动内容不会被固定/吸顶栏遮挡
- [ ] 已在小屏手机、大屏手机和平板（竖屏 + 横屏）上验证
- [ ] 水平内边距/栏间距可根据设备尺寸和方向正确调整
- [ ] 在组件、区块和页面层面都保持 4/8dp 的间距节奏
- [ ] 在较大设备上，长篇文本的行宽仍保持可读性（不是边到边的段落）

### 可访问性

- [ ] 所有有意义的图片/图标都具有无障碍标签
- [ ] 表单字段具有标签、提示和清晰的错误信息
- [ ] 颜色不是唯一的指示方式
- [ ] 在不破坏布局的情况下支持减少动态效果和动态文本大小
- [ ] 无障碍特征/角色/状态（已选中、已禁用、已展开）会被正确播报
