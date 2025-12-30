/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 */

export enum ImageFormat {
  PNG = "png",
  JPG = "jpg",
  JPEG = "jpeg",
}

export enum ImageSource {
  UPLOADED = "uploaded",
  PATTERN_OPTIMIZATION = "pattern_optimization",
  IMAGE_EXTRACTION = "image_extraction",
  STATE_DISCOVERY = "state_discovery",
}

export enum FailureStrategy {
  STOP = "stop",
  CONTINUE = "continue",
  PAUSE = "pause",
}

export enum SearchAlgorithm {
  TEMPLATE_MATCHING = "template_matching",
  FEATURE_MATCHING = "feature_matching",
  AI = "ai",
}

export enum ColorSpace {
  RGB = "rgb",
  GRAYSCALE = "grayscale",
  HSV = "hsv",
}

export enum LogLevel {
  DEBUG = "debug",
  INFO = "info",
  WARNING = "warning",
  ERROR = "error",
}

export enum TriggerType {
  TIME = "TIME",
  INTERVAL = "INTERVAL",
  STATE = "STATE",
  MANUAL = "MANUAL",
}

export enum CheckMode {
  CHECK_ALL = "CHECK_ALL",
  CHECK_INACTIVE_ONLY = "CHECK_INACTIVE_ONLY",
}

export enum ScheduleType {
  FIXED_RATE = "FIXED_RATE",
  FIXED_DELAY = "FIXED_DELAY",
}

export interface Category {
  /** Category name (e.g., 'Main', 'Testing') */
  name: string;
  /** Whether workflows in this category are available for automation */
  automationEnabled?: boolean;
}

export interface Resolution {
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
}

export interface ExecutionSettings {
  /** Default action timeout in milliseconds */
  defaultTimeout?: number;
  /** Default number of retry attempts */
  defaultRetryCount?: number;
  /** Delay between actions in milliseconds */
  actionDelay?: number;
  /** How to handle action failures */
  failureStrategy?: FailureStrategy;
  /** Run in headless mode */
  headless?: boolean;
  /** Target screen resolution */
  resolution?: Resolution | null;
}

export interface RecognitionSettings {
  /** Default similarity threshold */
  defaultThreshold?: number;
  /** Image search algorithm */
  searchAlgorithm?: SearchAlgorithm;
  /** Enable multi-scale search */
  multiScaleSearch?: boolean;
  /** Color space for matching */
  colorSpace?: ColorSpace;
  /** Enable edge detection preprocessing */
  edgeDetection?: boolean;
  /** Enable OCR text recognition */
  ocrEnabled?: boolean;
  /** OCR language code */
  ocrLanguage?: string;
}

export interface LoggingSettings {
  /** Log level */
  level?: LogLevel;
  /** Capture screenshot on errors */
  screenshotOnError?: boolean;
  /** Log file path */
  logFile?: string | null;
  /** Output logs to console */
  consoleOutput?: boolean;
  /** Log detailed matching information */
  detailedMatching?: boolean;
}

export interface PerformanceSettings {
  /** Maximum parallel action execution */
  maxParallelActions?: number;
  /** CPU usage limit percentage */
  cpuLimit?: number | null;
  /** Memory limit in MB */
  memoryLimit?: number | null;
  /** Cache images in memory */
  cacheImages?: boolean;
  /** Enable search optimization */
  optimizeSearch?: boolean;
}

export interface MouseActionSettings {
  /** Duration to hold mouse button */
  click_hold_duration?: number;
  /** Delay after releasing mouse button */
  click_release_delay?: number;
  /** Ensure mouse button is released */
  click_safety_release?: boolean;
  /** Interval between double-click clicks */
  double_click_interval?: number;
  /** Delay before starting drag */
  drag_start_delay?: number;
  /** Delay after ending drag */
  drag_end_delay?: number;
  /** Default drag duration */
  drag_default_duration?: number;
  /** Default move duration */
  move_default_duration?: number;
  /** Safety release delay */
  safety_release_delay?: number;
}

export interface KeyboardActionSettings {
  /** Duration to hold key */
  key_hold_duration?: number;
  /** Delay after releasing key */
  key_release_delay?: number;
  /** Interval between typed characters */
  typing_interval?: number;
  /** Duration to hold hotkey combination */
  hotkey_hold_duration?: number;
  /** Interval between hotkey presses */
  hotkey_press_interval?: number;
}

