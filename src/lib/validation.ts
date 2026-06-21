// Single source of truth for profile validation.
//
// A CasparCG config can be rejected by the server in ways that are tedious to
// diagnose from the log ("Failed to enable external keyer", "Could not enable
// primary video output"). Rather than let the user discover those at launch,
// every rule that the GUI can check ahead of time lives here, in one pure
// function. The UI consumes the result to grey out impossible choices, show
// inline feedback, and gate the Start button — so an invalid profile can never
// be launched, and ideally never even be constructed.

import type {
  GlobalConfig,
  DeckLinkDevice,
  DeckLinkConsumer,
} from './types';

export type Severity = 'error' | 'warning';

export interface ValidationIssue {
  /** Stable identifier so React keys and de-duplication are deterministic. */
  id: string;
  severity: Severity;
  /** 0-based channel index, when the issue is tied to a channel. */
  channelIndex?: number;
  /** 0-based consumer index within the channel, when tied to a consumer. */
  consumerIndex?: number;
  /** Field the issue concerns (e.g. 'device', 'keyer', 'key_device'). */
  field?: string;
  message: string;
}

/** A single claim on a physical DeckLink device made by a consumer. */
interface DeviceClaim {
  device: number;
  channelIndex: number;
  consumerIndex: number;
  role: 'fill' | 'key';
}

/** Every physical-device claim made across the whole profile. */
export function collectDeviceClaims(config: GlobalConfig): DeviceClaim[] {
  const claims: DeviceClaim[] = [];
  config.caspar.channels.forEach((ch, channelIndex) => {
    ch.consumers.forEach((cons, consumerIndex) => {
      if (cons.type !== 'decklink') return;
      claims.push({ device: cons.device, channelIndex, consumerIndex, role: 'fill' });
      // A DeckLink consumer only opens its key device in External (Separate
      // Device) mode; in every other keyer mode the key device is ignored and
      // so makes no claim on the hardware. Narrow key_device locally so the
      // claim never carries an undefined device.
      if (cons.keyer === 'external_separate_device' && cons.key_device !== undefined) {
        claims.push({ device: cons.key_device, channelIndex, consumerIndex, role: 'key' });
      }
    });
  });
  return claims;
}

/**
 * Physical devices already claimed by DeckLink consumers OTHER than the one at
 * (channelIndex, consumerIndex). The pickers use this to grey out cards that
 * are already taken, so a clash cannot be created in the first place.
 */
export function devicesClaimedElsewhere(
  config: GlobalConfig,
  channelIndex: number,
  consumerIndex: number,
): Set<number> {
  const taken = new Set<number>();
  for (const claim of collectDeviceClaims(config)) {
    if (claim.channelIndex === channelIndex && claim.consumerIndex === consumerIndex) continue;
    taken.add(claim.device);
  }
  return taken;
}

/** Validate a whole profile against the currently detected hardware. */
export function validateConfig(
  config: GlobalConfig,
  devices: DeckLinkDevice[],
): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const haveEnumeration = devices.length > 0;
  const deviceByIndex = new Map(devices.map((d) => [d.index, d] as const));

  // A physical card can be opened exactly once. Flag any device claimed by more
  // than one distinct consumer (claiming it twice from the same consumer is the
  // key-device-equals-fill-device case, handled by the per-consumer rule below).
  const claimsByDevice = new Map<number, DeviceClaim[]>();
  for (const claim of collectDeviceClaims(config)) {
    const list = claimsByDevice.get(claim.device) ?? [];
    list.push(claim);
    claimsByDevice.set(claim.device, list);
  }
  for (const [device, claims] of claimsByDevice) {
    const distinctConsumers = new Set(claims.map((c) => `${c.channelIndex}:${c.consumerIndex}`));
    if (distinctConsumers.size > 1) {
      const label = deviceByIndex.get(device)?.display_name ?? `Device ${device}`;
      for (const claim of claims) {
        issues.push({
          id: `clash:${device}:${claim.channelIndex}:${claim.consumerIndex}:${claim.role}`,
          severity: 'error',
          channelIndex: claim.channelIndex,
          consumerIndex: claim.consumerIndex,
          field: claim.role === 'fill' ? 'device' : 'key_device',
          message: `${label} is already used by another DeckLink consumer. A card can only be opened once.`,
        });
      }
    }
  }

  config.caspar.channels.forEach((ch, channelIndex) => {
    if (ch.consumers.length === 0) {
      issues.push({
        id: `empty-channel:${channelIndex}`,
        severity: 'warning',
        channelIndex,
        message: `Channel ${channelIndex + 1} has no consumers and will produce no output.`,
      });
    }
    ch.consumers.forEach((cons, consumerIndex) => {
      if (cons.type === 'decklink') {
        validateDeckLink(cons, channelIndex, consumerIndex, deviceByIndex, haveEnumeration, issues);
      } else if (cons.type === 'ndi' && !cons.name.trim()) {
        issues.push({
          id: `ndi-name:${channelIndex}:${consumerIndex}`,
          severity: 'warning',
          channelIndex,
          consumerIndex,
          field: 'name',
          message: 'NDI name is empty; CasparCG will fall back to "CasparCG".',
        });
      }
    });
  });

  const port = config.caspar.controllers.tcp.port;
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    issues.push({
      id: 'amcp-port',
      severity: 'error',
      field: 'port',
      message: `AMCP port ${port} is out of range (1–65535).`,
    });
  }

  return issues;
}

