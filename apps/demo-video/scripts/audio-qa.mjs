export const AUDIO_QA_LIMITS = Object.freeze({
  integratedLufsMin: -18,
  integratedLufsMax: -14,
  truePeakDbtpMax: -1,
  leadingSilenceSecondsMax: 1,
  trailingSilenceSecondsMax: 1,
  internalSilenceSecondsMax: 2,
  edgeToleranceSeconds: 0.1,
});

const asFiniteNumber = (value) => {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
};

export const parseLoudnorm = (stderr) => {
  const candidates = [...stderr.matchAll(/\{\s*"input_i"[\s\S]*?\}/g)];
  if (candidates.length === 0) return null;
  const parsed = JSON.parse(candidates.at(-1)[0]);
  return {
    integratedLufs: asFiniteNumber(parsed.input_i),
    truePeakDbtp: asFiniteNumber(parsed.input_tp),
    thresholdDb: asFiniteNumber(parsed.input_thresh),
  };
};

export const parseSilenceDetect = (stderr, durationSeconds) => {
  const events = [...stderr.matchAll(/silence_(start|end):\s*([0-9.]+)/g)].map(
    (match) => ({type: match[1], time: Number(match[2])}),
  );
  const segments = [];
  let pendingStart = null;
  for (const event of events) {
    if (event.type === "start") {
      pendingStart = event.time;
    } else if (pendingStart !== null) {
      segments.push({
        start: pendingStart,
        end: event.time,
        duration: Math.max(0, event.time - pendingStart),
      });
      pendingStart = null;
    }
  }
  if (pendingStart !== null && Number.isFinite(durationSeconds)) {
    segments.push({
      start: pendingStart,
      end: durationSeconds,
      duration: Math.max(0, durationSeconds - pendingStart),
    });
  }
  return segments;
};

export const evaluateAudioQa = ({
  loudness,
  silenceSegments,
  durationSeconds,
  limits = AUDIO_QA_LIMITS,
}) => {
  const errors = [];
  if (!loudness) return ["audio loudness: loudnorm did not emit parseable analysis"];

  if (!Number.isFinite(loudness.integratedLufs)) {
    errors.push("audio loudness: integrated loudness is not finite (audio may be silent)");
  } else if (
    loudness.integratedLufs < limits.integratedLufsMin ||
    loudness.integratedLufs > limits.integratedLufsMax
  ) {
    errors.push(
      `audio loudness: expected ${limits.integratedLufsMin} to ${limits.integratedLufsMax} LUFS, received ${loudness.integratedLufs}`,
    );
  }

  if (!Number.isFinite(loudness.truePeakDbtp)) {
    errors.push("audio true peak: loudnorm did not report a finite value");
  } else if (loudness.truePeakDbtp > limits.truePeakDbtpMax) {
    errors.push(
      `audio true peak: ceiling is ${limits.truePeakDbtpMax} dBTP, received ${loudness.truePeakDbtp}`,
    );
  }
  if (!Number.isFinite(loudness.thresholdDb)) {
    errors.push("audio silence threshold: loudnorm did not report a finite threshold");
  }
  if (!Number.isFinite(durationSeconds) || durationSeconds <= 0) {
    errors.push(`audio duration: expected a positive finite value, received ${durationSeconds}`);
    return errors;
  }

  for (const segment of silenceSegments) {
    const isLeading = segment.start <= limits.edgeToleranceSeconds;
    const isTrailing =
      segment.end >= durationSeconds - limits.edgeToleranceSeconds;
    if (isLeading && isTrailing) {
      errors.push(
        `audio silence: full-program silence detected (${segment.duration.toFixed(3)}s)`,
      );
    } else if (isLeading && segment.duration > limits.leadingSilenceSecondsMax) {
      errors.push(
        `audio silence: leading silence exceeds ${limits.leadingSilenceSecondsMax}s (${segment.duration.toFixed(3)}s)`,
      );
    } else if (isTrailing && segment.duration > limits.trailingSilenceSecondsMax) {
      errors.push(
        `audio silence: trailing silence exceeds ${limits.trailingSilenceSecondsMax}s (${segment.duration.toFixed(3)}s)`,
      );
    } else if (
      !isLeading &&
      !isTrailing &&
      segment.duration > limits.internalSilenceSecondsMax
    ) {
      errors.push(
        `audio silence: internal silence exceeds ${limits.internalSilenceSecondsMax}s (${segment.duration.toFixed(3)}s at ${segment.start.toFixed(3)}s)`,
      );
    }
  }
  return errors;
};
