/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Fingerprint of the bridge snapshot's app/route/version context — §5.8 G3.
 *
 * Apps may surface additional fingerprint fields in future; new fields
 * require a schema bump (`deny_unknown_fields` is set).
 */
export interface BridgeFingerprint {
  /**
   * Stable application identifier (e.g. `"qontinui-web"`,
   * `"qontinui-runner"`).
   */
  appId: string;
  /**
   * Optional app version string (semver or git short-SHA).
   */
  appVersion?: string | null;
  /**
   * Optional UI Bridge SDK version.
   */
  bridgeVersion?: string | null;
  /**
   * Number of elements visible in the snapshot.
   */
  elementCount: number;
  /**
   * Optional route path / hash at snapshot time.
   */
  route?: string | null;
  /**
   * ISO-8601 UTC timestamp when the snapshot was indexed.
   */
  snapshotTimestamp: string;
}
