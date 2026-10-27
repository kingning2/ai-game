import type { ComponentType } from 'react';

import AgentCardBasic from '././AgentCard/basic';
import AgentHandoffBasic from '././AgentHandoff/basic';
import AgentTeamBoardBasic from '././AgentTeamBoard/basic';
import AgentTimelineBasic from '././AgentTimeline/basic';
import AgentTreeBasic from '././AgentTree/basic';
import ApprovalGateBasic from '././ApprovalGate/basic';
import MemoryPanelBasic from '././MemoryPanel/basic';

/**
 * Demo 登记：key 供 <DemoPreview name="..." /> 引用。
 * 新增示例：在此登记 + 在组件页 md 里写
 *   <DemoPreview name="Xxx" />
 *   ```tsx
 *   <<< ./demos/Xxx/scene.tsx
 *   ```
 */
import AttachmentBasic from '././AttachmentList/basic';
import AttachmentNoPreview from '././AttachmentList/no-preview';
import AttachmentRemove from '././AttachmentList/remove';
import AttachmentSize from '././AttachmentList/size';
import AvatarBasic from '././Avatar/basic';
import AvatarCustom from '././Avatar/custom';
import AvatarImage from '././Avatar/image';
import AvatarList from '././Avatar/list';
import AvatarSize from '././Avatar/size';
import BadgeBasic from '././Badge/basic';
import BadgeCustom from '././Badge/custom';
import BadgeStatus from '././Badge/status';
import BadgeTone from '././Badge/tone';
import ButtonBasic from '././Button/basic';
import ButtonDisabled from '././Button/disabled';
import ButtonIcon from '././Button/icon';
import ButtonLoading from '././Button/loading';
import ButtonSize from '././Button/size';
import ButtonVariant from '././Button/variant';
import CardBasic from '././Card/basic';
import CardGrid from '././Card/grid';
import CardInteractive from '././Card/interactive';
import CardProfile from '././Card/profile';
import CardStat from '././Card/stat';
import CheckboxAgreement from '././Checkbox/agreement';
import CheckboxBasic from '././Checkbox/basic';
import CheckboxDisabled from '././Checkbox/disabled';
import CheckboxGroup from '././Checkbox/group';
import ComposerAttachments from '././Composer/attachments';
import ComposerBasic from '././Composer/basic';
import ComposerChat from '././Composer/chat';
import ComposerDisabled from '././Composer/disabled';
import ComposerMaxRows from '././Composer/max-rows';
import ComposerModels from '././Composer/models';
import ComposerStreaming from '././Composer/streaming';
import ContainerBasic from '././Container/basic';
import ContainerPage from '././Container/page';
import ContainerSize from '././Container/size';
import DataTableBasic from '././DataTable/basic';
import DataTableColumns from '././DataTable/columns';
import DataTableEmpty from '././DataTable/empty';
import DataTableFull from '././DataTable/full';
import DataTablePagination from '././DataTable/pagination';
import DataTableSelection from '././DataTable/selection';
import DataTableSort from '././DataTable/sort';
import DividerBasic from '././Divider/basic';
import DividerCombine from '././Divider/combine';
import DividerInset from '././Divider/inset';
import DividerVertical from '././Divider/vertical';
import FormAsyncSubmit from '././Form/async-submit';
import FormBasic from '././Form/basic';
import FormCheckboxRequired from '././Form/checkbox-required';
import FormFields from '././Form/fields';
import FormFull from '././Form/full';
import FormReset from '././Form/reset';
import FormValidation from '././Form/validation';
import InputBasic from '././Input/basic';
import InputDisabled from '././Input/disabled';
import InputError from '././Input/error';
import InputTypes from '././Input/types';
import LightboxBasic from '././Lightbox/basic';
import LightboxCaption from '././Lightbox/caption';
import LightboxControlled from '././Lightbox/controlled';
import LightboxCustom from '././Lightbox/custom';
import LightboxGallery from '././Lightbox/gallery';
import ListBasic from '././List/basic';
import ListClickable from '././List/clickable';
import ListMedia from '././List/media';
import ListSettings from '././List/settings';
import ListTrailing from '././List/trailing';
import MarkdownBasic from '././Markdown/basic';
import MarkdownCode from '././Markdown/code';
import MarkdownCustom from '././Markdown/custom';
import MarkdownGfm from '././Markdown/gfm';
import MarkdownStreaming from '././Markdown/streaming';
import MarkdownUnstyled from '././Markdown/unstyled';
import NavBarBasic from '././NavBar/basic';
import NavBarCentered from '././NavBar/centered';
import NavBarLeading from '././NavBar/leading';
import NavBarScroll from '././NavBar/scroll';
import NavBarStatic from '././NavBar/static';
import NavBarTrailing from '././NavBar/trailing';
import ProgressBasic from '././Progress/basic';
import ProgressClamp from '././Progress/clamp';
import ProgressDynamic from '././Progress/dynamic';
import ProgressUpload from '././Progress/upload';
import SectionHeaderBasic from '././SectionHeader/basic';
import SectionHeaderForm from '././SectionHeader/form';
import SectionHeaderList from '././SectionHeader/list';
import SegmentedBasic from '././SegmentedControl/basic';
import SegmentedContent from '././SegmentedControl/content';
import SegmentedFilter from '././SegmentedControl/filter';
import SegmentedFullWidth from '././SegmentedControl/full-width';
import SelectBadge from '././Select/badge';
import SelectBasic from '././Select/basic';
import SelectDisabled from '././Select/disabled';
import SelectGhost from '././Select/ghost';
import SliderBasic from '././Slider/basic';
import MessageBasic from '././Message/basic';
import MessageScroll from '././Message/scroll';
import TaskPlanBasic from '././TaskPlan/basic';
import TaskPlanAdvance from '././TaskPlan/advance';
import ApprovalCardBasic from '././ApprovalCard/basic';
import ApprovalCardSettled from '././ApprovalCard/settled';
import ConversationListBasic from '././ConversationList/basic';
import ErrorBannerKinds from '././ErrorBanner/kinds';
import ToastBasic from '././Toast/basic';
import PromptSuggestionsBasic from '././PromptSuggestions/basic';
import SkillCardBasic from '././SkillCard/basic';
import SkillCardToggle from '././SkillCard/toggle';
import SliderStep from '././Slider/step';
import SliderSynced from '././Slider/synced';
import SliderWithIcons from '././Slider/with-icons';
import SpinnerBasic from '././Spinner/basic';
import SpinnerColor from '././Spinner/color';
import SpinnerInline from '././Spinner/inline';
import SpinnerPlaceholder from '././Spinner/placeholder';
import StackAlign from '././Stack/align';
import StackBasic from '././Stack/basic';
import StackCombine from '././Stack/combine';
import StackDirection from '././Stack/direction';
import StackGap from '././Stack/gap';
import SwitchBasic from '././Switch/basic';
import SwitchDisabled from '././Switch/disabled';
import SwitchGroup from '././Switch/group';
import SwitchSettingRow from '././Switch/setting-row';
import ThinkingCollapsed from '././Thinking/collapsed';
import ThinkingControlled from '././Thinking/controlled';
import ThinkingReview from '././Thinking/review';
import ThinkingStreaming from '././Thinking/streaming';
import ToolExecutionBasic from '././ToolExecution/basic';
import WebSearchBasic from '././WebSearch/basic';
import WerewolfStageBasic from '././WerewolfStage/basic';
import SettingsPanelBasic from '././SettingsPanel/basic';
import WebSearchLive from '././WebSearch/live';
import ToolExecutionSequence from '././ToolExecution/sequence';
import TypewriterBasic from '././Typewriter/basic';
import TypewriterCursor from '././Typewriter/cursor';
import TypewriterFinished from '././Typewriter/finished';
import TypewriterSpeed from '././Typewriter/speed';
import TypewriterStream from '././Typewriter/stream';
import UsageBadgeBasic from '././Usage/badge-basic';
import UsageCombined from '././Usage/combined';
import UsageFormatTokens from '././Usage/format-tokens';
import UsageMeterBasic from '././Usage/meter-basic';
import UsageMeterLive from '././Usage/meter-live';