function validateDeckLink(
  c: DeckLinkConsumer,
  channelIndex: number,
  consumerIndex: number,
  deviceByIndex: Map<number, DeckLinkDevice>,
  haveEnumeration: boolean,
  issues: ValidationIssue[],
): void {
  const at = { channelIndex, consumerIndex };
  const dev = deviceByIndex.get(c.device);

  if (!dev) {
    issues.push({
      id: `dl-device-missing:${channelIndex}:${consumerIndex}`,
      severity: haveEnumeration ? 'error' : 'warning',
      ...at,
      field: 'device',
      message: haveEnumeration
        ? `Device ${c.device} is not among the detected DeckLink cards.`
        : `Device ${c.device} cannot be verified — no DeckLink cards were detected.`,
    });
  }

  // A keyer the card lacks is a hard failure ("Failed to enable … keyer").
  if (c.keyer === 'external' && dev && !dev.supports_external_keying) {
    issues.push({
      id: `dl-keyer-ext:${channelIndex}:${consumerIndex}`,
      severity: 'error',
      ...at,
      field: 'keyer',
      message: `${dev.model_name} has no external keyer. Use Default for plain fill, or External (Separate Device) with a key device.`,
    });
  }
  if (c.keyer === 'internal' && dev && !dev.supports_internal_keying) {
    issues.push({
      id: `dl-keyer-int:${channelIndex}:${consumerIndex}`,
      severity: 'error',
      ...at,
      field: 'keyer',
      message: `${dev.model_name} has no internal keyer.`,
    });
  }

  if (c.keyer === 'external_separate_device') {
    if (c.key_device === undefined) {
      issues.push({
        id: `dl-keydev-missing:${channelIndex}:${consumerIndex}`,
        severity: 'error',
        ...at,
        field: 'key_device',
        message: 'External (Separate Device) keying needs a key device.',
      });
    } else if (c.key_device === c.device) {
      issues.push({
        id: `dl-keydev-same:${channelIndex}:${consumerIndex}`,
        severity: 'error',
        ...at,
        field: 'key_device',
        message: 'Key device must be different from the fill device.',
      });
    } else if (haveEnumeration && !deviceByIndex.has(c.key_device)) {
      issues.push({
        id: `dl-keydev-hw:${channelIndex}:${consumerIndex}`,
        severity: 'error',
        ...at,
        field: 'key_device',
        message: `Key device ${c.key_device} is not among the detected DeckLink cards.`,
      });
    } else if (!haveEnumeration) {
      // Cannot confirm the key card exists, so do not hard-block — but surface
      // it, consistent with the unverifiable fill-device case above.
      issues.push({
        id: `dl-keydev-unverified:${channelIndex}:${consumerIndex}`,
        severity: 'warning',
        ...at,
        field: 'key_device',
        message: `Key device ${c.key_device} cannot be verified — no DeckLink cards were detected.`,
      });
    }
  } else if (c.key_device !== undefined) {
    issues.push({
      id: `dl-keydev-ignored:${channelIndex}:${consumerIndex}`,
      severity: 'warning',
      ...at,
      field: 'key_device',
      message: 'Key device is only used with External (Separate Device) keying; it will be ignored.',
    });
  }
}

/** Issues attached to a specific consumer. */
export function issuesForConsumer(
  issues: ValidationIssue[],
  channelIndex: number,
  consumerIndex: number,
): ValidationIssue[] {
  return issues.filter(
    (i) => i.channelIndex === channelIndex && i.consumerIndex === consumerIndex,
  );
}

/** Channel-level issues (not tied to a particular consumer). */
export function issuesForChannel(
  issues: ValidationIssue[],
  channelIndex: number,
): ValidationIssue[] {
  return issues.filter(
    (i) => i.channelIndex === channelIndex && i.consumerIndex === undefined,
  );
}

/** Profile-level issues (not tied to a channel). */
export function globalIssues(issues: ValidationIssue[]): ValidationIssue[] {
  return issues.filter((i) => i.channelIndex === undefined);
}

export function errorsOnly(issues: ValidationIssue[]): ValidationIssue[] {
  return issues.filter((i) => i.severity === 'error');
}