export interface FindActionSettings {
  /** Default find timeout in milliseconds */
  default_timeout?: number;
  /** Default retry count */
  default_retry_count?: number;
  /** Search polling interval in milliseconds */
  search_interval?: number;
}

export interface WaitActionSettings {
  /** Default pause before action */
  pause_before_action?: number;
  /** Default pause after action */
  pause_after_action?: number;
}

export interface ConfigSettings {
  /** Execution control settings */
  execution?: ExecutionSettings;
  /** Image recognition settings */
  recognition?: RecognitionSettings;
  /** Logging configuration */
  logging?: LoggingSettings | null;
  /** Performance tuning settings */
  performance?: PerformanceSettings | null;
  /** Mouse action timing */
  mouse?: MouseActionSettings | null;
  /** Keyboard action timing */
  keyboard?: KeyboardActionSettings | null;
  /** Find action defaults */
  find?: FindActionSettings | null;
  /** Wait action defaults */
  wait?: WaitActionSettings | null;
}

export interface CompatibleVersions {
  /** Compatible runner version */
  runner: string;
  /** Compatible website version */
  website: string;
}

export interface ConfigMetadata {
  /** Project/configuration name */
  name: string;
  /** Description */
  description?: string | null;
  /** Author name */
  author?: string | null;
  /** ISO 8601 creation timestamp */
  created: string;
  /** ISO 8601 last modified timestamp */
  modified: string;
  /** Tags for categorization */
  tags?: string[];
  /** Target application being automated */
  targetApplication?: string | null;
  /** Version compatibility information */
  compatibleVersions?: CompatibleVersions | null;
}

export interface ImageAsset {
  /** Unique identifier for the image */
  id: string;
  /** Human-readable name */
  name: string;
  /** Base64 encoded image data */
  data: string;
  /** Image format (png, jpg, jpeg) */
  format: ImageFormat;
  /** Image width in pixels */
  width: number;
  /** Image height in pixels */
  height: number;
  /** SHA256 hash for integrity verification */
  hash?: string | null;
  /** Optional base64 encoded mask image */
  mask?: string | null;
  /** S3 object key for cloud storage */
  s3Key?: string | null;
  /** ISO 8601 timestamp when presigned URL expires */
  urlExpiresAt?: string | null;
  /** Version number (default: 1) */
  version?: number | null;
  /** ID of the original image if this is a version */
  parentImageId?: string | null;
  /** Array of version IDs (only on parent images) */
  versions?: string[] | null;
  /** Monitor indices where this image should be used (default: [0]) */
  monitors?: number[] | null;
}

export interface Schedule {
  /** Unique identifier */
  id: string;
  /** Schedule name */
  name: string;
  /** Workflow to run */
  workflowId: string;
  /** Description */
  description?: string | null;
  /** Trigger type */
  triggerType: TriggerType;
  /** State check mode */
  checkMode: CheckMode;
  /** Schedule type */
  scheduleType: ScheduleType;
  /** Cron expression for TIME */
  cronExpression?: string | null;
  /** Interval in seconds */
  intervalSeconds?: number | null;
  /** State that triggers execution */
  triggerState?: string | null;
  /** Max iterations */
  maxIterations?: number | null;
  /** Delay between checks */
  stateCheckDelaySeconds?: number;
  /** Delay before rebuilding */
  stateRebuildDelaySeconds?: number;
  /** Failures before action */
  failureThreshold?: number;
  /** Is schedule enabled */
  enabled?: boolean;
  /** Creation timestamp */
  createdAt?: string | null;
  /** Last execution timestamp */
  lastExecutedAt?: string | null;
}

export interface ExecutionRecord {
  /** Unique identifier */
  id: string;
  /** Schedule ID */
  scheduleId: string;
  /** Workflow ID */
  workflowId: string;
  /** Start timestamp */
  startTime: string;
  /** End timestamp */
  endTime?: string | null;
  /** Whether execution succeeded */
  success: boolean;
  /** Number of iterations */
  iterationCount?: number;
  /** Error messages */
  errors?: string[];
  /** Additional metadata */
  metadata?: Record<string, any>;
}