export const demos: Record<string, ComponentType> = {
  AgentCardBasic,
  AgentHandoffBasic,
  AgentTeamBoardBasic,
  AgentTimelineBasic,
  AgentTreeBasic,
  ApprovalGateBasic,
  MemoryPanelBasic,
  AttachmentBasic,
  AttachmentNoPreview,
  AttachmentRemove,
  AttachmentSize,
  AvatarBasic,
  AvatarCustom,
  AvatarImage,
  AvatarList,
  AvatarSize,
  BadgeBasic,
  BadgeCustom,
  BadgeStatus,
  BadgeTone,
  ButtonBasic,
  ButtonDisabled,
  ButtonIcon,
  ButtonLoading,
  ButtonSize,
  ButtonVariant,
  CardBasic,
  CardGrid,
  CardInteractive,
  CardProfile,
  CardStat,
  CheckboxAgreement,
  CheckboxBasic,
  CheckboxDisabled,
  CheckboxGroup,
  ComposerAttachments,
  ComposerBasic,
  ComposerChat,
  ComposerDisabled,
  ComposerMaxRows,
  ComposerModels,
  ComposerStreaming,
  ContainerBasic,
  ContainerPage,
  ContainerSize,
  DataTableBasic,
  DataTableColumns,
  DataTableEmpty,
  DataTableFull,
  DataTablePagination,
  DataTableSelection,
  DataTableSort,
  DividerBasic,
  DividerCombine,
  DividerInset,
  DividerVertical,
  FormAsyncSubmit,
  FormBasic,
  FormCheckboxRequired,
  FormFields,
  FormFull,
  FormReset,
  FormValidation,
  InputBasic,
  InputDisabled,
  InputError,
  InputTypes,
  LightboxBasic,
  LightboxCaption,
  LightboxControlled,
  LightboxCustom,
  LightboxGallery,
  ListBasic,
  ListClickable,
  ListMedia,
  ListSettings,
  ListTrailing,
  MarkdownBasic,
  MarkdownCode,
  MarkdownCustom,
  MarkdownGfm,
  MarkdownStreaming,
  MarkdownUnstyled,
  NavBarBasic,
  NavBarCentered,
  NavBarLeading,
  NavBarScroll,
  NavBarStatic,
  NavBarTrailing,
  ProgressBasic,
  ProgressClamp,
  ProgressDynamic,
  ProgressUpload,
  SectionHeaderBasic,
  SectionHeaderForm,
  SectionHeaderList,
  SegmentedBasic,
  SegmentedContent,
  SegmentedFilter,
  SegmentedFullWidth,
  SelectBadge,
  SelectBasic,
  SelectDisabled,
  SelectGhost,
  SliderBasic,
  SkillCardBasic,
  SkillCardToggle,
  MessageBasic,
  MessageScroll,
  TaskPlanBasic,
  TaskPlanAdvance,
  ApprovalCardBasic,
  ApprovalCardSettled,
  ConversationListBasic,
  ErrorBannerKinds,
  ToastBasic,
  PromptSuggestionsBasic,
  SliderStep,
  SliderSynced,
  SliderWithIcons,
  SpinnerBasic,
  SpinnerColor,
  SpinnerInline,
  SpinnerPlaceholder,
  StackAlign,
  StackBasic,
  StackCombine,
  StackDirection,
  StackGap,
  SwitchBasic,
  SwitchDisabled,
  SwitchGroup,
  SwitchSettingRow,
  ThinkingCollapsed,
  ThinkingControlled,
  ThinkingReview,
  ThinkingStreaming,
  ToolExecutionBasic,
  ToolExecutionSequence,
  WebSearchBasic,
  WerewolfStageBasic,
  SettingsPanelBasic,
  WebSearchLive,
  TypewriterBasic,
  TypewriterCursor,
  TypewriterFinished,
  TypewriterSpeed,
  TypewriterStream,
  UsageBadgeBasic,
  UsageCombined,
  UsageFormatTokens,
  UsageMeterBasic,
  UsageMeterLive,
};
