type EvidenceMotionState = {
  visible: boolean;
  position: 'right' | 'bottom';
  reducedMotion: boolean;
  suspended: boolean;
};

/** Presentation only. The caller owns visibility, scope, focus and grid placement. */
export function evidenceMotion(node: HTMLElement, initial: EvidenceMotionState) {
  let state = initial;
  let animation: Animation | undefined;
  let generation = 0;
  const preference = window.matchMedia('(prefers-reduced-motion: reduce)');
  const original = { opacity: node.style.opacity, transform: node.style.transform };

  function settle() {
    generation += 1;
    animation?.cancel();
    animation = undefined;
    node.style.opacity = original.opacity;
    node.style.transform = original.transform;
  }

  function reveal() {
    if (!state.visible || state.suspended || state.reducedMotion || preference.matches) {
      settle();
      return;
    }
    // Retarget from the currently presented frame when the user moves the panel
    // again. CSS and drag pickup never own this wrapper's transform.
    const continuing = !!animation;
    const opacity = continuing ? getComputedStyle(node).opacity : '0.65';
    const transform = continuing ? getComputedStyle(node).transform
      : state.position === 'right' ? 'translateX(8px)' : 'translateY(8px)';
    animation?.cancel();
    const current = ++generation;
    animation = node.animate([
      { opacity, transform },
      { opacity: '1', transform: original.transform || 'none' },
    ], { duration: 200, easing: 'cubic-bezier(0.2, 0, 0, 1)', fill: 'both' });
    void animation.finished.then(() => { if (current === generation) settle(); }).catch(() => {});
  }

  const preferenceChanged = () => { if (preference.matches) settle(); };
  preference.addEventListener('change', preferenceChanged);
  if (state.visible) reveal();
  return {
    update(next: EvidenceMotionState) {
      const changed = next.visible !== state.visible || next.position !== state.position;
      state = next;
      if (!next.visible || next.suspended || next.reducedMotion || preference.matches) settle();
      else if (changed) reveal();
    },
    destroy() {
      preference.removeEventListener('change', preferenceChanged);
      settle();
    },
  };
}
